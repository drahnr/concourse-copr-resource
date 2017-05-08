extern crate hyper;
extern crate multipart;

use self::hyper::Client;

use self::multipart::client::lazy::Multipart;

use std::fs::{File,metadata};
use std::io::prelude::*;
use std::path::Path;
use std::vec::*;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct Params {
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

pub fn execute(path_to_srpm : Path, params : Params) {

	let max_n_bytes : u64 = 250_000_000 ;
	let metadata : MultipartRequestMetadata = params.chroots;

	let attr = metadata(path_to_srpm.to_str())?;
	if attr.len() > max_n_bytes {
		    return Err("srpm is too damn big, exceeds maximum byte size");
	}
	let mut f = File::open(path_to_srpm.to_str())?;
	let mut reader = BufReader::new();

	let _response = Multipart::new()
		.add_stream("metadata", metadata.to_string(), None, "application/json".parse().unwrap())
		.add_stream("srpm", reader.take(max_n_bytes), path_to_srpm.file_name().to_string(), "application/x-rpm".parse().unwrap())
			.client_request(&Client::new(), params.url.to_str())?;
}
