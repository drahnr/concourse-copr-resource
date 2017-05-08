extern crate serde;
extern crate serde_json;

extern crate regex;

use regex::Regex;

#[macro_use]
extern crate serde_derive;

use serde_json::Error;


use std::io::prelude::*;
use std::path::Path;
use std::io::BufReader;

pub mod ops;


fn dispatch(args : &mut std::env::Args) -> Result<(),String> {

	let name = args.nth(0)?;

	// read params from stdin
	let stdin = std::io::stdin();
	let mut handle = stdin.lock();
	let handle = BufReader::new(handle);

	let re = Regex::new(r"^(?:(?:(?:.*/)?opt/)?/resource/)?([^/]+)$")?;
	match re.captures(name.as_ref()) {
		Some(caps) => {
			match caps.get(1).map_or("XXXX", |m| m.as_str()) {
				"check" => {
					let params : ops::concourse_check::Params = serde_json::from_str(handle)?;
					ops::concourse_out::execute(params)?;
				},
				"in" => {
					let path : Path = args.nth(1)?.parse()?;
					let params : ops::concourse_in::Params = serde_json::from_reader(handle)?;
					ops::concourse_out::execute(path, params)?;
				},
				"out" => {
					let path : Path = args.nth(1)?.parse()?;
					let params : ops::concourse_out::Params = serde_json::from_reader(handle)?;
					ops::concourse_out::execute(path, params)?;
				},
				x => Err(format!("Unknown error {}", x)),
			};
		},
		_ => return Err(format!("no valid executable name provided {}", x)),
	}
	Ok(())
}



fn main() {
	let mut args = std::env::args();
	match dispatch(&mut args) {
		Ok(_) => {},
		Err(x) => { panic!("shit blew up {}", x)},
	}
}
