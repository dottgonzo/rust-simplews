#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;

use futures_util::sink::SinkExt;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use kanal::{AsyncReceiver, AsyncSender};
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::RootCertStore;
use rustls_pemfile::certs;
use tokio_tungstenite::{connect_async_tls_with_config, Connector, WebSocketStream};

use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

#[derive(Debug, Clone)]
pub struct PrivateCertChain {
    pub ca: Option<String>,
    pub cert: String,
    pub key: String,
}

#[derive(Clone)]
pub struct Wsconfig {
    pub insecure: bool,
    pub private_chain_file_path: Option<String>,
}

struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}
pub async fn initialize_default_tls(
    url: Url,
) -> anyhow::Result<(
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
)> {
    println!(
        "Connecting to the WebSocket server at {}...",
        &url.to_string()
    );

    let (ws_stream, _) = tokio_tungstenite::connect_async(&url.to_string()).await?;
    println!("Successfully connected to the WebSocket server.");

    Ok(ws_stream.split())
}

pub async fn initialize_insecure_tls(
    url: Url,
) -> anyhow::Result<(
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
)> {
    println!(
        "Connecting to the WebSocket server at {}...",
        &url.to_string()
    );

    let root_cert_store = RootCertStore::empty();

    let mut config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoVerifier));

    let connector = Connector::Rustls(Arc::new(config));

    let (ws_stream, _) = connect_async_tls_with_config(url, None, true, Some(connector)).await?;

    println!("Successfully connected to the WebSocket server.");

    Ok(ws_stream.split())
}

pub async fn initialize_private_tls(
    url: Url,
    private_chain_file_path: String,
) -> anyhow::Result<(
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
)> {
    println!(
        "Connecting to the WebSocket server at {}...",
        &url.to_string()
    );

    // Leggi il certificato del client
    let cert_file = File::open(private_chain_file_path).expect("Unable to open client_cert.pem");
    let mut cert_buf = BufReader::new(cert_file);
    let cert_chain = certs(&mut cert_buf).expect("Failed to read client_cert.pem");

    //     // Leggi la chiave privata del client
    //     let key_file = File::open("client_key.pem").expect("Unable to open client_key.pem");
    //     let mut key_buf = BufReader::new(key_file);
    //     let mut keys = rsa_private_keys(&mut key_buf).expect("Failed to read client_key.pem");

    //     let ca_cert_file = File::open("path_to_your_ca_cert.pem").expect("Unable to open CA cert");
    // let mut ca_cert_buf = BufReader::new(ca_cert_file);

    let mut root_cert_store = RootCertStore::empty();
    // let rust_cert = rustls::Certificate(include_bytes!("pina.movia.biz.pem").to_vec());

    root_cert_store.add_parsable_certificates(cert_chain.as_slice());

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = Connector::Rustls(Arc::new(config));

    // Connect to the web socket
    let url = Url::parse(url.as_str())?;

    let (ws_stream, _) = connect_async_tls_with_config(url, None, true, Some(connector)).await?;

    println!("Successfully connected to the WebSocket server.");

    Ok(ws_stream.split())
}

pub async fn initialize(
    uri: String,
    ws_config: Option<Wsconfig>,
) -> anyhow::Result<(
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
)> {
    let url = Url::parse(uri.as_str())?;
    if ws_config.clone().is_some() {
        let ws_cfg = ws_config.clone().unwrap();
        if ws_cfg.insecure {
            initialize_insecure_tls(url).await
        } else if ws_cfg.private_chain_file_path.is_some() {
            initialize_private_tls(url, ws_cfg.private_chain_file_path.unwrap()).await
        } else {
            initialize_default_tls(url).await
        }
    } else if url.scheme() == "ws" {
        initialize_insecure_tls(url).await
    } else {
        initialize_default_tls(url).await
    }
}
pub async fn websocket_handler(
    uri: String,
    ws_channel_receiver: AsyncReceiver<String>,
    events_channel_sender: AsyncSender<String>,
    ws_config: Option<Wsconfig>,
) -> anyhow::Result<()> {
    let (mut ws_sink, mut ws_stream) = initialize(uri, ws_config).await?;

    let tx_loop = tokio::spawn(async move {
        while let Ok(msg) = ws_channel_receiver.recv().await {
            ws_sink.send(Message::Text(msg)).await?;
        }
        Ok::<(), anyhow::Error>(())
    });

    let rx_loop = tokio::spawn(async move {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    events_channel_sender.send(text).await?;
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow::anyhow!("Error receiving message: {}", e));
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    });

    _ = tokio::try_join!(tx_loop, rx_loop)?;
    Err(anyhow::anyhow!("WebSocket handler exited!"))
}

pub fn create_channel() -> (AsyncSender<String>, AsyncReceiver<String>) {
    let (ws_channel_sender, ws_channel_receiver) = kanal::unbounded_async();
    (ws_channel_sender, ws_channel_receiver)
}

pub async fn start_websocket(
    uri: String,
    ws_channel_receiver: AsyncReceiver<String>,
    events_channel_sender: AsyncSender<String>,
    ws_config: Option<Wsconfig>,
) -> anyhow::Result<()> {
    let timeout_in_seconds = 60;
    println!("start websocket routine");

    loop {
        let t = websocket_handler(
            uri.clone(),
            ws_channel_receiver.clone(),
            events_channel_sender.clone(),
            ws_config.clone(),
        )
        .await;

        if t.is_err() {
            let msg = format!("websocket error {:?}", t.unwrap_err());
            eprintln!("{}", msg);
        }
        println!("websocket routine ended");

        tokio::time::sleep(Duration::from_secs(timeout_in_seconds)).await;
    }
}
