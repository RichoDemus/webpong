#[derive(Debug, Clone)]
pub enum WsEvent {
    Opened,
    Message(String),
    Error(String),
    Closed,
}