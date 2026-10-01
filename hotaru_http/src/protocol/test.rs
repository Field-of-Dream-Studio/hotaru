use std::sync::Arc;

use hotaru_core::{
    app::common::RuntimeConfig,
    connection::{ConnStream, HotaruRead, HotaruWrite},
    protocol::{Protocol, ProtocolFlow},
    url::UrlRoot,
};
use hotaru_io_tokio::TcpStream as HotaruTcpStream;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::{Duration, timeout},
};

use super::HTTP;
use crate::security::safety::HttpSafety;

struct ExchangeResult {
    response: Vec<u8>,
    flow: ProtocolFlow,
}

async fn exchange(raw_request: &[u8]) -> ExchangeResult {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let stream = HotaruTcpStream::new(stream);
        let (read, write, meta) = stream.split();
        let protocol = HTTP::server(HttpSafety::default());
        let channel = protocol.open_channel(read.into_buf(), write.into_buf_write(), meta);

        <HTTP as Protocol>::handle(
            &channel,
            Arc::new(RuntimeConfig::new()),
            Arc::new(UrlRoot::new()),
        )
        .await
        .unwrap()
    });

    let mut client = TcpStream::connect(address).await.unwrap();
    client.write_all(raw_request).await.unwrap();
    client.shutdown().await.unwrap();

    let mut response = Vec::new();
    timeout(Duration::from_secs(2), client.read_to_end(&mut response))
        .await
        .expect("server did not close the connection")
        .unwrap();

    let flow = timeout(Duration::from_secs(2), server)
        .await
        .expect("server task did not finish")
        .unwrap();

    ExchangeResult { response, flow }
}

fn assert_rejected(response: &[u8], expected_status: &str) {
    let response = String::from_utf8_lossy(response);
    assert!(
        response.starts_with(expected_status),
        "response was: {response}"
    );
    assert!(response.contains("connection: close\r\n"));
    assert!(!response.contains("200 OK"));
    assert!(!response.contains("404 Not Found"));
}

#[tokio::test]
async fn unknown_http_version_returns_505_and_closes() {
    let result = exchange(b"GET / HTTP/9.9\r\nHost: localhost\r\n\r\n").await;

    assert!(matches!(result.flow, ProtocolFlow::Close));
    assert_rejected(
        &result.response,
        "HTTP/1.1 505 HTTP Version Not Supported\r\n",
    );
}

#[tokio::test]
async fn recognised_non_http1_versions_return_505() {
    for version in ["HTTP/0.9", "HTTP/2.0", "HTTP/3.0"] {
        let request = format!("GET / {version}\r\nHost: localhost\r\n\r\n");
        let result = exchange(request.as_bytes()).await;

        assert!(matches!(result.flow, ProtocolFlow::Close));
        assert_rejected(
            &result.response,
            "HTTP/1.1 505 HTTP Version Not Supported\r\n",
        );
    }
}

#[tokio::test]
async fn malformed_http_version_returns_400_and_closes() {
    let result = exchange(b"GET / HTTPX\r\nHost: localhost\r\n\r\n").await;

    assert!(matches!(result.flow, ProtocolFlow::Close));
    assert_rejected(&result.response, "HTTP/1.1 400 Bad Request\r\n");
}

#[tokio::test]
async fn request_after_unsupported_version_is_not_processed() {
    let result = exchange(
        b"GET / HTTP/9.9\r\nHost: localhost\r\n\r\n\
          GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;

    assert!(matches!(result.flow, ProtocolFlow::Close));
    assert_rejected(
        &result.response,
        "HTTP/1.1 505 HTTP Version Not Supported\r\n",
    );
    assert_eq!(
        String::from_utf8_lossy(&result.response)
            .matches("HTTP/1.1 ")
            .count(),
        1
    );
}
