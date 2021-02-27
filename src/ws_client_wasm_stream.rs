use
{
    ws_stream_wasm       :: *                        ,
    // pharos               :: *                        ,
    // wasm_bindgen         :: UnwrapThrowExt           ,
    // wasm_bindgen_futures :: futures_0_3::spawn_local ,
    // futures              :: stream::StreamExt        ,
};
use futures::channel::mpsc::Receiver;
use wasm_bindgen::UnwrapThrowExt;
use futures::{StreamExt, SinkExt};
use quicksilver::log::warn;

pub struct Websocket {
    pub receiver: Receiver<String>,
}
//
// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }
//
// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }

pub async fn start_ws_client() -> Websocket {
    warn!("time to do stuff");
    let (mut tx, mut rx) = futures::channel::mpsc::channel::<String>(100);

    let (mut ws, wsio) = WsMeta::connect("ws://localhost:8080/echo", None ).await.expect_throw( "assume the connection succeeds" );

    let (mut send, mut receive) = wsio.split();

    send.send(WsMessage::Text("1".to_string())).await;

    while let Some(message) = receive.next().await {
        warn!("got message: {:?}", message);
        send.send(message.clone()).await;
        // if let WsMessage::Text(msg) = message {
        //     tx.send(msg).await;
        // }

    };

    Websocket { receiver:rx}

    // while let Some(message) = read.next().await {
    //     let message = message.unwrap();
    //     println!("Received: {:?}, returning it to main loop", message);
    //     sender.send(message.to_string()).await;
    //     write.send(message).await;
    //     tokio::time::sleep(Duration::from_millis(100)).await;
    // }







    // let framed              = Framed::new( wsio, Codec::new() );
    // let (mut out, mut msgs): (String, String) = framed.split();





    // let mut evts = ws.observe( ObserveConfig::default() ).expect_throw( "observe" );


    // ws.close().await;

    // Note that since WsMeta::connect resolves to an opened connection, we don't see
    // any Open events here.
    //
    // assert!( evts.next().await.unwrap_throw().is_closing() );
    // assert!( evts.next().await.unwrap_throw().is_closed () );



}