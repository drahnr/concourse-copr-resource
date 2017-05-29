use errors::*;


use walkdir::WalkDir;

extern crate crypto;

use self::crypto::digest::Digest;
use self::crypto::whirlpool::Whirlpool;

extern crate hyper;
extern crate hyper_native_tls;
extern crate multipart;

use self::hyper::Client;
use self::hyper::status::StatusCode;
use self::hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use self::hyper::net::HttpsConnector;
use self::hyper_native_tls::NativeTlsClient;
use self::hyper::client::Request;
use self::hyper::method::Method;
use self::hyper::net::Streaming;
use self::hyper::header::{ContentLength,ContentType};
use self::multipart::client::Multipart;

use std::fs::{File,metadata};
use std::io::prelude::*;
use std::vec::*;
use std::io::BufReader;
use std::io::Write;
use std::path::{PathBuf,Path};
use regex::Regex;

use serde_json;

use ops::interface::*;
use ops::error::ResponseError;

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
	let x = entry.path().to_str().ok_or("Failed to convert path to string")?;
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

    let params = input.params.ok_or("We need some parameters")?;

    let regex = params.regex.unwrap_or_default();
    let project_id = params.project_id.ok_or("Missing project_id parameter")?;

    let chroots = params.chroots.unwrap_or_default();
    let enable_net = params.enable_net.unwrap_or_default();
    let max_n_bytes = params.max_n_bytes.unwrap_or_default();
	let meta = MultipartRequestMetadata { project_id : project_id,
	                                      chroots : chroots,
	                                      enable_net : enable_net,
	                                     };

	let path_srpm = find_srpm_regex_match(&dir, &regex).chain_err(||"Could not find any matches with that regex")?;
	let path_srpm = path_srpm.ok_or(format!("No path found matching regex \"{}\"",regex))?;

	let path_srpm_str = path_srpm.to_str().ok_or("No valid srpm path")?;
	let attr = metadata(path_srpm_str).chain_err(||"Failed to read metadata")?;
	let size_srpm = attr.len() as u64;
	if size_srpm > max_n_bytes {
        bail!("srpm is too damn big, {} exceeds maximum byte size of {}", size_srpm, max_n_bytes);
	}
	let mut f = File::open(path_srpm_str).chain_err(||"fun")?;
	let mut reader = BufReader::new(f);

    let name_srpm = path_srpm.file_name().ok_or("No srpm file name")?.to_str().ok_or("Failed to convert to string")?;
    let name_srpm = String::from(path_srpm_str);
    let mime_srpm : Mime =  "application/x-rpm".parse().unwrap();
    let mime_json : Mime = "application/json".parse().unwrap();

    let ssl = NativeTlsClient::new().chain_err(||"Failed to create native tls client")?;
    let connector = HttpsConnector::new(ssl);

    let mut meta_json = serde_json::to_string(&meta).chain_err(||"Failed to serialize metadata")?;

	let url = input.source.url.parse().chain_err(||"Failed to parse url")?;

	let boundary = "stuff";

    let mut request = Request::with_connector(Method::Post, url, &connector)
   					.chain_err(||"Failed to create POST request")?;
    request.headers_mut().set(
			ContentType(
		 		Mime(
		 			TopLevel::Multipart, SubLevel::Ext("form-data".into()),
		 			vec![(Attr::Ext("boundary".into()), Value::Ext(boundary.into()))]
		 			)
		 		)
		 	);
	let total_len : u64 = meta_json.len() as u64 + size_srpm as u64;
    request.headers_mut().set(ContentLength(total_len));
	println!("total len {}", total_len);

    let mut multipart = Multipart::from_request(request).chain_err(||"Failed to create multipart request")?;

	multipart.write_stream::<_,_>("metadata", &mut meta_json.as_bytes(), None, Some(mime_json)).chain_err(||"Failed to prepare multipart metadata")?;
	multipart.write_stream::<_,_>("srpm", &mut reader.take(max_n_bytes), Some(name_srpm.as_str()), Some(mime_srpm)).chain_err(||"Failed to prepare multipart srpm")?;

	let response = multipart.send().chain_err(||"Failed to send request")?;

	writeln!(&mut ::std::io::stderr(), "Response received: {:?}", response);

	match response.status {
		StatusCode::BadRequest => { Err(ResponseError::InvalidRequest.into()) },
		StatusCode::Forbidden => { Err(ResponseError::AuthentificationFailure.into()) },
		StatusCode::Ok => { Ok(()) },
		_ => { Err(ResponseError::InvalidRequest.into()) },
	}
}
