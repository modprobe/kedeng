use anyhow::anyhow;
use encoding::{Encoding, all::ISO_8859_1};
use std::fs;

pub fn read_iso_8859_1_file(path: &str) -> anyhow::Result<String> {
    let file_content = fs::read(path);
    if let Ok(file_content) = file_content {
        return ISO_8859_1
            .decode(&file_content, encoding::DecoderTrap::Strict)
            .or(Err(anyhow::anyhow!("couldnt decode file")));
    }

    Err(anyhow!("couldnt load file"))
}
