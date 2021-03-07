use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use url::Url;
use crate::event_stream_mutex::{EventStream};
use futures::stream::SplitSink;

use crate::ws_event::WsEvent;

pub struct Websocket {
    pub event_stream: EventStream,
    write:  SplitSink<WebSocketStream<tokio_tungstenite::stream::Stream<tokio::net::TcpStream, tokio_native_tls::TlsStream<tokio::net::TcpStream>>>, tokio_tungstenite::tungstenite::Message>,
}

impl Websocket {
    pub async fn open(url: &str) -> Self {
        let url = Url::parse(url).unwrap();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (write, mut read)= ws_stream.split();

        let event_stream = EventStream::new();

        event_stream.buffer.lock().expect("expected obtain lock").push(WsEvent::Opened);

        let buffer_clone = event_stream.buffer.clone();
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        let str = msg.to_string();
                        buffer_clone.lock().expect("expected lock").push(WsEvent::Message(str));
                    },
                    Err(e) => log::info!("ws recv error: {:?}", e),
                };
            }
        });

        Websocket {
            event_stream,
            write,
        }
    }

    pub async fn send(&mut self, str: &str) {
        self.write.send(Message::text(str.clone())).await.expect("Websocket.send failed");
    }

    #[allow(dead_code)]
    pub async fn close(&mut self) {
        self.write.close().await.expect("Websocket.close failed");
    }
}

// async fn start(mut sender: Sender<String>) {
//     println!("start called");
//     let url = Url::parse("ws://localhost:8080/echo").unwrap();
//
//     // let (stdin_tx, stdin_rx):(futures_channel::mpsc::UnboundedSender<Message>,futures_channel::mpsc::UnboundedReceiver<Message>) = futures_channel::mpsc::unbounded();
//
//     // tokio::spawn(async move {
//     //
//     // });
//
//     let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
//     let (mut write, mut read) = ws_stream.split();
//
//     // let read_sharable = Arc::new(Mutex::new(read));
//
//     write.send(Message::text("1")).await;
//     while let Some(message) = read.next().await {
//         let message = message.unwrap();
//         println!("Received: {:?}, returning it to main loop", message);
//         sender.send(message.to_string()).await;
//         write.send(message).await;
//         tokio::time::sleep(Duration::from_millis(100)).await;
//     }
//
//     // let stdin_to_ws = stdin_rx.map(Ok).forward(write);
//     // let ws_to_stdout = {
//     //     read.for_each(|message| async {
//     //         let message = message.unwrap();
//     //         println!("Received: {:?}", message);
//     //         let data = message.into_data();
//     //         tokio::io::stdout().write_all(&data).await.unwrap();
//     //         write.send(message).await;
//     //         tokio::time::sleep(Duration::from_millis(100)).await;
//     //     })
//     // };
//     //
//     // write.send(Message::text("1")).await;
//
//
//     // future::select(write, read).await;
//
//
//     // ws_to_stdout.await;
//
//     // let (mut socket, response) =
//     //     connect(url.unwrap()).expect("Can't connect");
//     //
//     // println!("Connected to the server");
//     // println!("Response HTTP code: {}", response.status());
//     // println!("Response contains the following headers:");
//     // for (ref header, _value) in response.headers() {
//     //     println!("* {}", header);
//     // }
//     //
//     // socket.write_message(Message::Text("Hello WebSocket".into())).unwrap();
//     // loop {
//     //     let msg = socket.read_message().expect("Error reading message");
//     //     println!("Received: {}", msg);
//     // }
//     // socket.close(None);
// }
