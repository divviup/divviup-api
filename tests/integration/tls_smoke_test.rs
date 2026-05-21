use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tokio_rustls::TlsAcceptor;

#[tokio::test]
async fn https_connection() {
    // Choose aws-lc-rs as the default rustls crypto provider. This is what's currently enabled by
    // the default Cargo feature. Specifying a default provider here prevents runtime errors if
    // another dependency also enables the ring feature.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let self_signed = generate_simple_self_signed(["localhost".into()]).unwrap();

    let cert_der = CertificateDer::from(self_signed.cert.der().to_vec());
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        self_signed.signing_key.serialize_der(),
    ));

    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der.clone()], key_der)
        .unwrap();
    server_config.alpn_protocols = vec![b"http/1.1".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

    // Bind to "localhost" so the resolved address matches the loopback IP that
    // reqwest and rcgen use for hostname verification. The hostname in the
    // listener doesn't need to match the cert's SAN — only the hostname passed
    // to reqwest and rcgen matters — but "localhost" ensures we listen on
    // whichever loopback address the OS resolves it to.
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (tcp_stream, _) = listener.accept().await.unwrap();
        let mut tls_stream = tls_acceptor.accept(tcp_stream).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let _ = tls_stream.read(&mut buf).await.unwrap();
        let response = b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
        tls_stream.write_all(response).await.unwrap();
        tls_stream.shutdown().await.unwrap();
    });

    let root_cert = reqwest::tls::Certificate::from_der(cert_der.as_ref()).unwrap();
    let client = reqwest::Client::builder()
        .add_root_certificate(root_cert)
        .http1_only()
        .build()
        .unwrap();

    let resp = client
        .get(format!("https://localhost:{port}/"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
