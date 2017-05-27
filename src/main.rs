// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]


pub mod ops;

extern crate serde;

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate walkdir;

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
    use serde_json::error::Error as SerdeError;
    use ops::error::{MiscError, ResponseError};
    use walkdir::Error as WalkDirError;
    error_chain! {
        foreign_links {
            Json(SerdeError);
            Response(ResponseError);
            Misc(MiscError);
            WalkDir(WalkDirError);
        }
    }
}


// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
use errors::*;

use regex::Regex;


use serde_json::Error;

use std::io::prelude::*;
use std::path::PathBuf;
use std::io::BufReader;


fn dispatch(args : &mut std::env::Args) -> Result<()> {

	let name = args.nth(0).ok_or("Who am I?")?;

	// read params from stdin
	let stdin = std::io::stdin();
	let mut handle = stdin.lock();
	let handle = BufReader::new(handle);

	let re = Regex::new(r"^(?:(?:\./|/opt/)(?:resource/)?)?([^/]+)$").chain_err(||"Regex is shit")?;
	match re.captures(name.as_ref()) {
		Some(caps) => {
		    let x = caps.get(1).ok_or("Failed to get first capture")?;
			match x.as_str() {
				"check" => {
					let input : ops::rcheck::Input = serde_json::from_reader(handle).chain_err(|| "[check] Failed to parse json")?;
					// let params : ops::rcheck::Output =
					ops::rcheck::execute(input)?;
				},
				"in" => {
					let path : String = args.next().ok_or("[in] Missing commandline argument")?;
					let path = PathBuf::from(path);
					let params : ops::rin::Input = serde_json::from_reader(handle).chain_err(|| "[in] Failed to parse json")?;
					ops::rin::execute(path, params)?;
				},
				"out" => {
					let path : String = args.next().ok_or("[out] Missing commandline argument")?;
					let path = PathBuf::from(path);
					let params : ops::rout::Input = serde_json::from_reader(handle).chain_err(|| "[out] Failed to parse json")?;
					ops::rout::execute(path, params)?;
				},
				x => bail!("The file has to be named as either check/in/out but was {}", x),
			};
		},
		None => bail!("Resource binary not in correct path {}", name),
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
