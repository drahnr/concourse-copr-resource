use errors::*;

use std::io::{Write, Read};

use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper::client::Request;
use hyper::net::{Fresh, Streaming};
use hyper::header::{ContentLength, ContentType, Authorization, Basic};
use hyper::client::response::Response;

use std::io;

pub struct MultipartRequest {
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

pub fn multipart_mime(bound: &str) -> Mime {
    Mime(TopLevel::Multipart,
         SubLevel::Ext("form-data".into()),
         vec![(Attr::Ext("boundary".into()), Value::Ext(bound.into()))])
}


// let resp : hyper::Request<Streaming> = MultipartRequest::new()?.
// .add_stream(...)?
// .add_stream(...)?
// .send()?;
