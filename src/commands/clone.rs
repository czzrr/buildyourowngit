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
    // let mut args = Vec::new();
    // args.push("0011command=fetch");
    // args.push("0014agent=git/2.34.1");
    // args.push("0016object-format=sha1");
    // args.push("000dthin-pack");
    // args.push("000dofs-delta");
    // args.push("0032want 20f7295d14cbf2d4a12bf41d3a1b6bf17c04c6a3");
    // args.push("0032want 20f7295d14cbf2d4a12bf41d3a1b6bf17c04c6a3");
    // args.push("0009done");
    // args.push("0000");
    // let body = args.join("\n");
    // println!("{}", body);
    // dbg!(body.len());
    let body = "0011command=fetch0032want 20f7295d14cbf2d4a12bf41d3a1b6bf17c04c6a3\n0009done\n0000";
    //let body = format!("0011command=fetch\n00320016object-format=sha1\n00dthin-pack\n000dofs-delta\nwant 003220f7295d14cbf2d4a12bf41d3a1b6bf17c04c6a3\n0009done\n0000");
    //let body = format!("0011command=fetch\00032want 003d20f7295d14cbf2d4a12bf41d3a1b6bf17c04c6a3\00009done\00000");
    let response = client
        .post(url)
        //.header("User-Agent", "git/2.34.1")
        .header("Git-Protocol", "version=2")
        //.header("Content-Type", "application/x-git-upload-pack-request")
        //.header("Accept", "application/x-git-upload-pack-result").body(body)
        //.header("Accept-Encoding", "deflate, gzip, br, zstd")
        .body(body).send()
        .unwrap();
    println!("{:?}", response);
    let body = response.bytes().unwrap();
    dbg!(&body);

    

    

    // let ref_disc = ReferenceDiscovery::parse(&body).unwrap().1;
    // println!("{:?}", ref_disc);

    // let mut wants = Vec::new();
    // wants.push("0014command=ls-refs".to_owned());
    // for (sha, _) in &ref_disc.sha_ref_pairs {
    //     wants.push(format!("0032want {}", sha));
    // }
    // wants.push("0000".to_owned());
    // wants.push("0009done\n".to_owned());

    // let wants = wants.join("\n");

    // println!("{}", wants);

    // let response = client
    //     .post(format!("{}/git-upload-pack", repository_url))
    //     .body(wants)
    //     .header("Content-Type", "application/x-git-upload-pack-request")
    //     .send()
    //     .unwrap();
    // println!("{:?}", response);
    // let body = response.text().unwrap();
    // println!("\n{}\n", body);

    // let lines: Vec<_> = body.split('\n').collect();

    // for line in &lines {
    //     println!("{}", line);
    // }

    // assert!(lines[0] == "001e# service=git-upload-pack");

    // let xs: Vec<&str> = lines[1][4..].split_whitespace().collect();

    // for x in &xs {
    //     println!("{}", x);
    // }

    // let sha = xs[0];
    // let reff = xs[1].split('\0').next().unwrap();

    // dbg!(&sha);
    // dbg!(&reff);

    // let pkt_lines = &lines[2..lines.len()-1];

    // println!("pkt_lines");
    // for pkt_line in pkt_lines {
    //     println!("{}", pkt_line);
    // }

    // let mut pkt_lines_pairs = Vec::new();

    // for pkt_line in pkt_lines {
    //     let split: Vec<_> = pkt_line.split_whitespace().collect();
    //     pkt_lines_pairs.push((split[0], split[1]));
    // }

    // println!("pkt_lines_pairs");
    // for pkt_line_pair in pkt_lines_pairs {
    //     println!("{:?}", pkt_line_pair);
    // }
}
