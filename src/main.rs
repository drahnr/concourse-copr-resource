// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate serde;
extern crate serde_json;

extern crate regex;

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
#[macro_use]
extern crate error_chain;

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}


// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
use errors::*;

use regex::Regex;

#[macro_use]
extern crate serde_derive;

use serde_json::Error;

use std::io::prelude::*;
use std::path::PathBuf;
use std::io::BufReader;

pub mod ops;

fn dispatch(args : &mut std::env::Args) -> Result<()> {

	let name = args.nth(0).ok_or("Who am I?")?;

	// read params from stdin
	let stdin = std::io::stdin();
	let mut handle = stdin.lock();
	let handle = BufReader::new(handle);

	let re = Regex::new(r"^(?:(?:(?:.*/)?opt/)?/resource/)?([^/]+)$").chain_err(||"Regex is shit")?;
	match re.captures(name.as_ref()) {
		Some(caps) => {
		    let x = caps.get(1).ok_or("Failed get furst capture")?;
			match x.as_str() {
				"check" => {
					let params : ops::concourse_check::Params = serde_json::from_reader(handle).chain_err(|| "Failed to parse json")?;
					ops::concourse_check::execute(params)?;
				},
				"in" => {
					let path : String = args.nth(1).ok_or("Missing argument")?;
					let path = PathBuf::from(path);
					let params : ops::concourse_in::Params = serde_json::from_reader(handle).chain_err(|| "Failed to parse json")?;
					ops::concourse_in::execute(path, params)?;
				},
				"out" => {
					let path : String = args.nth(1).ok_or("Missing argument")?;
					let path = PathBuf::from(path);
					let params : ops::concourse_out::Params = serde_json::from_reader(handle).chain_err(|| "Failed to parse json")?;
					ops::concourse_out::execute(path, params)?;
				},
				x => bail!("The file has to be named as either check/in/out but was {}", x),
			};
		},
		None => bail!("File not in correct path"),
	}
	Ok(())
}


fn main() {
	let mut args = std::env::args();
	if let Err(e) = dispatch(&mut args) {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }
        ::std::process::exit(1);
	}
}
