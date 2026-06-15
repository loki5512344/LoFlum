use crate::domain::connection::ConnectionStatus;
use crate::domain::file_entry::{EntryKind, FileEntry};
use crate::domain::transfer::{TransferKind, TransferTask};
use crate::fs::remote::RemoteRegistry;
use crate::transfer::queue::TransferQueue;
use crate::ui::drag::{make_remote_payload, DragPayload};
use crate::ui::panels::file_pane::{file_table, FileTableResponse};
use crate::ui::state::{AppState, PendingRemoteList};
use egui::TextEdit;
use std::sync::Arc;

pub fn remote_parent(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "/" {
        return "/".into();
    }
    if let Some(idx) = trimmed.rfind('/') {
        let parent = &trimmed[..idx];
        if parent.is_empty() {
            "/".into()
        } else {
            parent.to_string()
        }
    } else {
        "/".into()
    }
}

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    tab_idx: usize,
    registry: &Arc<RemoteRegistry>,
    rt_handle: &tokio::runtime::Handle,
    queue: &TransferQueue,
) {
    ui.horizontal(|ui| {
        ui.heading("\u{1F310} Remote");
    });

    let frame = egui::Frame::none().inner_margin(egui::Margin::symmetric(4.0, 2.0));
    let (_inner, dropped) = ui.dnd_drop_zone::<DragPayload, _>(frame, |ui| {
        render_content(ui, state, tab_idx, registry, rt_handle);
    });

    let is_connected = state.tabs[tab_idx].status == ConnectionStatus::Connected;
    let connection_id = state.tabs[tab_idx].id.clone();
    let remote_path = state.tabs[tab_idx].remote_path.clone();

    if let Some(payload_arc) = dropped {
        let payload = &*payload_arc;
        match payload {
            DragPayload::LocalFile(local_path, file_name) => {
                if !is_connected {
                    state.status_message = "No active connection".into();
                    return;
                }
                let remote_path_str =
                    format!("{}/{}", remote_path.trim_end_matches('/'), file_name);
                let task = TransferTask::new(
                    TransferKind::Upload,
                    connection_id.clone(),
                    local_path.clone(),
                    remote_path_str,
                    file_name.clone(),
                    0,
                );
                queue.push(task);
                state.status_message = "Upload queued via drag & drop".into();
            }
            DragPayload::RemoteFile(_, _, _) => {
                state.status_message = "Cannot drop remote file onto remote pane".into();
            }
        }
    }
}

fn render_content(
    ui: &mut egui::Ui,
    state: &mut AppState,
    tab_idx: usize,
    registry: &Arc<RemoteRegistry>,
    rt_handle: &tokio::runtime::Handle,
) {
    let label = state.tabs[tab_idx].label.clone();
    ui.horizontal(|ui| {
        ui.label(format!("\u{25CF} {}", label));
        if state.tabs[tab_idx].loading {
            ui.label("loading...");
        }
    });

    let path_changed;
    {
        let tab = &state.tabs[tab_idx];
        let mut path = tab.remote_path.clone();
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut path)
                    .id("remote_path".into())
                    .desired_width(f32::INFINITY),
            );
        });
        path_changed = path != tab.remote_path;
        if path_changed {
            state.tabs[tab_idx].remote_path = path;
        }
    }

    if path_changed {
        let loading = state.tabs[tab_idx].loading;
        if !loading {
            trigger_list(state, tab_idx, registry, rt_handle);
        }
    }

    ui.separator();

    let entries;
    let mut selected;
    {
        let tab = &state.tabs[tab_idx];
        entries = tab.remote_entries.clone();
        selected = tab.remote_selected.clone();
    }

    let is_connected = state.tabs[tab_idx].status == ConnectionStatus::Connected;
    let conn_id = state.tabs[tab_idx].id.clone();

    let table_id = format!("remote_table_{}", tab_idx);
    let FileTableResponse { double_clicked, .. } =
        file_table(ui, &table_id, &entries, &mut selected, move |entry| {
            if is_connected {
                make_remote_payload(entry, &conn_id)
            } else {
                None
            }
        });

    state.tabs[tab_idx].remote_selected = selected;

    if let Some(name) = double_clicked {
        let (nav_path, is_dir) = {
            if name == ".." {
                let parent = remote_parent(state.tabs[tab_idx].remote_path.as_str());
                (parent, true)
            } else {
                let entry = state.tabs[tab_idx]
                    .remote_entries
                    .iter()
                    .find(|e| e.name == name);
                match entry {
                    Some(e) if e.kind == EntryKind::Dir => (e.path.clone(), true),
                    _ => (state.tabs[tab_idx].remote_path.clone(), false),
                }
            }
        };

        if is_dir {
            state.tabs[tab_idx].remote_path = nav_path;
            let loading = state.tabs[tab_idx].loading;
            if !loading {
                trigger_list(state, tab_idx, registry, rt_handle);
            }
        }
    }

    ui.separator();
    {
        let tab = &state.tabs[tab_idx];
        ui.horizontal(|ui| {
            ui.label(format!("{} items", tab.remote_entries.len()));
            if tab.loading {
                ui.label(" loading...");
            }
        });
    }
}

pub fn trigger_list(
    state: &mut AppState,
    tab_idx: usize,
    registry: &Arc<RemoteRegistry>,
    rt_handle: &tokio::runtime::Handle,
) {
    if state.tabs[tab_idx].status != ConnectionStatus::Connected {
        state.status_message = "Not connected".into();
        return;
    }
    state.tabs[tab_idx].loading = true;

    let connection_id = state.tabs[tab_idx].id.clone();
    let path = state.tabs[tab_idx].remote_path.clone();
    let registry = registry.clone();

    let result: Arc<std::sync::Mutex<Option<Result<Vec<FileEntry>, String>>>> =
        Arc::new(std::sync::Mutex::new(None));
    let result_clone = result.clone();

    rt_handle.spawn(async move {
        let fs = match registry.get(&connection_id) {
            Some(fs) => fs,
            None => {
                *result_clone.lock().unwrap() = Some(Err("connection not found".into()));
                return;
            }
        };
        let r = fs.list(&path).await.map_err(|e| e.to_string());
        *result_clone.lock().unwrap() = Some(r);
    });

    state
        .pending_remote_list
        .push(PendingRemoteList { tab_idx, result });
}
