use crate::domain::hash::Hash;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub fn write(repo_root: &Path, data: &[u8]) -> std::io::Result<Hash> {
    let hash = Hash::from_bytes(data);
    let path = hash.get_path(repo_root);

    if !path.exists() {
        fs::create_dir_all(path.parent().unwrap())?;
        let file = fs::File::create(path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(data)?;
        encoder.finish()?;
    }
    Ok(hash)
}

pub fn read(repo_root: &Path, hash: &Hash) -> std::io::Result<Vec<u8>> {
    let path = hash.get_path(repo_root);
    let file = fs::File::open(path)?;
    let mut decoder = ZlibDecoder::new(file);
    let mut contents = Vec::new();
    decoder.read_to_end(&mut contents)?;
    Ok(contents)
}
