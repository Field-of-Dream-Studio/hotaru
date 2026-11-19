use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;

use hotaru_core::{
    app::server::Server,
    executable::{ProtocolEntryBuilder, def::Endpoint},
    prelude::Arc,
};
use hotaru_http::{
    DefaultHttpTransport, HTTP, context::HttpReqCtx, response::response_templates::normal_response,
    safety::HttpSafety,
};
use hotaru_io_tokio::TcpStream as HotaruTcpStream;
use hotaru_rt_tokio::TokioRuntime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

#[tokio::test]
async fn unsupported_http_version_is_rejected_before_routing() {
    let route_hits = Arc::new(AtomicUsize::new(0));
    let handler_hits = route_hits.clone();

    let app = Server::<DefaultHttpTransport, TokioRuntime>::new()
        .binding("127.0.0.1:0")
        .single_protocol(ProtocolEntryBuilder::new(HTTP::server(
            HttpSafety::default(),
        )))
        .build();

    app.insert(Endpoint::<HTTP>::endpoint(
        "/",
        "root",
        move |_ctx: &mut HttpReqCtx| {
            let handler_hits = handler_hits.clone();
            Box::pin(async move {
                handler_hits.fetch_add(1, Ordering::SeqCst);
                normal_response(200u16, "routed")
            })
        },
    ))
    .expect("root route should bind");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test listener should bind");
    let address = listener.local_addr().unwrap();
    let server = app.clone();
    let accept_task = tokio::spawn(async move {
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.unwrap();
            server.clone().handle_wire(HotaruTcpStream::new(stream));
        }
    });

    let unsupported = exchange(address, b"GET / HTTP/9.9\r\nHost: localhost\r\n\r\n").await;
    assert!(unsupported.starts_with("HTTP/1.1 505 HTTP Version Not Supported\r\n"));
    assert!(
        unsupported
            .to_ascii_lowercase()
            .contains("\r\nconnection: close\r\n")
    );
    assert_eq!(route_hits.load(Ordering::SeqCst), 0);

    let supported = exchange(
        address,
        b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert!(supported.starts_with("HTTP/1.1 200 OK\r\n"));
    assert_eq!(route_hits.load(Ordering::SeqCst), 1);

    accept_task.await.unwrap();
}

async fn exchange(address: core::net::SocketAddr, request: &[u8]) -> String {
    let mut stream = TcpStream::connect(address)
        .await
        .expect("client should connect to test server");
    stream
        .write_all(request)
        .await
        .expect("client should write request");

    let mut response = Vec::new();
    timeout(Duration::from_secs(2), stream.read_to_end(&mut response))
        .await
        .expect("server should close the connection")
        .expect("client should read response");
    String::from_utf8(response).expect("HTTP response should be UTF-8")
}
