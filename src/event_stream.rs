use futures_util::future::poll_fn;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::Arc;
use std::task::Poll;
use crate::ws_event::WsEvent;

/// The source of events for a `blinds` application
///
/// An `EventStream` instance is supplied by [`run`], so creating one is not necessary. Use the
/// [`next_event`] function to wait for [`Event`]s.
///
/// [`next_event`]: EventStream::next_event
/// [`Event`]: Event
/// [`run`]: crate::run()
pub struct EventStream {
    buffer: Arc<RefCell<EventBuffer>>,
}

impl EventStream {
    pub(crate) fn new() -> EventStream {
        EventStream {
            buffer: Arc::new(RefCell::new(EventBuffer {
                events: VecDeque::new(),
            })),
        }
    }

    pub(crate) fn buffer(&self) -> Arc<RefCell<EventBuffer>> {
        self.buffer.clone()
    }

    /// Returns a future that will provide the next [`Event`], or None if the events are exhausted
    ///
    /// If there are no events, the Future will wait until new events are received, allowing the OS
    /// or browser to take back control of the event loop. If this doesn't get run, desktop windows
    /// will freeze and browser windows will lock up, so it's important to call and `.await` the
    /// Future even if the events are ignored.
    ///
    /// [`Event`]: Event
    pub fn next_event<'a>(&'a mut self) -> impl 'a + Future<Output = Option<WsEvent>> {
        poll_fn(move |_cx| {
            let mut buffer = self.buffer.borrow_mut();
            let option = buffer.events.pop_front();
            // console_log!("popped {:?}, buffer ready?: {}", option, buffer.ready);
            match option {
                Some(event) => Poll::Ready(Some(event)),
                None => {
                    Poll::Ready(None)
                }
            }
        })
    }
}

pub(crate) struct EventBuffer {
    events: VecDeque<WsEvent>,
}

impl EventBuffer {
    pub fn push(&mut self, event: WsEvent) {
        self.events.push_back(event);
    }
}
