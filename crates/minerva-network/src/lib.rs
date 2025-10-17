//! Networking facade for real-time event publication.

use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use minerva_types::{events::SystemEvent, Result};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tracing::info;

#[async_trait]
pub trait RealtimeServer: Send + Sync {
    async fn run(&self) -> Result<()>;
    async fn publish(&self, event: SystemEvent) -> Result<()>;
    fn subscribe(&self) -> BoxStream<'static, SystemEvent>;
}

/// Simple in-process server backed by a broadcast channel.
#[derive(Clone)]
pub struct LocalServer {
    tx: broadcast::Sender<SystemEvent>,
}

impl LocalServer {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }
}

#[async_trait]
impl RealtimeServer for LocalServer {
    async fn run(&self) -> Result<()> {
        info!("Starting local realtime server (noop)");
        Ok(())
    }

    async fn publish(&self, event: SystemEvent) -> Result<()> {
        let _ = self.tx.send(event);
        Ok(())
    }

    fn subscribe(&self) -> BoxStream<'static, SystemEvent> {
        BroadcastStream::new(self.tx.subscribe())
            .filter_map(|event| async move { event.ok() })
            .boxed()
    }
}
