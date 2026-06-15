use crate::domain::connection::ConnectionStatus;
use crate::domain::file_entry::EntryKind;
use crate::domain::transfer::{TransferKind, TransferTask};
use crate::fs::remote::RemoteRegistry;
use crate::transfer::queue::TransferQueue;
use crate::ui::state::AppState;
use std::sync::Arc;

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    queue: &TransferQueue,
    registry: &Arc<RemoteRegistry>,
    _rt_handle: &tokio::runtime::Handle,
) {
    ui.horizontal(|ui| {
        let add_btn = egui::Button::new("+ New Connection");
        if ui.add(add_btn).clicked() {
            state.show_connect_dialog = true;
        }

        ui.separator();

        let has_local_selection = state.local_selected.is_some();
        let has_remote_selection = state
            .active_tab_ref()
            .and_then(|t| t.remote_selected.as_ref())
            .is_some();
        let (is_connected, tab_exists) = {
            let active = state.active_tab_ref();
            (
                active
                    .map(|t| t.status == ConnectionStatus::Connected)
                    .unwrap_or(false),
                active.is_some(),
            )
        };

        let tab_id = state.active_tab_ref().map(|t| t.id.clone());
        let remote_path = state.active_tab_ref().map(|t| t.remote_path.clone());

        let upload_btn = egui::Button::new("\u{2191} Upload")
            .min_size(egui::vec2(100.0, 24.0));
        ui.add_enabled_ui(is_connected && has_local_selection && tab_exists, |ui| {
            if ui.add(upload_btn).clicked()
                && let Some(sel) = state.local_selected.clone()
                && let Some(entry) = state.local_entries.iter().find(|e| e.name == sel)
            {
                if entry.kind == EntryKind::File {
                    let file_name = entry.name.clone();
                    let remote_base = remote_path.clone().unwrap_or_else(|| "/".into());
                    let remote_path_str =
                        format!("{}/{}", remote_base.trim_end_matches('/'), file_name);
                    let cid = tab_id.clone().unwrap_or_default();
                    let task = TransferTask::new(
                        TransferKind::Upload,
                        cid,
                        entry.path.clone(),
                        remote_path_str,
                        file_name,
                        entry.size.unwrap_or(0),
                    );
                    if registry.get(&task.connection_id).is_some() {
                        queue.push(task);
                        state.status_message = "Upload queued".into();
                    } else {
                        state.status_message = "No connection".into();
                    }
                } else {
                    state.status_message = "Cannot upload directory".into();
                }
            }
        });

        let download_btn = egui::Button::new("\u{2193} Download")
            .min_size(egui::vec2(100.0, 24.0));
        ui.add_enabled_ui(is_connected && has_remote_selection && tab_exists, |ui| {
            if ui.add(download_btn).clicked()
                && let Some(sel) = tab_remote_selected(state)
            {
                let file_name = sel.clone();
                let remote_base = remote_path.clone().unwrap_or_else(|| "/".into());
                let remote_path_str =
                    format!("{}/{}", remote_base.trim_end_matches('/'), file_name);
                let local_path_str =
                    format!("{}/{}", state.local_path.trim_end_matches('/'), file_name);
                let size = state
                    .active_tab_ref()
                    .map(|t| {
                        t.remote_entries
                            .iter()
                            .find(|e| e.name == sel)
                            .and_then(|e| e.size)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                let cid = tab_id.clone().unwrap_or_default();
                let task = TransferTask::new(
                    TransferKind::Download,
                    cid,
                    local_path_str,
                    remote_path_str,
                    file_name,
                    size,
                );
                if registry.get(&task.connection_id).is_some() {
                    queue.push(task);
                    state.status_message = "Download queued".into();
                } else {
                    state.status_message = "No connection".into();
                }
            }
        });

        ui.separator();

        let queue_label = if state.show_queue {
            "Queue \u{25BC}"
        } else {
            "Queue \u{25B2}"
        };
        if ui.button(queue_label).clicked() {
            state.show_queue = !state.show_queue;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.separator();
            let task_count = state.queue_tasks.len();
            if task_count > 0 {
                ui.label(format!("{} tasks", task_count));
            }
        });
    });
}

fn tab_remote_selected(state: &AppState) -> Option<String> {
    let idx = state.active_tab.min(state.tabs.len().saturating_sub(1));
    state.tabs.get(idx).and_then(|t| t.remote_selected.clone())
}
