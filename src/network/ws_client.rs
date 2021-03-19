use futures::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use url::Url;

use crate::event_stream::EventStream;
use crate::network::ws_event::WsEvent;

pub struct Websocket {
    pub event_stream: EventStream<WsEvent>,
    write: SplitSink<
        WebSocketStream<
            tokio_tungstenite::stream::Stream<
                tokio::net::TcpStream,
                tokio_native_tls::TlsStream<tokio::net::TcpStream>,
            >,
        >,
        tokio_tungstenite::tungstenite::Message,
    >,
}

impl Websocket {
    pub async fn open() -> Self {
        #[cfg(debug_assertions)]
        let ws_url = "ws://localhost:8080";
        #[cfg(not(debug_assertions))]
        let ws_url = "wss://webpong.richodemus.com";

        Self::open_url(ws_url).await
    }

    pub async fn open_url(url: &str) -> Self {
        let url = Url::parse(url).unwrap();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (write, mut read) = ws_stream.split();

        let event_stream = EventStream::new();

        event_stream
            .buffer()
            .lock()
            .expect("expected obtain lock")
            .push_back(WsEvent::Opened);

        let buffer_clone = event_stream.buffer().clone();
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        let str = msg.to_string();
                        let msg = serde_json::from_str(str.as_str()).expect("deserialize");
                        buffer_clone
                            .lock()
                            .expect("expected lock")
                            .push_back(WsEvent::Message(msg));
                    }
                    Err(e) => log::info!("ws recv error: {:?}", e),
                };
            }
        });

        Websocket {
            event_stream,
            write,
        }
    }

    pub async fn send(&mut self, msg: crate::network::message::ClientMessage) {
        let msg = crate::network::message::Message::ClientMessage(msg);
        let str = serde_json::to_string(&msg).expect("Serialize json");
        self.write
            .send(Message::text(str.as_str()))
            .await
            .expect("Websocket.send failed");
    }

    #[allow(dead_code)]
    pub async fn close(&mut self) {
        self.write.close().await.expect("Websocket.close failed");
    }
}
