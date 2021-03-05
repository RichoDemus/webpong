use futures_util::future::poll_fn;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use wasm_bindgen::prelude::*;
use crate::ws_server2::WebsocketClient;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// The source of events for a `blinds` application
///
/// An `EventStream` instance is supplied by [`run`], so creating one is not necessary. Use the
/// [`next_event`] function to wait for [`Event`]s.
///
/// [`next_event`]: EventStream::next_event
/// [`Event`]: Event
/// [`run`]: crate::run()
pub struct EventStream {
    pub buffer: Arc<Mutex<EventBuffer>>,
}

impl EventStream {
    pub(crate) fn new() -> EventStream {
        EventStream {
            buffer: Arc::new(Mutex::new(EventBuffer {
                events: VecDeque::new(),
                waker: None,
                ready: false,
            })),
        }
    }

    // pub(crate) fn buffer(&self) -> Arc<Mutex<EventBuffer>> {
    //     self.buffer.clone().lock()
    // }

    /// Returns a future that will provide the next [`Event`], or None if the events are exhausted
    ///
    /// If there are no events, the Future will wait until new events are received, allowing the OS
    /// or browser to take back control of the event loop. If this doesn't get run, desktop windows
    /// will freeze and browser windows will lock up, so it's important to call and `.await` the
    /// Future even if the events are ignored.
    ///
    /// [`Event`]: Event
    pub fn next_event<'a>(&'a mut self) -> impl 'a + Future<Output = Option<WebsocketClient>> {
        poll_fn(move |cx| {
            let buffer = self.buffer.clone();
            let mut buffer = buffer.lock().expect("expected to obtain lock");
            let option = buffer.events.pop_front();
            // console_log!("popped {:?}, buffer ready?: {}", option, buffer.ready);
            match option {
                Some(event) => Poll::Ready(Some(event)),
                None => {
                    Poll::Ready(None)
                    // if buffer.ready {
                    //     buffer.ready = false;
                    //     Poll::Ready(None)
                    // } else {
                    //     buffer.waker = Some(cx.waker().clone());
                    //     Poll::Pending
                    // }
                }
            }
        })
    }
}

pub struct EventBuffer {
    events: VecDeque<WebsocketClient>,
    waker: Option<Waker>,
    ready: bool,
}

impl EventBuffer {
    pub fn push(&mut self, event: WebsocketClient) {
        self.events.push_back(event);
        self.mark_ready();
    }

    pub fn mark_ready(&mut self) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
        self.ready = true;
    }
}
