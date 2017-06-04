use std::path::PathBuf;

use errors::*;

#[derive(Serialize, Deserialize)]
pub struct Input {}

pub fn execute(_dir: PathBuf, _json_params: Input) -> Result<()> {
    Ok(())
}
