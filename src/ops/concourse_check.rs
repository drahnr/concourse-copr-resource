use errors::*;

extern crate crypto;
extern crate serde;

use self::crypto::digest::Digest;
use self::crypto::whirlpool::Whirlpool;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path,PathBuf};
use serde::de::{Deserialize,Deserializer};


#[derive(Serialize, Deserialize)]
pub struct ParamsSource {
	srpm_path : String,
}

#[derive(Serialize, Deserialize)]
pub struct ParamsVersion {
	digest : String,
}

#[derive(Serialize, Deserialize)]
pub struct Params {
	version: Option<ParamsVersion>,
	source : ParamsSource,
}


pub fn execute(params: Params) -> Result<()> {
	let mut digest = Whirlpool::new();
	let mut digest_result = [0u8; 8];

	let path = Path::new(&params.source.srpm_path);
	let path = path.to_str().ok_or("No way")?;
	let mut f = File::open(path).chain_err(||"wahtever")?;
	let mut buffer = [0u8; 16384];
	let mut bytes_read : u64 = 0;
	while true {
        println!("Starting buffer read...");
		let n = f.read(&mut buffer[..]).chain_err(||"Failed to read file")?;

		if n==0 {
			println!("Reading file completed");
			digest.result(&mut digest_result);
		}
		bytes_read += n as u64;
		println!("Read {} bytes of {} completed", bytes_read, path);
		digest.input(&buffer[0..n]);

	}
	if digest_result != [0; 8] {
		println!("most likely a valid digest");
	}
	Ok(())
}
