use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::Value;

use super::config::{start_error, StartError};

const MAX_HTTP_HEADER_BYTES: usize = 16 * 1024;
const MAX_HTTP_LINE_BYTES: usize = 8 * 1024;
const MAX_HTTP_BODY_BYTES: usize = 8 * 1024 * 1024;

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

    read_http_response(stream)
}

fn read_http_response(stream: TcpStream) -> Result<String, StartError> {
    let mut reader = BufReader::new(stream);
    let header_bytes = read_header_bytes(&mut reader)?;
    let header_text = std::str::from_utf8(&header_bytes).map_err(start_error)?;
    let body_bytes = if header_value(header_text, "transfer-encoding")
        .is_some_and(|value| value.to_ascii_lowercase().contains("chunked"))
    {
        read_chunked_body(&mut reader)?
    } else if let Some(length) = header_value(header_text, "content-length") {
        let content_length = length.trim().parse::<usize>().map_err(start_error)?;
        read_content_length_body(&mut reader, content_length)?
    } else {
        read_until_eof_body(&mut reader)?
    };
    let body_text = String::from_utf8(body_bytes).map_err(start_error)?;
    Ok(format!("{header_text}{body_text}"))
}

fn read_header_bytes(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, StartError> {
    let mut headers = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        let read = reader.read(&mut byte).map_err(start_error)?;
        if read == 0 {
            return Err(StartError {
                message: if headers.is_empty() {
                    "HTTP response closed before headers".to_string()
                } else {
                    "HTTP response closed before header terminator".to_string()
                },
            });
        }
        headers.push(byte[0]);
        if headers.len() > MAX_HTTP_HEADER_BYTES {
            return Err(StartError {
                message: format!("HTTP response headers exceeded {MAX_HTTP_HEADER_BYTES} bytes"),
            });
        }
        if headers.ends_with(b"\r\n\r\n") {
            return Ok(headers);
        }
    }
}

fn header_value<'a>(headers: &'a str, name: &str) -> Option<&'a str> {
    headers.lines().skip(1).find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.eq_ignore_ascii_case(name).then_some(value.trim())
    })
}

fn read_content_length_body(
    reader: &mut BufReader<TcpStream>,
    content_length: usize,
) -> Result<Vec<u8>, StartError> {
    if content_length > MAX_HTTP_BODY_BYTES {
        return Err(StartError {
            message: format!("HTTP response body exceeded {MAX_HTTP_BODY_BYTES} bytes"),
        });
    }
    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body).map_err(start_error)?;
    Ok(body)
}

fn read_until_eof_body(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, StartError> {
    let mut body = Vec::new();
    reader
        .take(MAX_HTTP_BODY_BYTES as u64 + 1)
        .read_to_end(&mut body)
        .map_err(start_error)?;
    if body.len() > MAX_HTTP_BODY_BYTES {
        return Err(StartError {
            message: format!("HTTP response body exceeded {MAX_HTTP_BODY_BYTES} bytes"),
        });
    }
    Ok(body)
}

fn read_chunked_body(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, StartError> {
    let mut body = Vec::new();
    loop {
        let chunk_size = read_chunk_size(reader)?;
        if chunk_size == 0 {
            read_trailers(reader)?;
            return Ok(body);
        }
        if body.len().saturating_add(chunk_size) > MAX_HTTP_BODY_BYTES {
            return Err(StartError {
                message: format!("HTTP response body exceeded {MAX_HTTP_BODY_BYTES} bytes"),
            });
        }
        let previous_len = body.len();
        body.resize(previous_len + chunk_size, 0);
        reader
            .read_exact(&mut body[previous_len..])
            .map_err(start_error)?;
        let mut crlf = [0_u8; 2];
        reader.read_exact(&mut crlf).map_err(start_error)?;
        if crlf != *b"\r\n" {
            return Err(StartError {
                message: "HTTP chunk missing trailing CRLF".to_string(),
            });
        }
    }
}

fn read_chunk_size(reader: &mut BufReader<TcpStream>) -> Result<usize, StartError> {
    let line_bytes = read_crlf_line(reader)?;
    let line = std::str::from_utf8(&line_bytes).map_err(start_error)?;
    let size_hex = line
        .trim_end()
        .split_once(';')
        .map_or(line.trim_end(), |(size, _)| size);
    usize::from_str_radix(size_hex, 16).map_err(start_error)
}

fn read_trailers(reader: &mut BufReader<TcpStream>) -> Result<(), StartError> {
    loop {
        let line = read_crlf_line(reader)?;
        if line == b"\r\n" {
            return Ok(());
        }
    }
}

fn read_crlf_line(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, StartError> {
    let mut line = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        let read = reader.read(&mut byte).map_err(start_error)?;
        if read == 0 {
            return Err(StartError {
                message: "HTTP response closed before line terminator".to_string(),
            });
        }
        line.push(byte[0]);
        if line.len() > MAX_HTTP_LINE_BYTES {
            return Err(StartError {
                message: format!("HTTP response line exceeded {MAX_HTTP_LINE_BYTES} bytes"),
            });
        }
        if line.ends_with(b"\r\n") {
            return Ok(line);
        }
    }
}
