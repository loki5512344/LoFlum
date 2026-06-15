use std::sync::Arc;
use tokio::time::{Duration, sleep};

use crate::domain::transfer::{TaskState, TransferKind};
use crate::fs::remote::RemoteRegistry;
use crate::transfer::queue::TransferQueue;

pub fn spawn_worker(
    queue: TransferQueue,
    registry: Arc<RemoteRegistry>,
    rt_handle: tokio::runtime::Handle,
) {
    rt_handle.spawn(async move {
        loop {
            let task = {
                let task = queue.pop();
                if let Some(ref t) = task
                    && t.state != TaskState::Queued
                {
                    queue.push(t.clone());
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }
                task
            };

            let mut task = match task {
                Some(t) => t,
                None => {
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }
            };

            task.state = TaskState::Running;
            queue.update_state(&task.id, TaskState::Running);

            let fs = match registry.get(&task.connection_id) {
                Some(fs) => fs,
                None => {
                    task.state = TaskState::Failed("connection not found".into());
                    queue.update_state(&task.id, task.state.clone());
                    continue;
                }
            };

            let result = match task.kind {
                TransferKind::Upload => fs.upload(&task.local_path, &task.remote_path).await,
                TransferKind::Download => fs.download(&task.remote_path, &task.local_path).await,
            };

            match result {
                Ok(()) => {
                    task.state = TaskState::Completed;
                    task.transferred_bytes = task.total_bytes;
                    queue.update_state(&task.id, TaskState::Completed);
                    queue.update_progress(&task.id, task.total_bytes, 0);
                }
                Err(e) => {
                    let msg = e.to_string();
                    task.state = TaskState::Failed(msg);
                    queue.update_state(&task.id, task.state.clone());
                }
            }
        }
    });
}
