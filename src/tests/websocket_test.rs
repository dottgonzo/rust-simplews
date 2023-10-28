#[cfg(test)]
mod tests {

    static WEBSOCKET_URI: &str = "wss://localhost:3000";

    #[tokio::test]
    async fn test_websocket_private_tls() {
        let websocket_uri = WEBSOCKET_URI.to_string();

        let (ws_channel_sender, ws_channel_receiver) = crate::create_channel();
        let (_, events_channel_receiver) = crate::create_channel();

        let my_cert_bytes = include_bytes!("nodeserver/ca_cert.pem");

        let insecure_config = crate::Wsconfig {
            insecure: None,
            private_chain_bytes: Some(my_cert_bytes),
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
    async fn test_websocket_insecure() {
        let websocket_uri = WEBSOCKET_URI.to_string();

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
}
