use std::path::PathBuf;

use errors::*;

use ops::interface::*;

use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Input {}

pub fn execute(_dir: PathBuf, _json_params: Input) -> Result<()> {
    let version = ResourceVersion { digest: [0;32] };
    let version = serde_json::to_string(&version)
        .chain_err(|| "Failed to convert version to json")?;
    println!("{}", version);
    Ok(())
}
