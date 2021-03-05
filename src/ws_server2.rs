use futures_util::{SinkExt, StreamExt};
use log::*;
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tungstenite::{Result};
use tokio_tungstenite::tungstenite::Message;
use tungstenite::Error;
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Relaxed, Release, Acquire};
use crate::event_stream_mutex::EventStream;
use futures::stream::SplitSink;
use crate::ws_event::WsEvent;
use crate::event_stream_mutex_client;

pub struct WebsocketServer {
    pub running: Arc<Mutex<bool>>,
    // ws_streams: Arc<Mutex<Vec<WebSocketStream<TcpStream>>>>,
    pub event_stream: event_stream_mutex_client::EventStream,
}

impl WebsocketServer {
    pub async fn start() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        // let ws_streams = Arc::new(Mutex::new(vec![]));
        let addr = "127.0.0.1:8080";
        let listener = TcpListener::bind(&addr).await?;
        info!("Listening on: {}", addr);

        let event_stream = event_stream_mutex_client::EventStream::new();
        let buffer = event_stream.buffer.clone();
        let running = Arc::new(Mutex::new(true));
        // let streams = ws_streams.clone();
        let result = WebsocketServer{
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
                        let peer = stream.peer_addr().expect("connected streams should have a peer address");
                        info!("New client: {:?}", peer);
                        let ws_stream = accept_async(stream).await.expect("failed to accept");

                        let client = WebsocketClient::from(ws_stream);
                        buffer.lock().expect("lock to send a new client").push(client);
                    }
                    _ => (),//info!("poll result: {:?}", o),
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            info!("Stopping websocket listening")
        });

        Ok(result)
    }

    pub async fn close(&mut self) {
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
                info!("Client received: {:?}", msg);
                let msg = msg.expect("message received unwrap");
                match msg {
                    Message::Text(msg) => {
                        buffer.lock().expect("client lock").push(WsEvent::Message(msg));
                    }
                    _ => {
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100));
            }
            info!("closing socket...");
        });

        let websocket_client = WebsocketClient {
            event_stream,
            send
        };
        websocket_client
    }

     pub async fn send(&mut self, msg: &str) {
        self.send.send(Message::Text(msg.to_string())).await;
    }
}
