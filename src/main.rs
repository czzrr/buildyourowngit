#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    //println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory")
    } else if args[1] == "cat-file" && args[2] == "-p" {
        let blob_sha = args[3].as_bytes();
        let blob_dir = &blob_sha[..2];
        let blob_file = &blob_sha[2..];
        let blob_contents = fs::read(format!(".git/objects/{}/{}", std::str::from_utf8(&blob_dir).unwrap(), std::str::from_utf8(&blob_file).unwrap())).unwrap();
        let mut decoder = flate2::bufread::ZlibDecoder::new(&blob_contents[..]);
        let mut decoded_blob = String::new();
        decoder.read_to_string(&mut decoded_blob).unwrap();
        //println!("{:?}", decoded_blob);
        let contents: String = decoded_blob.chars().skip_while(|c| c != &'\0').skip(1).collect();
        print!("{}", contents);
    } else {
        println!("unknown command: {}", args[1])
    }
}
