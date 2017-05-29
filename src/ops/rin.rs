use std::path::PathBuf;

use errors::*;

use ops::interface::*;

#[derive(Serialize, Deserialize)]
pub struct Input {
    x: String,
}

pub fn execute(dir: PathBuf, json_params: Input) -> Result<()> {
    unimplemented!();
}
