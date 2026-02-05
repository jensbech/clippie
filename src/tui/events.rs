use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum Event {
    /// Terminal tick event
    Tick,
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize event
    Resize(u16, u16),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    #[allow(dead_code)]
    tx: mpsc::UnboundedSender<Event>,
    stop: Arc<AtomicBool>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let stop = Arc::new(AtomicBool::new(false));

        let stop_clone = Arc::clone(&stop);
        let tx_clone = tx.clone();

        thread::spawn(move || {
            loop {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }

                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    if let Ok(event) = event::read() {
                        match event {
                            CrosstermEvent::Key(key) => {
                                let _ = tx_clone.send(Event::Key(key));
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                let _ = tx_clone.send(Event::Mouse(mouse));
                            }
                            CrosstermEvent::Resize(w, h) => {
                                let _ = tx_clone.send(Event::Resize(w, h));
                            }
                            _ => {}
                        }
                    }
                }

                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                let _ = tx_clone.send(Event::Tick);
            }
        });

        EventHandler { rx, tx, stop }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new();
        handler.stop();
    }
}
