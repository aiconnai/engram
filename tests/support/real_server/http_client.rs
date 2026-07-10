use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

use serde_json::Value;

use super::config::{start_error, StartError};

pub(crate) fn http_get(port: u16, path: &str) -> Result<String, StartError> {
    http_request(port, "GET", path, None, None)
}

pub(crate) fn http_json_rpc(
    port: u16,
    api_key: Option<&str>,
    request: Value,
) -> Result<Value, StartError> {
    let body = request.to_string();
    let response = http_request(port, "POST", "/mcp", api_key, Some(body.as_bytes()))?;
    let (_, body) = response.split_once("\r\n\r\n").ok_or_else(|| StartError {
        message: format!("HTTP response missing header/body split: {response}"),
    })?;
    serde_json::from_str(body).map_err(start_error)
}

fn http_request(
    port: u16,
    method: &str,
    path: &str,
    api_key: Option<&str>,
    body: Option<&[u8]>,
) -> Result<String, StartError> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(start_error)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(start_error)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(start_error)?;

    let body = body.unwrap_or_default();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
        body.len()
    )
    .map_err(start_error)?;
    if let Some(key) = api_key {
        write!(stream, "Authorization: Bearer {key}\r\n").map_err(start_error)?;
    }
    write!(stream, "\r\n").map_err(start_error)?;
    stream.write_all(body).map_err(start_error)?;
    stream.flush().map_err(start_error)?;

    let mut response = String::new();
    std::io::Read::read_to_string(&mut stream, &mut response).map_err(start_error)?;
    Ok(response)
}
