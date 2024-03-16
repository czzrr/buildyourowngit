use nom::{
    bytes::complete::{tag, take, take_while}, IResult
};

#[derive(Debug)]
struct ReferenceDiscovery {
    sha_ref_pairs: Vec<(String, String)>,
    capabilities: Vec<String>
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

        let mut sha_ref_pairs: Vec<(String, String)> = lines[2..lines.len() - 1].iter().map(|line| {
            let mut split = line[4..].split(' ');
            let sha = split.next().unwrap().to_owned();
            let reff = split.next().unwrap().to_owned();
            (sha, reff)
        }).collect();
        sha_ref_pairs.insert(0, (sha.to_owned(), reference.to_owned()));
        
        println!("--- FINISHED PARSING ---");

        Ok((input, ReferenceDiscovery { sha_ref_pairs, capabilities }))
    }
}

pub fn clone(repository_url: String) {
    println!("Cloning repository: {}", repository_url);

    let response = reqwest::blocking::get(format!(
        "{}/info/refs?service=git-upload-pack",
        repository_url
    ))
    .unwrap();
    let body = response.text().unwrap();
    println!("\n{}\n", body);

    let ref_disc = ReferenceDiscovery::parse(&body).unwrap().1;
    println!("{:?}", ref_disc);

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
