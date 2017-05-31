use errors::*;

use walkdir::WalkDir;

extern crate crypto;

use self::crypto::digest::Digest;
use self::crypto::whirlpool::Whirlpool;

extern crate hyper;
extern crate hyper_native_tls;

use self::hyper::Client;
use self::hyper::status::StatusCode;
use self::hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use self::hyper_native_tls::NativeTlsClient;
use self::hyper::client::Request;
use self::hyper::method::Method;
use self::hyper::net::{HttpsConnector, Fresh, Streaming};
use self::hyper::header::{ContentLength, ContentType, Authorization, Basic};
use self::hyper::client::response::Response;

use std::io;

struct MultipartRequest {
    boundary: String,
    request: Request<Fresh>,
    buffer: Vec<u8>,
}

impl MultipartRequest {
    pub fn from_request(request: Request<Fresh>,
                        boundary: Option<&str>)
                        -> Result<MultipartRequest> {
        Ok(MultipartRequest {
               request: request,
               buffer: Vec::new(),
               boundary: boundary.unwrap_or("random").to_string(),
           })
    }

    pub fn add_stream<R>(&mut self,
                         name: &str,
                         stream: &mut R,
                         filename: Option<&str>,
                         content_type: Option<Mime>)
                         -> Result<()>
        where R: Read
    {
        write!(self.buffer, "--{}\r\n", self.boundary)
            .chain_err(|| "Failed to write header")?;
        write!(self.buffer,
               "Content-Disposition: form-data; name=\"{}\"",
               name)
                .chain_err(|| "Failed to write header")?;
        filename.map(|filename| write!(self.buffer, "; filename=\"{}\"", filename));
        content_type.map(|content_type| write!(self.buffer, "\r\nContent-Type: {}", content_type));
        self.buffer
            .write_all(b"\r\n\r\n")
            .chain_err(|| "Failed to write closing line breaks of block header")?;

        io::copy(stream, &mut self.buffer)
            .chain_err(|| "Failed to copy stream content")?;
        self.buffer
            .write_all(b"\r\n\r\n")
            .chain_err(|| "Failed to write closing line breaks of block body")?;

        Ok(())
    }

    pub fn send(mut self, username: String, password: Option<String>) -> Result<Response> {
        write!(self.buffer, "--{}--\r\n", self.boundary)
            .chain_err(|| "Failed to write closing")?;

        {
            let headers = self.request.headers_mut();

            headers.set(ContentType(multipart_mime(self.boundary.as_str())));
            headers.set(ContentLength(self.buffer.len() as u64));
            headers.set(Authorization(Basic { username, password }));
        }

        let mut req: Request<Streaming> = self.request
            .start()
            .chain_err(|| "Failed to write request header")?;
        req.write_all(&self.buffer as &[u8])
            .chain_err(|| "Failed to write request body")?;
        let resp = req.send().chain_err(|| "Failed to send request")?;
        Ok(resp)
    }
}



/// Create a `Content-Type: multipart/form-data;boundary={bound}`
pub fn content_type(bound: &str) -> ContentType {
    ContentType(multipart_mime(bound))
}

fn multipart_mime(bound: &str) -> Mime {
    Mime(TopLevel::Multipart,
         SubLevel::Ext("form-data".into()),
         vec![(Attr::Ext("boundary".into()), Value::Ext(bound.into()))])
}


// let resp : hyper::Request<Streaming> = MultipartRequest::new()?.
// .add_stream(...)?
// .add_stream(...)?
// .send()?;












use std::fs::{File, metadata};
use std::io::prelude::*;
use std::vec::*;
use std::io::BufReader;
use std::io::Write;
use std::path::{PathBuf, Path};
use regex::Regex;

use serde_json;

use ops::interface::*;
use ops::error::ResponseError;

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

    writeln!(&mut ::std::io::stderr(), "base dir: {:?}", dir);

    let re = Regex::new(srpm_regex)
        .chain_err(|| "srpm regex failed to parse")?;

    let directory = Path::new(dir);

    for entry in WalkDir::new(&directory) {
        let entry = entry.chain_err(|| "WalkDir entry is useless")?;
        let path = entry.path();
        if path.is_file() {
            writeln!(&mut ::std::io::stderr(), "Checking path: {:?}", path);
            let path_str = path.to_str().ok_or("Failed to convert path to string")?;
            if re.is_match(path_str) {
                    writeln!(&mut ::std::io::stderr(), "Final pick: {:?}", path);
                    return Ok(Some(PathBuf::from(path_str)));
            }
        } else {
            writeln!(&mut ::std::io::stderr(), "Not a file: {:?}", path);
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

    while true {
        let n = f.read(&mut buffer[..])
            .chain_err(|| "Failed to read file")?;
        if n == 0 {
            digest.result(&mut digest_result);
            break;
        }
        bytes_read += n as u64;
        digest.input(&buffer[0..n]);
    }
    Ok(digest_result)
}


pub fn execute(dir: PathBuf, input: Input) -> Result<()> {

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
    let mut f = File::open(path_srpm_str).chain_err(|| "fun")?;
    let mut reader = BufReader::new(f);

    let name_srpm = path_srpm
        .file_name()
        .ok_or("No srpm file name")?
        .to_str()
        .ok_or("Failed to convert to string")?;
    let name_srpm = String::from(path_srpm_str);
    let mime_srpm: Mime = "application/x-rpm".parse().unwrap();
    let mime_json: Mime = "application/json".parse().unwrap();

    let ssl = NativeTlsClient::new()
        .chain_err(|| "Failed to create native tls client")?;
    let connector = HttpsConnector::new(ssl);

    let mut meta_json = serde_json::to_string(&meta)
        .chain_err(|| "Failed to serialize metadata")?;

    let url = input
        .source
        .url
        .parse()
        .chain_err(|| "Failed to parse url")?;

    let boundary = "stuff";

    let mut request = Request::with_connector(Method::Post, url, &connector)
        .chain_err(|| "Failed to create POST request")?;

    let boundary = "randomarbitrarystuff";

    let mut multipart = MultipartRequest::from_request(request, Some(&boundary))
        .chain_err(|| "Failed to create multipart request")?;

    multipart
        .add_stream("metadata", &mut meta_json.as_bytes(), None, Some(mime_json))
        .chain_err(|| "Failed to prepare multipart metadata")?;
    multipart
        .add_stream("srpm",
                    &mut reader.take(max_n_bytes),
                    Some(name_srpm.as_str()),
                    Some(mime_srpm))
        .chain_err(|| "Failed to prepare multipart srpm")?;

    let response = multipart
        .send(input.source.login, Some(input.source.token))
        .chain_err(|| "Failed to send request")?;

    writeln!(&mut ::std::io::stderr(),
             "Response received: {:?}",
             response);

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

            writeln!(&mut ::std::io::stderr(), "digest: {}", version);

            let version = serde_json::to_string(&version)
                .chain_err(|| "Failed to convert version to json")?;
            println!("{}", version);
            Ok(())
        }
        _ => Err(ResponseError::InvalidRequest.into()),
    }
}
