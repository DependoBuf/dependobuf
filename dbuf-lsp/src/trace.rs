/// Module for trace hangling. Exports:
/// * `LoggerNotifier` struct, that makes a task, which sends every log entry to client.
/// * `setup_tracing` function, that setup tracing subsciber for all threads.
use std::io::{self, Write};

use tokio::sync::mpsc;

use tower_lsp::Client;
use tower_lsp::lsp_types::MessageType;
use tracing_subscriber::EnvFilter;

/// A write for unbounded mpsc channel.
struct ChannelWriter {
    /// Input of channel to send logs
    tx: mpsc::UnboundedSender<String>,
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let readed = buf.len();
        let data = String::from_utf8_lossy(buf).into_owned();
        let _ = self.tx.send(data);
        Ok(readed)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Function that setups tracing for all threads.
///
/// # Panics
///
/// Panics if called more than once.
pub fn setup_tracing(tx: mpsc::UnboundedSender<String>) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_writer(move || ChannelWriter { tx: tx.clone() })
        .with_max_level(tracing::Level::INFO)
        .with_env_filter(env_filter)
        .with_ansi(false)
        .init();
}

/// Notifier, that on every log sends it to client
pub struct LoggerNotifier {
    /// Lsp client
    pub client: Client,
    /// Channel of input messages
    pub rx: mpsc::UnboundedReceiver<String>,
}

impl LoggerNotifier {
    /// A function that runs logger as separate task. Returs handle of task
    pub fn run_task(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(entry) = self.rx.recv().await {
                self.client.log_message(MessageType::LOG, entry).await;
            }
        })
    }
}
