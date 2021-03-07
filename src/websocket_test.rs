#[cfg(test)]
mod tests {
    use std::error::Error;

    use log::*;
    use tokio::io::AsyncWriteExt;
    use tokio::time::{Duration, Instant};

    use crate::ws_event::WsEvent;
    use crate::{ws_client, ws_server2};

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn my_test() -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::builder()
            .filter_module("webpong", log::LevelFilter::Info)
            .try_init();

        let ws_server = ws_server2::WebsocketServer::start();
        let ws_client_one = ws_client::Websocket::open("ws://localhost:8080");
        let ws_client_two = ws_client::Websocket::open("ws://localhost:8080");

        let (server, mut second, mut third) =
            tokio::join!(ws_server, ws_client_one, ws_client_two,);

        let mut server = server?;
        let server_running = server.running.clone();
        tokio::spawn(async move {
            while let Some(mut ws_client) = server.event_stream.next_event().await {
                tokio::spawn(async move {
                    info!("Listening to messages from client");
                    loop {
                        if let Some(msg) = ws_client.event_stream.next_event().await {
                            match msg {
                                WsEvent::Message(msg) => {
                                    let msg = msg.as_str();
                                    info!("Server received msg: {:?}", msg);
                                    ws_client.send(msg).await;
                                    tokio::time::sleep(Duration::from_millis(100)).await;
                                }
                                _ => (),
                            }
                            tokio::time::sleep(Duration::from_millis(100));
                        }
                    }
                    info!("Listening to client done");
                });
                tokio::time::sleep(Duration::from_millis(100));
            }
            server.close();
        });

        let futures = vec![second, third]
            .into_iter()
            .map(|mut ws| {
                tokio::spawn(async move {
                    loop {
                        match ws.event_stream.next_event().await {
                            None => {}
                            Some(msg) => match msg {
                                WsEvent::Opened => ws.send("hello from client").await,
                                WsEvent::Message(msg) => {
                                    info!("ws client msg: {:?}", msg);
                                    break;
                                }
                                WsEvent::Error(_) => {}
                                WsEvent::Closed => {}
                            },
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    ws.close().await;
                })
            })
            .collect::<Vec<_>>();

        for future in futures {
            future.await.expect("future await failed");
        }

        *server_running.lock().expect("close server") = false;
        Ok(())
    }
}
