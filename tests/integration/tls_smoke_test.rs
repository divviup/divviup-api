use rcgen::generate_simple_self_signed;
use test_support::{assert_eq, *};
use tokio::{net::TcpListener, spawn};
use trillium_client::Client;
use trillium_http::Stopper;
use trillium_rustls::{rustls::RootCertStore, RustlsAcceptor, RustlsConfig};
use trillium_tokio::ClientConfig;

#[tokio::test]
async fn https_connection() {
    // Choose aws-lc-rs as the default rustls crypto provider. This is what's currently enabled by
    // the default Cargo feature. Specifying a default provider here prevents runtime errors if
    // another dependency also enables the ring feature.
    let _ = trillium_rustls::rustls::crypto::aws_lc_rs::default_provider().install_default();

    let self_signed = generate_simple_self_signed(["localhost".into()]).unwrap();

    let stopper = Stopper::new();
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    let server_config = trillium_tokio::config()
        .with_acceptor(RustlsAcceptor::from_single_cert(
            self_signed.cert.pem().as_bytes(),
            self_signed.key_pair.serialize_pem().as_bytes(),
        ))
        .without_signals()
        .with_stopper(stopper.clone())
        .with_prebound_server(listener);
    spawn(server_config.run_async(|conn: Conn| async { conn.ok("") }));

    let mut root_store = RootCertStore::empty();
    root_store.add_parsable_certificates([self_signed.cert.der().clone()]);

    let client = Client::new(RustlsConfig::new(
        trillium_rustls::rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
        ClientConfig::default(),
    ));
    let url = format!("https://localhost:{}/", local_addr.port());
    let conn = client.get(url).await.unwrap();
    assert_status!(conn, 200);

    stopper.stop();
}
