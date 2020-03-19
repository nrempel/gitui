use crossterm::event::{self, Event};
use std::{
    sync::mpsc::{self, Receiver},
    thread::{self, sleep},
    time::{Duration, Instant},
};

///
#[derive(Clone)]
pub enum QueueEvent {
    Tick,
    Event(Event),
}

static MAX_POLL_DURATION: Duration = Duration::from_secs(2);
static MIN_POLL_DURATION: Duration = Duration::from_millis(5);
static MAX_BATCHING_DURATION: Duration = Duration::from_millis(25);
static TICK_DURATION: Duration = Duration::from_secs(2);

/// we run 2 threads feeding us with update events.
///
/// Thread 1:
///     We will
pub fn start_polling_thread() -> Receiver<Vec<QueueEvent>> {
    let (tx, rx) = mpsc::channel();

    let tx1 = tx.clone();
    thread::spawn(move || {
        let mut last_send = Instant::now();
        let mut batch = Vec::new();

        loop {
            let timeout = if batch.len() > 0 {
                MIN_POLL_DURATION
            } else {
                MAX_POLL_DURATION
            };
            if let Some(e) = poll(timeout) {
                batch.push(QueueEvent::Event(e));
            }

            if batch.len() > 0
                && last_send.elapsed() > MAX_BATCHING_DURATION
            {
                tx1.send(batch).unwrap();
                batch = Vec::new();
                last_send = Instant::now();
            }
        }
    });

    thread::spawn(move || loop {
        tx.send(vec![QueueEvent::Tick]).unwrap();
        sleep(TICK_DURATION);
    });

    rx
}

///
fn poll(dur: Duration) -> Option<Event> {
    if event::poll(dur).unwrap() {
        // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
        let event = event::read().unwrap();
        Some(event)
    } else {
        None
    }
}
