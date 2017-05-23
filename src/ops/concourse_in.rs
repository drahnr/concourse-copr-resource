use errors::*;

extern crate hyper;
extern crate multipart;

use self::hyper::Client;
use self::hyper::mime::{Mime, TopLevel, SubLevel};

use self::multipart::client::lazy::Multipart;

use std::fs::{File,metadata};
use std::io::prelude::*;
use std::path::PathBuf;
use std::vec::*;
use std::io::BufReader;

use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Params {
        project_id : u32,
        login: String,
        username: String,
        token: String,
        url: String,
        chroots: Vec<String>,
        enable_net: bool,
        max_n_bytes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MultipartRequestMetadata {
	project_id : u32,
	chroots: Vec<String>,
	enable_net: bool,
}

pub fn execute(path_to_srpm : PathBuf, params : Params) -> Result<()> {

	let max_n_bytes : u64 = 250_000_000 ;
	let meta = MultipartRequestMetadata { project_id : params.project_id,
	                                        chroots : params.chroots,
	                                        enable_net : params.enable_net,
	                                        };

	let attr = metadata(path_to_srpm.to_str().ok_or("No valid srpm path")?).chain_err(||"Failed to read metadata")?;
	if attr.len() > max_n_bytes {
        bail!("srpm is too damn big, exceeds maximum byte size");
	}
	let mut f = File::open(path_to_srpm.to_str().ok_or("No valid srpm path")?).chain_err(||"fun")?;
	let mut reader = BufReader::new(f);

    let path_srpm = path_to_srpm.file_name().ok_or("No srpm file name")?.to_str().ok_or("Failed to convert to string")?;
    let path_srpm = String::from(path_srpm);
    let mime_srpm : Mime =  "application/x-rpm".parse().unwrap();
    let mime_json : Mime = "application/json".parse().unwrap();

    let meta_json = serde_json::to_string(&meta).chain_err(||"Failed to serialize metadata")?;
	let _response = Multipart::new()
		.add_stream::<_,_,String>("metadata", meta_json.as_bytes(), None, Some(mime_json))
		.add_stream::<_,_,String>("srpm", reader.take(max_n_bytes), Some(path_srpm), Some(mime_srpm))
			.client_request(&Client::new(), params.url.trim()).chain_err(||"Failed to send request")?;
	Ok(())
}
