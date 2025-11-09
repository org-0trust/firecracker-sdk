use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use http::{Method, StatusCode};
use regex::Regex;

#[derive(Debug)]
pub enum Http {
    Request {
        path: PathBuf,
        method: Method,
        header_map: HashMap<String, String>,
        body: String,
    },
    Response {
        code: StatusCode,
        header_map: HashMap<String, String>,
        body: String,
    },
}

impl Http {
    pub fn new_request<P: AsRef<Path>>(path: P, method: Method) -> Self {
        Self::Request {
            path: path.as_ref().to_path_buf(),
            method,
            header_map: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn add_header(mut self, k: &str, v: &str) -> Self {
        match self {
            Http::Request {
                ref mut header_map, ..
            } => header_map.insert(k.to_string(), v.to_string()),
            Http::Response {
                ref mut header_map, ..
            } => header_map.insert(k.to_string(), v.to_string()),
        };
        self
    }

    pub fn body(mut self, b: impl ToString) -> Self {
        match self {
            Http::Request { ref mut body, .. } => *body = b.to_string(),
            Http::Response { ref mut body, .. } => *body = b.to_string(),
        };
        self.add_header("Content-Length", &b.to_string().len().to_string())
    }

    pub fn build(self) -> Box<[u8]> {
        match self {
            Http::Request {
                path,
                method,
                header_map,
                body,
            } => {
                let mut req = format!("{} {} HTTP/1.1\r\n", method, path.to_string_lossy());

                for (k, v) in header_map {
                    req.push_str(&format!("{}: {}\r\n", k, v));
                }

                req.push_str("\r\n");
                req.push_str(&body);

                req.as_bytes().into()
            }
            Http::Response { .. } => todo!(),
        }
    }
}

impl From<Vec<u8>> for Http {
    fn from(value: Vec<u8>) -> Self {
        parse_http_response(&value).unwrap()
    }
}

fn parse_http_response(raw: &[u8]) -> Result<Http> {
    println!("{}", String::from_utf8(raw.to_vec()).unwrap());
    let sep = b"\r\n\r\n";
    let split_pos = raw
        .windows(sep.len())
        .position(|w| w == sep)
        .unwrap_or(raw.len().saturating_sub(sep.len()));
    let header_part = &raw[..split_pos + sep.len()];
    let body_part = &raw[split_pos + sep.len()..];

    let header_text =
        str::from_utf8(header_part).map_err(|e| anyhow::anyhow!("headers not utf8: {}", e))?;

    let status_re = Regex::new(r"(?m)^HTTP/(\d+\.\d+)\s+(\d{3})\s+(.*)\r?$")?;
    let status_caps = status_re
        .captures(header_text)
        .ok_or_else(|| anyhow::anyhow!("invalid status line"))?;
    let status_code: u16 = status_caps.get(2).unwrap().as_str().parse()?;

    let header_re = Regex::new(r"(?m)^([^:\r\n]+):\s*(.*)\r?$")?;
    let mut headers = HashMap::new();
    for cap in header_re.captures_iter(header_text) {
        let name = cap.get(1).unwrap().as_str().trim().to_ascii_lowercase();
        let value = cap.get(2).unwrap().as_str().trim().to_string();
        headers
            .entry(name)
            .and_modify(|e: &mut String| {
                e.push_str(", ");
                e.push_str(&value)
            })
            .or_insert(value);
    }

    let body = if let Some(te) = headers.get("transfer-encoding") {
        if te.to_ascii_lowercase().contains("chunked") {
            parse_chunked(body_part)?
        } else {
            body_part.to_vec()
        }
    } else if let Some(cl) = headers.get("content-length") {
        let len: usize = cl
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid content-length"))?;
        let take = std::cmp::min(len, body_part.len());
        body_part[..take].to_vec()
    } else {
        body_part.to_vec()
    };

    Ok(Http::Response {
        code: StatusCode::from_u16(status_code)?,
        header_map: headers,
        body: String::from_utf8(body)?,
    })
}

fn parse_chunked(mut raw: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();

    loop {
        let pos = raw
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| anyhow::anyhow!("chunked: missing chunk-size line ending"))?;
        let size_line = &raw[..pos];
        let size_str = str::from_utf8(size_line)?.trim();
        let size_hex = size_str.split(';').next().unwrap();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| anyhow::anyhow!("invalid chunk size"))?;
        raw = &raw[pos + 2..];
        if size == 0 {
            break;
        }
        if raw.len() < size + 2 {
            bail!("chunked: truncated chunk");
        }
        out.extend_from_slice(&raw[..size]);
        if &raw[size..size + 2] != b"\r\n" {
            bail!("chunked: missing CRLF after chunk");
        }
        raw = &raw[size + 2..];
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\nContent-Type: text/plain\r\n\r\nHello, world!";
        let res = parse_http_response(raw).unwrap();
        if let Http::Response {
            code,
            header_map,
            body,
        } = res
        {
            assert_eq!(code, StatusCode::OK);
            assert_eq!(header_map.get("content-type").unwrap(), "text/plain");
            assert_eq!(body, "Hello, world!");
        }
    }

    #[test]
    fn test_chunked() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nHello\r\n6\r\n, worl\r\n2\r\nd!\r\n0\r\n\r\n";
        let res = parse_http_response(raw).unwrap();
        if let Http::Response { body, .. } = res {
            assert_eq!(body, "Hello, world!")
        }
    }
}
