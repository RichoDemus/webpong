use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::time::{sleep, Duration};
use log::*;
use tokio::sync::oneshot::Sender;
use tokio::task::JoinHandle;

struct ServerStub {
    closer: Sender<()>,
    handle: JoinHandle<()>,
}

impl ServerStub {
    async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let (sender, mut receiver) = oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            loop {
                match receiver.try_recv() {
                    Ok(_) => {
                        // shutdown
                        info!("Got shutdown message, closing..");
                        break;
                    }
                    Err(e) => match e {
                        TryRecvError::Closed => {
                            // sender closed
                            info!("Sender closed..");
                            break;
                        }
                        TryRecvError::Empty => {
                            // do another loop
                            // code goes here
                            info!("Do stuff");
                        }
                    }
                }
                sleep(Duration::from_millis(100)).await;
            }
            info!("Task closed");
        });

        Ok(ServerStub {
            closer:sender,
            handle,
        })
    }

    async fn stop(self) -> Result<(), Box<dyn std::error::Error>> {
        self.closer.send(()).map_err(|_|String::from("Failed to stop"))?;
        self.handle.await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[tokio::test]
    async fn test_normal_flow() -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .try_init();

        let server = ServerStub::start().await?;

        sleep(Duration::from_millis(300)).await;

        server.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_drop() -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .try_init();

        let server = ServerStub::start().await?;

        sleep(Duration::from_millis(300)).await;

        mem::drop(server);
        sleep(Duration::from_millis(300)).await;
        Ok(())
    }
}
