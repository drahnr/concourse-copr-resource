use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Params {
	x : String,
}

pub fn execute(path_to_srpm : Path, json_params : Params) {
	unimplemented!();
}
