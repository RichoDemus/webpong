#[cfg(test)]
mod tests {
    use log::*;
    use tokio::time::Duration;

    use crate::ws_event::WsEvent;
    use crate::{ws_client, ws_server};

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn my_test() -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::builder()
            .filter_module("webpong", log::LevelFilter::Trace)
            .try_init();

        let ws_server = ws_server::WebsocketServer::start();
        let ws_client_one = ws_client::Websocket::open_url("ws://localhost:8080");
        let ws_client_two = ws_client::Websocket::open_url("ws://localhost:8080");

        let (server, client_one, client_two) =
            tokio::join!(ws_server, ws_client_one, ws_client_two,);

        let mut server = server?;
        let server_running_for_stopping = server.running.clone();
        let server_running_for_listen_loop = server.running.clone();
        tokio::spawn(async move {
            while *server_running_for_listen_loop.lock().expect("loop") {
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
                                        break;
                                    }
                                    m => warn!("Server received unexpected msg: {:?}", m),
                                }
                                tokio::time::sleep(Duration::from_millis(1)).await;
                            }
                        }
                        trace!("Server listening loop ended...");
                    });
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
            server.close().await;
            trace!("Server task stopped");
        });

        let client_futures = vec![client_one, client_two]
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
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                    info!("Client listening loop ending...");
                    ws.close().await;
                    trace!("Client listening loop ended");
                })
            })
            .collect::<Vec<_>>();

        info!("Waiting for futures...");
        for future in client_futures {
            future.await.expect("future await failed");
        }
        info!("futures done");

        *server_running_for_stopping.lock().expect("close server") = false;
        info!("Main thread ending, both clients are done");
        Ok(())
    }
}
