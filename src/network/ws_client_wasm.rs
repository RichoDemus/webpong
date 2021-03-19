use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use crate::event_stream::EventStream;
use crate::network::ws_event::WsEvent;

pub struct Websocket {
    pub event_stream: EventStream<WsEvent>,
    ws: WebSocket,
}

impl Websocket {
    pub async fn open() -> Self {
        #[cfg(debug_assertions)]
        let ws_url = "ws://localhost:8080";
        #[cfg(not(debug_assertions))]
        let ws_url = "wss://webpong.richodemus.com";

        Self::open_url(ws_url).await
    }

    pub async fn open_url(url: &str) -> Self {
        let ws: WebSocket = WebSocket::new(url).unwrap();

        let event_stream = EventStream::new();

        {
            // On Error
            let buffer = event_stream.buffer().clone();
            let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                buffer
                    .lock()
                    .expect("aquire lock")
                    .push_back(WsEvent::Error(e.message().into()));
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();
        }

        {
            // On Message
            let buffer = event_stream.buffer().clone();
            let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let txt: String = txt.into();
                    let msg = serde_json::from_str(txt.as_str()).expect("deserialize json");
                    buffer
                        .lock()
                        .expect("aquire lock")
                        .push_back(WsEvent::Message(msg));
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }

        {
            // on close
            let buffer = event_stream.buffer().clone();
            let onclose_callback = Closure::wrap(Box::new(move |_| {
                buffer
                    .lock()
                    .expect("aquire lock")
                    .push_back(WsEvent::Closed);
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();
        }

        {
            // on open
            let buffer = event_stream.buffer().clone();
            let onopen_callback = Closure::wrap(Box::new(move |_| {
                buffer
                    .lock()
                    .expect("aquire lock")
                    .push_back(WsEvent::Opened);
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();
        }

        Websocket { event_stream, ws }
    }

    pub async fn send(&mut self, msg: crate::network::message::ClientMessage) {
        let msg = crate::network::message::Message::ClientMessage(msg);
        let str = serde_json::to_string(&msg).expect("Serialize json");
        self.ws
            .send_with_str(str.as_str())
            .expect("wsclient.send failed");
    }

    #[allow(dead_code)]
    fn close(&mut self) {
        let _ = self.ws.close();
    }
}
