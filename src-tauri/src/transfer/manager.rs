use crate::fs::remote::RemoteRegistry;
use crate::transfer::queue::TransferQueue;
use std::sync::Arc;

pub struct TransferManager {
    pub queue: TransferQueue,
    pub registry: Arc<RemoteRegistry>,
}

impl TransferManager {
    pub fn new(registry: Arc<RemoteRegistry>) -> Arc<Self> {
        Arc::new(Self {
            queue: TransferQueue::default(),
            registry,
        })
    }

    pub fn registry(&self) -> &RemoteRegistry {
        &self.registry
    }
}
