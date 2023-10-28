
use std::sync::Arc;
use std::time::Duration;


use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use kanal::{AsyncReceiver, AsyncSender};
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::RootCertStore;
use tokio_tungstenite::{connect_async_tls_with_config, Connector};

use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

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

pub async fn websocket_handler(
    uri: &String,
    ws_channel_receiver: AsyncReceiver<String>,
    events_channel_sender: AsyncSender<String>,
) -> anyhow::Result<()> {

    println!(
        "Connecting to the WebSocket server at {}...",
        uri
    );

    let mut root_cert_store = RootCertStore::empty();
    // let rust_cert = rustls::Certificate(include_bytes!("pina.movia.biz.pem").to_vec());
    // root_cert_store.add(&rust_cert)?;

    // let mut tcp_stream = std::net::TcpStream::connect(&dev_config.websocket_url).unwrap();
    let mut config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoVerifier));

    let connector = Connector::Rustls(Arc::new(config));

    let full_url = uri.as_str();

    // Connect to the web socket
    let url = Url::parse(&full_url)?;

    let (ws_stream, _) = connect_async_tls_with_config(url, None, true, Some(connector)).await?;

    println!("Successfully connected to the WebSocket server.");
    let (mut ws_sink, mut ws_stream) = ws_stream.split();

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
                    println!("Received message: {}", text);

                    events_channel_sender.send(text).await?;

                    // parse text message into MessageTaskJsonRX
                    // let message_task_parsed =
                    //     serde_json::from_str::<MessageTaskJsonRX>(text.as_str());
                    // if let Ok(message_task_parsed) = message_task_parsed {
                    //     let the_answer = server_msg_dispatch(&message_task_parsed).await?;
                    //     let answer_json = serde_json::to_string(&the_answer)?;
                    //     events_channel_sender.send(answer_json).await?;
                    // } else {
                    //     eprintln!(
                    //         "Error parsing message from websocket: {}",
                    //         message_task_parsed.unwrap_err()
                    //     );
                    // }
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

pub async fn start_websocket(
    uri: &String,
    ws_channel_receiver: AsyncReceiver<String>,
    events_channel_sender: AsyncSender<String>
) -> anyhow::Result<()> {
    let timeout_in_seconds = 60;
    println!("start websocket routine");

    loop {
        let t = websocket_handler(uri,ws_channel_receiver.clone(), events_channel_sender.clone()).await;

        if t.is_err() {
            let msg = format!("websocket error {:?}", t.unwrap_err());
            eprintln!("{}", msg);
        }
        println!("websocket routine ended");

        tokio::time::sleep(Duration::from_secs(timeout_in_seconds)).await;
    }
}
