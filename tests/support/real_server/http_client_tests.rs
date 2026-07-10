use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use super::http_client::http_get;

#[test]
fn http_get_reads_fragmented_content_length_response() {
    let (port, server) = serve_response(vec![
        b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\n".to_vec(),
        b"\r\n{\"ok\":".to_vec(),
        b"true}".to_vec(),
    ]);

    let response = http_get(port, "/health").expect("fragmented response");

    server.join().expect("fragmented server joined");
    assert!(response.ends_with("{\"ok\":true}"), "{response}");
}

#[test]
fn http_get_reads_chunked_response_body() {
    let (port, server) = serve_response(vec![
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec(),
        b"6\r\n{\"ok\":\r\n5\r\ntrue}\r\n0\r\n\r\n".to_vec(),
    ]);

    let response = http_get(port, "/health").expect("chunked response");

    server.join().expect("chunked server joined");
    assert!(response.ends_with("{\"ok\":true}"), "{response}");
}

#[test]
fn http_get_rejects_oversized_header_before_body() {
    let oversized = format!(
        "HTTP/1.1 200 OK\r\nX-Pad: {}\r\n\r\n",
        "a".repeat(16 * 1024)
    );
    let (port, server) = serve_response(vec![oversized.into_bytes()]);

    let error = http_get(port, "/health").expect_err("oversized header should fail");

    server.join().expect("oversized header server joined");
    assert!(
        error.to_string().contains("headers exceeded"),
        "unexpected error: {error}"
    );
}

#[test]
fn http_get_rejects_oversized_chunk_metadata_line() {
    let oversized = format!(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1;{}\r\na\r\n0\r\n\r\n",
        "a".repeat(8 * 1024)
    );
    let (port, server) = serve_response(vec![oversized.into_bytes()]);

    let error = http_get(port, "/health").expect_err("oversized chunk line should fail");

    server.join().expect("oversized chunk server joined");
    assert!(
        error.to_string().contains("line exceeded"),
        "unexpected error: {error}"
    );
}

#[test]
fn http_get_rejects_oversized_content_length_before_body() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 8388609\r\n\r\n".to_vec();
    assert_error_contains(
        vec![response],
        "oversized content-length should fail",
        "body exceeded",
    );
}

#[test]
fn http_get_rejects_malformed_chunk_trailing_crlf() {
    assert_error_contains(
        vec![b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1\r\naXX".to_vec()],
        "malformed chunk trailing CRLF should fail",
        "chunk missing trailing CRLF",
    );
}

#[test]
fn http_get_rejects_malformed_chunk_terminator() {
    assert_error_contains(
        vec![b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX".to_vec()],
        "malformed chunk terminator should fail",
        "line terminator",
    );
}

fn assert_error_contains(chunks: Vec<Vec<u8>>, context: &str, expected: &str) {
    let (port, server) = serve_response(chunks);

    let error = http_get(port, "/health").expect_err(context);

    server.join().expect("error response server joined");
    assert!(
        error.to_string().contains(expected),
        "expected {expected:?} in error: {error}"
    );
}

fn serve_response(chunks: Vec<Vec<u8>>) -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let port = listener.local_addr().expect("listener address").port();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept test request");
        read_request_headers(&mut stream);
        for chunk in chunks {
            stream.write_all(&chunk).expect("write response chunk");
        }
        stream.flush().expect("flush response");
    });
    (port, server)
}

fn read_request_headers(stream: &mut TcpStream) {
    let mut request = Vec::new();
    let mut byte = [0_u8; 1];
    while !request.ends_with(b"\r\n\r\n") {
        let read = stream.read(&mut byte).expect("read test request");
        assert_ne!(read, 0, "client closed before request headers ended");
        request.push(byte[0]);
    }
}
