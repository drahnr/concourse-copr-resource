use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use errors::*;


#[derive(Serialize, Deserialize)]
pub struct Params {
	x : String,
}

pub fn execute(path_to_srpm : PathBuf, json_params : Params) -> Result<()> {
	unimplemented!();
}
