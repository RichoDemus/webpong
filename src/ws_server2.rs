use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::time::Duration;

use futures::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

use crate::event_stream_mutex::EventStream;
use crate::event_stream_mutex_client;
use crate::ws_event::WsEvent;

pub struct WebsocketServer {
    pub running: Arc<Mutex<bool>>,
    // ws_streams: Arc<Mutex<Vec<WebSocketStream<TcpStream>>>>,
    pub event_stream: event_stream_mutex_client::EventStream,
}

impl WebsocketServer {
    pub async fn start() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        // let ws_streams = Arc::new(Mutex::new(vec![]));
        let addr = "0.0.0.0:8080";
        let listener = TcpListener::bind(&addr).await?;
        info!("Listening on: {}", addr);

        let event_stream = event_stream_mutex_client::EventStream::new();
        let buffer = event_stream.buffer.clone();
        let running = Arc::new(Mutex::new(true));
        // let streams = ws_streams.clone();
        let result = WebsocketServer {
            running: running.clone(),
            // ws_streams,
            event_stream,
        };
        tokio::spawn(async move {
            while *running.lock().expect("lock") {
                let future = listener.accept();
                futures::pin_mut!(future);
                match futures::poll!(future) {
                    Poll::Ready(Ok((stream, _addr))) => {
                        match stream.peer_addr() {
                            Ok(peer) => info!("New client: {:?}", peer),
                            Err(e) => info!("failed to obtain peer: {:?}", e),
                        }
                        match accept_async(stream).await {
                            Ok(ws_stream) => {
                                let client = WebsocketClient::from(ws_stream);
                                buffer
                                    .lock()
                                    .expect("lock to send a new client")
                                    .push(client);
                            }
                            Err(e) => info!("Failed to upgrade websocket: {:?}", e),
                        }
                    }
                    _ => (), //info!("poll result: {:?}", o),
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            info!("Server no longer listening for new connections")
        });

        Ok(result)
    }

    pub async fn close(&mut self) {
        info!("Starting to close server listening socket");
        *self.running.clone().lock().expect("running read lock") = false;
    }
}

pub struct WebsocketClient {
    pub event_stream: EventStream,
    send: SplitSink<WebSocketStream<TcpStream>, Message>,
}

impl WebsocketClient {
    pub fn from(ws: WebSocketStream<TcpStream>) -> Self {
        let (send, mut receive) = ws.split();
        let event_stream = EventStream::new();

        let buffer = event_stream.buffer.clone();
        tokio::spawn(async move {
            while let Some(msg) = receive.next().await {
                info!("Received {:?} from client", msg);
                match msg {
                    Ok(msg) => match msg {
                        Message::Text(msg) => {
                            buffer
                                .lock()
                                .expect("client lock")
                                .push(WsEvent::Message(msg));
                        }
                        _ => {
                            break;
                        }
                    },
                    Err(e) => {
                        info!("websocket client error: {:?}", e);
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            info!("closing socket...");
            buffer.lock().expect("client lock").push(WsEvent::Closed);
        });

        let websocket_client = WebsocketClient { event_stream, send };
        websocket_client
    }

    pub async fn send(&mut self, msg: &str) {
        self.send
            .send(Message::Text(msg.to_string()))
            .await
            .expect("WebsocketClient.send failed");
    }
}
