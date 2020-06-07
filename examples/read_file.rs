use anyhow::Result;
use jagged::Archive;
use std::convert::TryFrom;
use std::path::Path;

fn main() -> Result<()> {
    let path = Path::new("release/jagex.jag");
    let archive = Archive::try_from(path)?;

    println!("archive entry size: {}", archive.len());
    Ok(())
}
