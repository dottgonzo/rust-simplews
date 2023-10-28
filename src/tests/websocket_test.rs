#[cfg(test)]
mod tests {

    use crate::websocket;


    #[tokio::test]
    async fn test_websocket_handler() {



        let (ws_channel_sender, ws_channel_receiver) = kanal::unbounded_async();
        let (events_channel_sender, events_channel_receiver) = kanal::unbounded_async();

        // WebSocket connection on separate thread

        let uri = String::from("ciao");

        websocket::websocket_handler(&uri, ws_channel_receiver, events_channel_sender).await;
    }
}
