use std::io::{self, Write};

use crate::common::hash_to_path;

pub fn run(hash: &str, parent_hash: &str, message: &str) -> anyhow::Result<()> {
    if !hash_to_path(&hash)?.exists() {
        return Err(anyhow::anyhow!("tree does not exist"));
    }
    if !hash_to_path(&parent_hash)?.exists() {
        return Err(anyhow::anyhow!("parent tree does not exist"));
    }

    let mut buf: Vec<u8> = Vec::new();
    buf.extend(b"tree ");
    buf.extend(hash.as_bytes());
    buf.extend(b"\n");
    buf.extend(b"parent ");
    buf.extend(parent_hash.as_bytes());
    buf.extend(b"\n");
    buf.extend(b"author John Doe <john@doe.com> 1710605448 +0100\n");
    buf.extend(b"committer John Doe <john@doe.com> 1710605448 +0100\n");
    buf.extend(b"\n");
    buf.extend(message.as_bytes());
    buf.extend(b"\n");

    let mut newbuf: Vec<u8> = Vec::new();
    newbuf.extend(b"commit ");
    newbuf.extend(buf.len().to_string().as_bytes());
    newbuf.extend(&buf);

    io::stdout().write_all(&buf).unwrap();

    Ok(())
}
