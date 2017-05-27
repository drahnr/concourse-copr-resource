use errors::*;


use walkdir::WalkDir;

extern crate crypto;

use self::crypto::digest::Digest;
use self::crypto::whirlpool::Whirlpool;

extern crate hyper;
extern crate multipart;

use self::hyper::Client;
use self::hyper::status::StatusCode;
use self::hyper::mime::{Mime, TopLevel, SubLevel};

use self::multipart::client::lazy::Multipart;

use std::fs::{File,metadata};
use std::io::prelude::*;
use std::vec::*;
use std::io::BufReader;
use std::path::{PathBuf,Path};
use regex::Regex;

use serde_json;

use ops::interface::*;
use ops::error::ResponseError;

use errors::Error;

#[derive(Serialize, Deserialize)]
pub struct Input {
    source : ResourceSource,
    params : Option<ResourceParams>,
}

#[derive(Serialize, Deserialize)]
pub struct MultipartRequestMetadata {
	project_id : u32,
	chroots: Vec<String>,
	enable_net: bool,
}

fn find_srpm_regex_match(dir : &PathBuf, srpm_regex : &String) -> Result<Option<PathBuf>> {

    let re = Regex::new(srpm_regex).chain_err(||"srpm regex failed to parse")?;

    let directory = Path::new(dir);

    for entry in WalkDir::new(&directory) {
    	let entry = entry.chain_err(||"WalkDir entry is useless")?;
    	let x = entry.path().to_str().ok_or("Failed to convert path")?;
        match re.captures(x) {
            Some(_) => {
                println!("{}", x);
                return Ok(Some(PathBuf::from(x)));
            },
            None => {
            }
        }
    }
    Ok(None)
}

fn calculate_whirlpool(path : &PathBuf) -> Result<[u8; 64]> {
	let mut digest = Whirlpool::new();
	let mut digest_result = [0u8; 64];

	let path = path.to_str().ok_or("Failed to convert path to string", )?;
	let mut f = File::open(path).chain_err(||format!("Failed to open"))?;
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
    Ok(digest_result)
}


pub fn execute(dir : PathBuf, input : Input) -> Result<()> {

    let params = input.params.unwrap_or_default();
    let chroots = params.chroots.unwrap_or_default();
    let enable_net = params.enable_net.unwrap_or_default();
    let max_n_bytes = params.max_n_bytes.unwrap_or_default();
	let meta = MultipartRequestMetadata { project_id : input.source.project_id,
	                                      chroots : chroots,
	                                      enable_net : enable_net,
	                                     };

    let path_srpm = find_srpm_regex_match(&dir, &input.source.regex).chain_err(||"Could not find any matches with that regex")?;
	let path_srpm = path_srpm.ok_or("No path found matching regex")?;

	let path_srpm_str = path_srpm.to_str().ok_or("No valid srpm path")?;
	let attr = metadata(path_srpm_str).chain_err(||"Failed to read metadata")?;
	if attr.len() > max_n_bytes {
        bail!("srpm is too damn big, exceeds maximum byte size of {}", max_n_bytes);
	}
	let mut f = File::open(path_srpm_str).chain_err(||"fun")?;
	let mut reader = BufReader::new(f);

    let name_srpm = path_srpm.file_name().ok_or("No srpm file name")?.to_str().ok_or("Failed to convert to string")?;
    let name_srpm = String::from(path_srpm_str);
    let mime_srpm : Mime =  "application/x-rpm".parse().unwrap();
    let mime_json : Mime = "application/json".parse().unwrap();

    let meta_json = serde_json::to_string(&meta).chain_err(||"Failed to serialize metadata")?;
	let response = Multipart::new()
		.add_stream::<_,_,String>("metadata", meta_json.as_bytes(), None, Some(mime_json))
		.add_stream::<_,_,String>("srpm", reader.take(max_n_bytes), Some(name_srpm), Some(mime_srpm))
			.client_request(&Client::new(), input.source.url.trim()).chain_err(||"Failed to send request")?;

	match response.status {
		StatusCode::BadRequest => { Err(ResponseError::InvalidRequest.into()) },
		StatusCode::Forbidden => { Err(ResponseError::AuthentificationFailure.into()) },
		StatusCode::Ok => { Ok(()) },
		_ => { Err(ResponseError::InvalidRequest.into()) },
	}
}
