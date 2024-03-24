use std::io::{stdout, Write};

use nom::{
    bytes::complete::{tag, take, take_while},
    IResult,
};

#[allow(dead_code)]
#[derive(Debug)]
struct ReferenceDiscovery {
    sha_ref_pairs: Vec<(String, String)>,
    capabilities: Vec<String>,
}

impl ReferenceDiscovery {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        println!("--- PARSING REFERENCE DISCOVERY ---");
        let lines: Vec<&str> = input.split('\n').collect();

        // assert!(lines[0] == "001e# service=git-upload-pack");
        // assert!(&lines[1][..4] == "0000");

        // // First ref
        // let first_ref = &lines[1][4..];
        // dbg!(first_ref);
        // let size = usize::from_str_radix(&first_ref[..4], 16).unwrap();
        // dbg!(size);
        // let sha = &first_ref[4..44];

        // First line
        let input = lines[0];
        let _ = tag("001e# service=git-upload-pack")(input)?;
        let input = lines[1];
        let (input, _) = tag("0000")(input)?;

        // First ref
        let (input, _size) = take(4usize)(input)?;
        //let size = usize::from_str_radix(size, 16).unwrap();
        //let remaining_size = size - 4;
        //let (remaining_pkt_lines, input) = take(remaining_size)(input)?;
        println!("pkt_line: {}", input);

        let (input, sha) = take(40usize)(input)?;
        println!("sha: {}", sha);

        let (input, _) = tag(" ")(input)?;
        let (input, reference) = take_while(|b| b != '\0')(input)?;
        println!("ref: {}", reference);

        let (input, _) = tag("\0")(input)?;
        let capabilities: Vec<String> = input.split(|b| b == ' ').map(|c| c.to_owned()).collect();

        println!("capabilities: {:?}", capabilities);

        let mut sha_ref_pairs: Vec<(String, String)> = lines[2..lines.len() - 1]
            .iter()
            .map(|line| {
                let mut split = line[4..].split(' ');
                let sha = split.next().unwrap().to_owned();
                let reff = split.next().unwrap().to_owned();
                (sha, reff)
            })
            .collect();
        sha_ref_pairs.insert(0, (sha.to_owned(), reference.to_owned()));

        println!("--- FINISHED PARSING ---");

        Ok((
            input,
            ReferenceDiscovery {
                sha_ref_pairs,
                capabilities,
            },
        ))
    }
}

pub fn clone(repository_url: String) {
    println!("Cloning repository: {}", repository_url);

    let client = reqwest::blocking::Client::new();

    // let response = client
    //     .get(format!(
    //         "{}/info/refs?service=git-upload-pack",
    //         repository_url
    //     ))
    //     .header("git-protocol", "version=2").send()
    //     .unwrap();
    // let body = response.text().unwrap();
    // println!("\n{}\n", body);

    // let url = format!("{}/git-upload-pack", repository_url);
    // let body = format!("0014command=ls-refs\n0000");
    // let response = client
    //     .post(url)
    //     .header("git-protocol", "version=2")
    //     .body(body)
    //     .send()
    //     .unwrap();
    // println!("{:?}", response);
    // let body = response.text().unwrap();
    // println!("\n{}\n", body);

    let url = format!("{}/git-upload-pack", repository_url);
    // Hardcoded for now.
    // TODO: use hashes from ls-refs
    let body =
        "0011command=fetch00010032want 20f7295d14cbf2d4a12bf41d3a1b6bf17c04c6a3\n0009done\n0000";
    let response = client
        .post(url)
        .header("Git-Protocol", "version=2")
        .body(body)
        .send()
        .unwrap();
    //println!("{:?}", response);
    let body = response.bytes().unwrap();
    //dbg!(&body);

    let mut buf = &body[..];

    // Find start of PACK.
    // TODO: do this in a smarter way so that progress is printed
    loop {
        let size =
            usize::from_str_radix(&String::from_utf8(buf[..4].to_vec()).unwrap(), 16).unwrap();
        // What is the 0x01 byte?
        if buf[4..9] == b"\x01PACK"[..] {
            break;
        } else {
            buf = buf.get(size..).unwrap();
        }
    }
    println!("processing pack of len {}", buf.len());
    buf = &buf[9..];

    let protocol_version = u32::from_be_bytes(buf[..4].try_into().unwrap());
    dbg!(protocol_version);
    buf = &buf[4..];

    let num_objects = u32::from_be_bytes(buf[..4].try_into().unwrap());
    dbg!(num_objects);
    buf = &buf[4..];

    for _ in 0..num_objects {
        println!("\n--- processing object ---");
        let (object_type, idx, size) = pack_entry_size(buf);
        buf = &buf[idx..];
        // `size` is the size of the decompressed data, so we don't know how many bytes
        // to read from `buf`. Therefore, we just decompress data from `buf`.
        let mut decompressed_data = Vec::with_capacity(size);
        let offset_to_next_entry = decompress_stream(buf, &mut decompressed_data);
        assert!(decompressed_data.len() == size);
        // Write data to stdout for now.
        stdout().write_all(&decompressed_data).unwrap();

        buf = &buf[offset_to_next_entry..];
    }
    // TODO: collect pack entries into Vec.
}

fn decompress_stream(buf: &[u8], decompressed_data: &mut Vec<u8>) -> usize {
    let mut decompress = flate2::Decompress::new(true);
    decompress
        .decompress_vec(buf, decompressed_data, flate2::FlushDecompress::None)
        .unwrap();
    dbg!(decompress.total_in());

    decompress.total_in() as usize
}

fn pack_entry_size(buf: &[u8]) -> (u8, usize, usize) {
    let mut idx = 0;

    let mut c = buf[idx];
    idx += 1;

    let object_type = (c >> 4) & 0x7;
    dbg!(object_type);

    let mut size = (c & 0x0f) as usize;
    let mut shift: u8 = 4;

    while c & 0x80 != 0 {
        c = buf[idx];
        idx += 1;
        size += ((c & 0x7f) as usize) << shift;
        shift += 7;
    }
    dbg!(size);

    (object_type, idx, size)
}
