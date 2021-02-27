use futures_util::{SinkExt, StreamExt};
use tokio::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use futures::channel::mpsc::{Receiver, Sender};

pub struct Websocket {
    runtime: Runtime,
    pub receiver: Receiver<String>,
}

pub fn start_ws_client() -> Websocket {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (mut tx, mut rx) = futures::channel::mpsc::channel::<String>(100);
    runtime
        .spawn(start(tx));

    Websocket {
        runtime,
        receiver: rx,
    }
}

async fn start(mut sender: Sender<String>) {
    println!("start called");
    let url = Url::parse("ws://localhost:8080/echo").unwrap();

    // let (stdin_tx, stdin_rx):(futures_channel::mpsc::UnboundedSender<Message>,futures_channel::mpsc::UnboundedReceiver<Message>) = futures_channel::mpsc::unbounded();

    // tokio::spawn(async move {
    //
    // });

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // let read_sharable = Arc::new(Mutex::new(read));

    write.send(Message::text("1")).await;
    while let Some(message) = read.next().await {
        let message = message.unwrap();
        println!("Received: {:?}, returning it to main loop", message);
        sender.send(message.to_string()).await;
        write.send(message).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    // let ws_to_stdout = {
    //     read.for_each(|message| async {
    //         let message = message.unwrap();
    //         println!("Received: {:?}", message);
    //         let data = message.into_data();
    //         tokio::io::stdout().write_all(&data).await.unwrap();
    //         write.send(message).await;
    //         tokio::time::sleep(Duration::from_millis(100)).await;
    //     })
    // };
    //
    // write.send(Message::text("1")).await;


    // future::select(write, read).await;


    // ws_to_stdout.await;

    // let (mut socket, response) =
    //     connect(url.unwrap()).expect("Can't connect");
    //
    // println!("Connected to the server");
    // println!("Response HTTP code: {}", response.status());
    // println!("Response contains the following headers:");
    // for (ref header, _value) in response.headers() {
    //     println!("* {}", header);
    // }
    //
    // socket.write_message(Message::Text("Hello WebSocket".into())).unwrap();
    // loop {
    //     let msg = socket.read_message().expect("Error reading message");
    //     println!("Received: {}", msg);
    // }
    // socket.close(None);
}
