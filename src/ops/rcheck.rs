use errors::*;

extern crate crypto;
extern crate serde;


use ops::interface::*;

use self::crypto::digest::Digest;
use self::crypto::whirlpool::Whirlpool;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path,PathBuf};
use serde::de::{Deserialize,Deserializer};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Input {
	version: Option<ResourceVersion>,
	source : ResourceSource,
}

pub fn execute(input: Input) -> Result<()> {
	let mut digest = Whirlpool::new();
	let mut digest_result = [0u8; 64];

	let path = Path::new(&input.source.srpm_path);
	let path = path.to_str().ok_or(format!("Failed to convert path to string {:?}", path))?;
	let mut f = File::open(path).chain_err(||format!("Failed to open {}", path))?;
	let mut buffer = [0u8; 16384];
	let mut bytes_read : u64 = 0;

	while true {
	    let n = f.read(&mut buffer[..]).chain_err(||"Failed to read file")?;
	    if n == 0 {
	        digest.result(&mut digest_result);
	        break;
	    }
		bytes_read += n as u64;
		digest.input(&buffer[0..n]);
	}

    let mut v : Vec<ResourceVersion> = Vec::new();

    let mut x: [u8; 32] = Default::default();
    x.copy_from_slice(&digest_result[0..32]);
    let version_current = ResourceVersion { digest : x, };
    // TODO tell me I am pretty
    match input.version {
        Some(version) => {
            if version != version_current {
                v.push(version_current);
            }
        },
        None => {}
    }
    let x = serde_json::to_string(&v)?;
    println!("{}", x);
    Ok(())
}
