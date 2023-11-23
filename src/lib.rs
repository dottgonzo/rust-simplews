#[cfg(test)]
mod tests;

use std::time::Duration;

use futures_util::sink::SinkExt;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use kanal::{AsyncReceiver, AsyncSender};
// use rustls::client::{ServerCertVerified, ServerCertVerifier};
// use rustls::RootCertStore;
// use rustls_pemfile::certs;
use tokio_tungstenite::WebSocketStream;

use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

#[derive(Clone)]
pub struct Wsconfig {
    pub insecure: Option<bool>,
    pub private_chain_bytes: Option<Vec<u8>>,
}

#[cfg(feature = "tls")]

struct NoVerifier;
#[cfg(feature = "tls")]

impl rustls::client::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
pub async fn initialize_default_websocket_connection(
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

#[cfg(feature = "tls")]
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

    let root_cert_store = rustls::RootCertStore::empty();

    let mut config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(std::sync::Arc::new(NoVerifier));

    let connector = tokio_tungstenite::Connector::Rustls(std::sync::Arc::new(config));

    let (ws_stream, _) =
        tokio_tungstenite::connect_async_tls_with_config(url, None, true, Some(connector)).await?;

    println!("Successfully connected to the WebSocket server.");

    Ok(ws_stream.split())
}

#[cfg(feature = "tls")]
pub async fn initialize_private_tls(
    url: Url,
    private_chain_bytes: &[u8],
) -> anyhow::Result<(
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
)> {
    println!(
        "Connecting to the WebSocket server at {}...",
        &url.to_string()
    );

    let mut cert_cursor = std::io::Cursor::new(private_chain_bytes);
    let cert_chain = rustls_pemfile::certs(&mut cert_cursor)?;

    print!("cert_chain: {:?}", cert_chain.as_slice());

    let mut root_cert_store = rustls::RootCertStore::empty();

    root_cert_store.add_parsable_certificates(cert_chain.as_slice());

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = tokio_tungstenite::Connector::Rustls(std::sync::Arc::new(config));

    // Connect to the web socket
    let url = Url::parse(url.as_str())?;

    let (ws_stream, _) =
        tokio_tungstenite::connect_async_tls_with_config(url, None, true, Some(connector)).await?;

    println!("Successfully connected to the WebSocket server.");

    Ok(ws_stream.split())
}

#[cfg(feature = "tls")]
pub async fn initialize(
    uri: Url,
    ws_config: Option<Wsconfig>,
) -> anyhow::Result<(
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
)> {
    let url = Url::parse(uri.as_str())?;
    if ws_config.clone().is_some() {
        let ws_cfg = ws_config.clone().unwrap();
        if ws_cfg.insecure.is_some() {
            initialize_insecure_tls(url).await
        } else if ws_cfg.private_chain_bytes.is_some() {
            initialize_private_tls(url, &ws_cfg.private_chain_bytes.unwrap()).await
        } else {
            initialize_default_websocket_connection(url).await
        }
    } else {
        if url.scheme() == "ws" {
            println!(
                "Connecting to the OPEN WebSocket server at {}...",
                &url.to_string()
            );
        }

        initialize_default_websocket_connection(url).await
    }
}

#[cfg(not(feature = "tls"))]
pub async fn websocket_handler(
    uri: Url,
    ws_channel_receiver: AsyncReceiver<String>,
    events_channel_sender: AsyncSender<String>,
) -> anyhow::Result<()> {
    let (mut ws_sink, mut ws_stream) = initialize_default_websocket_connection(uri).await?;

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

#[cfg(feature = "tls")]
pub async fn websocket_handler(
    uri: Url,
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

#[cfg(feature = "tls")]
pub async fn start_websocket(
    uri: Url,
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

        println!(
            "restarting websocket routine in {} seconds",
            timeout_in_seconds
        );
        tokio::time::sleep(Duration::from_secs(timeout_in_seconds)).await;
    }
}

#[cfg(not(feature = "tls"))]
pub async fn start_websocket(
    uri: Url,
    ws_channel_receiver: AsyncReceiver<String>,
    events_channel_sender: AsyncSender<String>,
) -> anyhow::Result<()> {
    let timeout_in_seconds = 60;
    println!("start websocket routine");

    loop {
        let t = websocket_handler(
            uri.clone(),
            ws_channel_receiver.clone(),
            events_channel_sender.clone(),
        )
        .await;

        if t.is_err() {
            let msg = format!("websocket error {:?}", t.unwrap_err());
            eprintln!("{}", msg);
        }

        println!(
            "restarting websocket routine in {} seconds",
            timeout_in_seconds
        );
        tokio::time::sleep(Duration::from_secs(timeout_in_seconds)).await;
    }
}
