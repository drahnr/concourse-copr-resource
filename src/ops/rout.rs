use errors::*;

use walkdir::WalkDir;

use std::fs::{File, metadata};
use std::vec::*;
use std::io::BufReader;
use std::io::{Write, Read};
use std::path::{PathBuf, Path};
use regex::Regex;

use serde_json;

use crypto::digest::Digest;
use crypto::whirlpool::Whirlpool;

use ops::multipart::*;
use ops::interface::*;
use ops::error::ResponseError;

use hyper::status::StatusCode;
use hyper::mime::Mime;
use hyper_native_tls::NativeTlsClient;
use hyper::client::Request;
use hyper::method::Method;
use hyper::net::HttpsConnector;

#[derive(Serialize, Deserialize)]
pub struct Input {
    source: ResourceSource,
    params: Option<ResourceParams>,
}

#[derive(Serialize, Deserialize)]
pub struct MultipartRequestMetadata {
    project_id: u32,
    chroots: Vec<String>,
    enable_net: bool,
}


fn find_srpm_regex_match(dir: &PathBuf, srpm_regex: &String) -> Result<Option<PathBuf>> {

    writeln!(&mut ::std::io::stderr(), "base dir: {:?}", dir)
        .chain_err(|| "")?;

    let re = Regex::new(srpm_regex)
        .chain_err(|| "srpm regex failed to parse")?;

    let directory = Path::new(dir);

    for entry in WalkDir::new(&directory) {
        let entry = entry.chain_err(|| "WalkDir entry is useless")?;
        let path = entry.path();
        if path.is_file() {
            let path_str = path.to_str().ok_or("Failed to convert path to string")?;
            if re.is_match(path_str) {
                writeln!(&mut ::std::io::stderr(), "Final pick: {:?}", path)
                    .chain_err(|| "")?;
                return Ok(Some(PathBuf::from(path_str)));
            }
        }
    }
    Ok(None)
}

fn calculate_whirlpool(path: &PathBuf) -> Result<[u8; 64]> {
    let mut digest = Whirlpool::new();
    let mut digest_result = [0u8; 64];

    let path = path.to_str().ok_or("Failed to convert path to string")?;
    let mut f = File::open(path).chain_err(|| format!("Failed to open"))?;
    let mut buffer = [0u8; 16384];
    let mut bytes_read: u64 = 0;

    loop {
        let n = f.read(&mut buffer[..])
            .chain_err(|| "Failed to read file")?;
        if n == 0 {
            digest.result(&mut digest_result);
            break;
        }
        bytes_read += n as u64;
        digest.input(&buffer[0..n]);
    }

    writeln!(&mut ::std::io::stderr(),
             "Digest calculated over {:?} bytes",
             bytes_read)
            .chain_err(|| "Failed to write out bytes_read")?;

    Ok(digest_result)
}


pub fn execute(mut dir: PathBuf, input: Input) -> Result<()> {

    let params = input.params.ok_or("We need some parameters")?;

    let regex = params.regex.unwrap_or_default();
    let project_id = params.project_id.ok_or("Missing project_id parameter")?;

    let chroots = params.chroots.unwrap_or_default();
    let enable_net = params.enable_net.unwrap_or_default();
    let max_n_bytes = params.max_n_bytes.unwrap_or_default();
    let meta = MultipartRequestMetadata {
        project_id: project_id,
        chroots: chroots,
        enable_net: enable_net,
    };
    dir.push(params.rpmbuild_dir);

    let path_srpm = find_srpm_regex_match(&dir, &regex)
        .chain_err(|| "Could not find any matches with that regex")?;
    let path_srpm = path_srpm
        .ok_or(format!("No path found matching regex \"{}\"", regex))?;

    let path_srpm_str = path_srpm.to_str().ok_or("No valid srpm path")?;
    let attr = metadata(path_srpm_str)
        .chain_err(|| "Failed to read metadata")?;
    let size_srpm = attr.len() as u64;
    if size_srpm > max_n_bytes {
        bail!("srpm is too damn big, {} exceeds maximum byte size of {}",
              size_srpm,
              max_n_bytes);
    }
    let f = File::open(path_srpm_str).chain_err(|| "fun")?;
    let reader = BufReader::new(f);

    let name_srpm = path_srpm
        .file_name()
        .ok_or("No srpm file name")?
        .to_str()
        .ok_or("Failed to convert to string")?;
    let mime_srpm: Mime = "application/x-rpm".parse().unwrap();
    let mime_json: Mime = "application/json".parse().unwrap();

    let ssl = NativeTlsClient::new()
        .chain_err(|| "Failed to create native tls client")?;
    let connector = HttpsConnector::new(ssl);

    let meta_json = serde_json::to_string(&meta)
        .chain_err(|| "Failed to serialize metadata")?;

    let url = input
        .source
        .url
        .parse()
        .chain_err(|| "Failed to parse url")?;

    let request = Request::with_connector(Method::Post, url, &connector)
        .chain_err(|| "Failed to create POST request")?;

    let boundary = "randomarbitrarystuffwhichisprettysureorthogonalslashunique";

    let mut multipart = MultipartRequest::from_request(request, Some(&boundary))
        .chain_err(|| "Failed to create multipart request")?;

    multipart
        .add_stream("metadata", &mut meta_json.as_bytes(), None, Some(mime_json))
        .chain_err(|| "Failed to prepare multipart metadata")?;
    multipart
        .add_stream("srpm",
                    &mut reader.take(max_n_bytes),
                    Some(name_srpm),
                    Some(mime_srpm))
        .chain_err(|| "Failed to prepare multipart srpm")?;

    let response = multipart
        .send(input.source.login, Some(input.source.token))
        .chain_err(|| "Failed to send request")?;

    writeln!(&mut ::std::io::stderr(),
             "Response received: {:?}",
             response)
            .chain_err(|| "Failed to write out response received")?;

    match response.status {
        StatusCode::BadRequest => Err(ResponseError::InvalidRequest.into()),
        StatusCode::Forbidden => Err(ResponseError::AuthentificationFailure.into()),
        StatusCode::Created => {
            let digest = calculate_whirlpool(&path_srpm)
                .chain_err(|| "Failed to calculate digest")?;
            // TODO implement serialize for 64 bits instead of cropping
            let mut snip = [0u8; 32];
            snip.copy_from_slice(&digest[0..32]);
            let version = ResourceVersion { digest: snip };

            writeln!(&mut ::std::io::stderr(), "digest: {}", version)
                .chain_err(|| "Failed to write out version")?;

            let version = serde_json::to_string(&version)
                .chain_err(|| "Failed to convert version to json")?;
            println!("{}", version);
            Ok(())
        }
        _ => Err(ResponseError::InvalidRequest.into()),
    }
}
