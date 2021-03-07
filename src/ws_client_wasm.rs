use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use crate::event_stream::EventStream;
use crate::ws_event::WsEvent;

pub struct Websocket {
    pub event_stream: EventStream<WsEvent>,
    ws: WebSocket,
}

impl Websocket {
    pub async fn open(url: &str) -> Self {
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
                    buffer
                        .lock()
                        .expect("aquire lock")
                        .push_back(WsEvent::Message(txt.into()));
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

    pub fn send(&mut self, str: &str) {
        self.ws.send_with_str(str).expect("wsclient.send failed");
    }

    #[allow(dead_code)]
    fn close(&mut self) {
        let _ = self.ws.close();
    }
}