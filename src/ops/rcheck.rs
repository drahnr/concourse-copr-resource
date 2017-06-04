use errors::*;

use ops::interface::*;

use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Input {
    version: Option<ResourceVersion>,
    source: ResourceSource,
}

pub fn execute(_input: Input) -> Result<()> {
    let v: Vec<ResourceVersion> = Vec::new();
    let x = serde_json::to_string(&v)?;
    println!("{}", x);
    Ok(())
}
