use errors::*;

extern crate crypto;
extern crate serde;


use ops::interface::*;

use self::crypto::digest::Digest;
use self::crypto::whirlpool::Whirlpool;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use serde::de::{Deserialize, Deserializer};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Input {
    version: Option<ResourceVersion>,
    source: ResourceSource,
}

pub fn execute(input: Input) -> Result<()> {
    let mut v: Vec<ResourceVersion> = Vec::new();
    let x = serde_json::to_string(&v)?;
    println!("{}", x);
    Ok(())
}
