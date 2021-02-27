#[cfg(not(target_arch = "wasm32"))]
use warp::ws::{WebSocket};
#[cfg(not(target_arch = "wasm32"))]
use warp::{Filter};

#[cfg(not(target_arch = "wasm32"))]
use futures_util::{StreamExt, FutureExt, SinkExt};
use tokio::time::Duration;

pub fn start_ws_server() {
    println!("tokio");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(start())
}


async fn start() {
    let routes = warp::path("echo")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| async {
                println!("onupgrade");
                // Just echo all messages back...
                let (mut tx, mut rx) = websocket.split();

                let msg = warp::filters::ws::Message::text("0");
                tx.send(msg).await;

                while let Some(result) = rx.next().await {
                    let msg: warp::ws::Message = match result {
                        Ok(msg) => msg,
                        Err(e) => {
                            eprintln!("websocket error: {}", e);
                            break;
                        }
                    };
                    println!("received message: {:?}", msg);
                    let msg:String = msg.to_str().unwrap().to_string();
                    let mut msg:u64 = msg.parse().unwrap();
                    msg += 1;
                    let msg = warp::filters::ws::Message::text(format!("{}", msg));
                    tx.send(msg).await;
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }

                // rx.forward(tx).map(|result| {
                //     if let Err(e) = result {
                //         eprintln!("websocket error: {:?}", e);
                //     }
                // })
            })
        });

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
