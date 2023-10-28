#[cfg(test)]
mod tests {

static  WEBSOCKET_URI: &str = "wss://localhost:3000";


    #[tokio::test]
    async fn test_websocket_handler() {

        let websocket_uri = WEBSOCKET_URI.to_string();


        let (ws_channel_sender, ws_channel_receiver) = crate::create_channel();
        let (events_channel_sender, events_channel_receiver) = crate::create_channel();



        tokio::spawn(crate::websocket_handler(websocket_uri, events_channel_receiver.clone(), ws_channel_sender.clone()));


        while let Ok(msg) = ws_channel_receiver.recv().await {
            println!("get msg");

            println!("message_task_parsed: {:?}", msg);
                if msg == "ping" {
                    ws_channel_sender.send(msg).await.unwrap();
                }
 
        }

    }
}
