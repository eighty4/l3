use std::time::Duration;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::notification::LambdaNotification;

struct TerminalDashboardEventLoop {
    de_rx: Receiver<LambdaNotification>,
}

pub struct TerminalDashboard {
    pub de_tx: Sender<LambdaNotification>,
}

impl TerminalDashboardEventLoop {
    fn new() -> (Self, Sender<LambdaNotification>) {
        let (de_tx, de_rx) = mpsc::channel::<LambdaNotification>(10);
        (Self { de_rx }, de_tx)
    }

    async fn start(&mut self) {}
}

impl TerminalDashboard {
    pub fn new() -> Self {
        let (mut event_loop, de_tx) = TerminalDashboardEventLoop::new();
        tokio::spawn(async move { event_loop.start().await });
        Self { de_tx }
    }

    pub async fn start_dev_interface(&self) {
        tokio::time::sleep(Duration::from_secs(1000)).await;
    }
}
