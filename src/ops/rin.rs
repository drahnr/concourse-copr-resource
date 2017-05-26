use std::path::PathBuf;

use errors::*;

use ops::interface::*;

#[derive(Serialize, Deserialize)]
pub struct Params {
	x : String,
}

pub fn execute(path_to_srpm : PathBuf, json_params : Params) -> Result<()> {
	unimplemented!();
}
