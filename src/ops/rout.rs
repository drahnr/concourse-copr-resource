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

use ops::interface::*;

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

use std::fs::walk_dir;
fn find_srpm_regex_match(dir : &String, sprm_regex : &String) -> Option<Path> {

    let re = Regex::new(srpm_regex).chain_err(||"srpm regex failed to parse")?;

    let directory = Path::new(dir);

    for path in walk_dir(&directory) {
        match re.captures(path.display()) {
            Some(x) => {
                println!("{}", path.display());
                return Some(path);
            },
            None => {
            }
        }
    }
    None
}

pub fn execute(dir : String, input : Input) -> Result<()> {

    let params = input.params.unwrap_or_default();
    let chroots = params.chroots.unwrap_or_default();
    let enable_net = params.enable_net.unwrap_or_default();
    let max_n_bytes = params.max_n_bytes.unwrap_or_default();
	let meta = MultipartRequestMetadata { project_id : input.source.project_id,
	                                      chroots : chroots,
	                                      enable_net : enable_net,
	                                     };

    let path_srpm = find_srpm_regex_match(&dir, &input.source.srpm_regex).ok_or("Could not find any matches with that regex")?;

	let attr = metadata(path_srpm.to_str().ok_or("No valid srpm path")?).chain_err(||"Failed to read metadata")?;
	if attr.len() > max_n_bytes {
        bail!("srpm is too damn big, exceeds maximum byte size of {}", max_n_bytes);
	}
	let mut f = File::open(path_to_srpm.to_str().ok_or("No valid srpm path")?).chain_err(||"fun")?;
	let mut reader = BufReader::new(f);

    let name_srpm = path_srpm.file_name().ok_or("No srpm file name")?.to_str().ok_or("Failed to convert to string")?;
    let name_srpm = String::from(path_srpm);
    let mime_srpm : Mime =  "application/x-rpm".parse().unwrap();
    let mime_json : Mime = "application/json".parse().unwrap();

    let meta_json = serde_json::to_string(&meta).chain_err(||"Failed to serialize metadata")?;
	let _response = Multipart::new()
		.add_stream::<_,_,String>("metadata", meta_json.as_bytes(), None, Some(mime_json))
		.add_stream::<_,_,String>("srpm", reader.take(max_n_bytes), Some(name_srpm), Some(mime_srpm))
			.client_request(&Client::new(), input.params.url.trim()).chain_err(||"Failed to send request")?;
	Ok(())
}
