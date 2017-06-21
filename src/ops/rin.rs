use std::path::PathBuf;

use errors::*;

use ops::interface::*;

use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Input {}

#[derive(Serialize, Deserialize)]
pub struct Output {
    pub version: ResourceVersion,
    // meta
}

pub fn execute(_dir: PathBuf, _json_params: Input) -> Result<()> {
    let version = ResourceVersion { digest: [0;32] };
    let output = Output { version : version};
    let output = serde_json::to_string(&output)
        .chain_err(|| "Failed to convert version to json")?;

    println!("{}", output);
    Ok(())
}
