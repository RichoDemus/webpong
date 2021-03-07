use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use crate::event_stream_mutex::EventStream;
use crate::ws_event::WsEvent;

pub struct Websocket {
    pub event_stream: EventStream,
    ws: WebSocket,
}

impl Websocket {
    pub async fn open(url: &str) -> Self {
        // console_log!("Time to open socket");
        let ws: WebSocket = WebSocket::new(url).unwrap();
        // console_log!("opened, kinda");

        let event_stream = EventStream::new();

        {
            // On Error
            let buffer = event_stream.buffer.clone();
            let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                // console_log!("onerror");
                buffer
                    .lock()
                    .expect("aquire lock")
                    .push(WsEvent::Error(e.message().into()));
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();
        }

        {
            // On Message
            let buffer = event_stream.buffer.clone();
            let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                // console_log!("onmessage: {:?}", e);
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    // console_log!("onmessage: {:?}", txt);
                    buffer
                        .lock()
                        .expect("aquire lock")
                        .push(WsEvent::Message(txt.into()));
                    // console_log!("after push");
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }

        {
            // on close
            let buffer = event_stream.buffer.clone();
            let onclose_callback = Closure::wrap(Box::new(move |_| {
                // console_log!("onclose");
                buffer.lock().expect("aquire lock").push(WsEvent::Closed);
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();
        }

        {
            // on open
            let buffer = event_stream.buffer.clone();
            let onopen_callback = Closure::wrap(Box::new(move |_| {
                // console_log!("onopen");
                buffer.lock().expect("aquire lock").push(WsEvent::Opened);
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();
        }

        // console_log!("return from ws client");
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
