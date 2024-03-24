use crate::common::Object;
use std::io::{stdout, Write};

pub fn run(object_hash: &str) -> anyhow::Result<()> {
    let object = Object::read(object_hash)?;
    stdout().write_all(&object.contents)?;

    Ok(())
}
