extern crate crypto;
 
use self::crypto::digest::Digest;
use self::crypto::whirlpool::Whirlpool;
 
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct ParamsSource {
	srpm_path : Path,
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


pub fn execute(params: Params) -> Result<(),String> {
	let mut digest = Whirlpool::new();
	let mut digest_result = [0; 8];

	let path = params.source.srpm_path;
	let mut f = File::open(path.to_str())?;
	let mut buffer = [0; 16384];
	let mut bytes_read : u64 = 0;
	loop {
		match f.read(&mut buffer[..]) {
			Ok(n) => {
				if n==0 {
					println!("Reading file completed");
					digest.result(&digest_result);
					break;
				}
				bytes_read += n;
				println!("Read {} bytes of {} completed", bytes_read, path.file_name().to_str());
				digest.input(buffer[0..n]);
			},
			Err(x) => {
				return Err("whatever");
			},
		};
	}
	if digest_result != [0; 8] {
		println!("most likely a valid digest");
	}
	Ok(())
}
