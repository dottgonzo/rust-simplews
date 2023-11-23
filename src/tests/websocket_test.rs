#[cfg(test)]
mod tests {

    #[tokio::test]
    #[cfg(feature = "tls")]

    async fn test_websocket_private_tls() {
        static WEBSOCKET_URI: &str = "wss://localhost:3000";

        let websocket_uri = url::Url::parse(WEBSOCKET_URI).unwrap();

        let (ws_channel_sender, ws_channel_receiver) = crate::create_channel();
        let (_, events_channel_receiver) = crate::create_channel();

        let my_cert_bytes = include_bytes!("nodeserver/ca_cert.pem");

        let insecure_config = crate::Wsconfig {
            insecure: None,
            private_chain_bytes: Some(my_cert_bytes.to_vec()),
        };

        tokio::spawn(crate::start_websocket(
            websocket_uri,
            events_channel_receiver.clone(),
            ws_channel_sender.clone(),
            Some(insecure_config),
        ));

        while let Ok(msg) = ws_channel_receiver.recv().await {
            println!("get msg");

            println!("message_task_parsed: {:?}", msg);
            if msg == "ping" {
                ws_channel_sender.send(msg).await.unwrap();
            }
        }
    }

    #[tokio::test]
    #[cfg(feature = "tls")]
    async fn test_websocket_insecure() {
        static WEBSOCKET_URI: &str = "wss://localhost:3000";

        let websocket_uri = url::Url::parse(WEBSOCKET_URI).unwrap();

        let (ws_channel_sender, ws_channel_receiver) = crate::create_channel();
        let (_, events_channel_receiver) = crate::create_channel();

        let insecure_config = crate::Wsconfig {
            insecure: Some(true),
            private_chain_bytes: None,
        };

        tokio::spawn(crate::start_websocket(
            websocket_uri,
            events_channel_receiver.clone(),
            ws_channel_sender.clone(),
            Some(insecure_config),
        ));

        while let Ok(msg) = ws_channel_receiver.recv().await {
            println!("get msg");

            println!("message_task_parsed: {:?}", msg);
            if msg == "ping" {
                ws_channel_sender.send(msg).await.unwrap();
            }
        }
    }

    #[tokio::test]
    #[cfg(not(feature = "tls"))]
    #[cfg(feature = "no-tls")]
    async fn test_websocket_clear() {
        static WEBSOCKET_URI: &str = "ws://localhost:3000";

        let websocket_uri = url::Url::parse(WEBSOCKET_URI).unwrap();

        let (ws_channel_sender, ws_channel_receiver) = crate::create_channel();
        let (_, events_channel_receiver) = crate::create_channel();

        tokio::spawn(crate::start_websocket(
            websocket_uri,
            events_channel_receiver.clone(),
            ws_channel_sender.clone(),
        ));

        while let Ok(msg) = ws_channel_receiver.recv().await {
            println!("get msg");

            println!("message_task_parsed: {:?}", msg);
            if msg == "ping" {
                ws_channel_sender.send(msg).await.unwrap();
            }
        }
    }
}
