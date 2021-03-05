#[cfg(not(target_arch = "wasm32"))]
use warp::ws::{WebSocket};
#[cfg(not(target_arch = "wasm32"))]
use warp::{Filter};

#[cfg(not(target_arch = "wasm32"))]
use futures_util::{StreamExt, FutureExt, SinkExt};
use tokio::time::Duration;
use crate::event_stream_mutex::{EventStream, EventBuffer};
use std::sync::{Arc, Mutex};
use crate::ws_event::WsEvent;
use futures::stream::{SplitStream, SplitSink};
use warp::ws::Message;

pub struct WsServer {
    clients: Arc<Mutex<Vec<SplitSink<WebSocket, Message>>>>,
    pub event_stream: Arc<Mutex<EventBuffer>>,
}

impl WsServer {
    pub fn new() -> Self {
        let event_stream = EventStream::new();
        let clients = Arc::new(Mutex::new(vec![]));

        let clients2 = clients.clone();
        // tokio::spawn(async move{
        //     let clients2 = clients2.clone();
        //
        //     let routes = warp::path("echo")
        //         // The `ws()` filter will prepare the Websocket handshake.
        //         .and(warp::ws())
        //         .map(|ws: warp::ws::Ws| {
        //             // let clients = clients.clone();
        //             // And then our closure will be called when it completes...
        //             ws.on_upgrade(|websocket| async {
        //                 // let clients = clients.clone();
        //                 println!("onupgrade");
        //                 // Just echo all messages back...
        //                 let (mut tx, mut rx) = websocket.split();
        //                 clients2.lock().expect("expected to get lock").push(tx);
        //
        //                 while let Some(result) = rx.next().await {
        //                     let msg: warp::ws::Message = match result {
        //                         Ok(msg) => { msg },
        //                         Err(e) => {
        //                             eprintln!("websocket error: {}", e);
        //                             break;
        //                         }
        //                     };
        //                     println!("received message: {:?}", msg);
        //                     let msg:String = msg.to_str().unwrap().to_string();
        //                     event_stream.buffer.lock()
        //                         .expect("expected to obtain lock").push(WsEvent::Message(msg.clone()));
        //                 }
        //             })
        //         });
        //
        //     println!("Starting warp...");
        //
        //     warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
        //
        // });

        WsServer {
            clients: clients.clone(),
            event_stream: event_stream.buffer.clone(),
        }
    }
}
