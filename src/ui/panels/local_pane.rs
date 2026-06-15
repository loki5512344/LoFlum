use crate::domain::connection::ConnectionStatus;
use crate::domain::file_entry::{EntryKind, FileEntry};
use crate::domain::transfer::{TransferKind, TransferTask};
use crate::fs::local;
use crate::fs::remote::RemoteRegistry;
use crate::transfer::queue::TransferQueue;
use crate::ui::drag::{make_local_payload, DragPayload};
use crate::ui::panels::file_pane::{file_table, FileTableResponse};
use crate::ui::state::AppState;
use egui::TextEdit;
use std::sync::Arc;

fn parent_path(path: &str) -> String {
    let p = std::path::Path::new(path);
    p.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn make_parent_entry(path: &str) -> FileEntry {
    FileEntry {
        name: "..".into(),
        path: parent_path(path),
        kind: EntryKind::Dir,
        size: None,
        modified: None,
        permissions: None,
    }
}

pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    queue: &TransferQueue,
    registry: &Arc<RemoteRegistry>,
    _rt_handle: &tokio::runtime::Handle,
) {
    ui.horizontal(|ui| {
        ui.heading("\u{1F4C1} Local");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("\u{1F5C4}").clicked() {
                state.local_tree_open = !state.local_tree_open;
            }
        });
    });

    let frame = egui::Frame::none().inner_margin(egui::Margin::symmetric(4.0, 2.0));
    let (_inner, dropped) = ui.dnd_drop_zone::<DragPayload, _>(frame, |ui| {
        if state.local_tree_open {
            ui.horizontal(|ui| {
                ui.separator();
                ui.vertical(|ui| {
                    render_tree(ui, state);
                });
                ui.separator();
                ui.vertical(|ui| {
                    render_content(ui, state);
                });
            });
        } else {
            render_content(ui, state);
        }
    });

    if let Some(payload_arc) = dropped {
        let payload = &*payload_arc;
        match payload {
            DragPayload::RemoteFile(remote_path, file_name, conn_id) => {
                if registry.get(conn_id).is_none() {
                    state.status_message = "Connection no longer active".into();
                    return;
                }
                let local_path_str =
                    format!("{}/{}", state.local_path.trim_end_matches('/'), file_name);
                let task = TransferTask::new(
                    TransferKind::Download,
                    conn_id.clone(),
                    local_path_str,
                    remote_path.clone(),
                    file_name.clone(),
                    0,
                );
                queue.push(task);
                state.status_message = "Download queued via drag & drop".into();
            }
            DragPayload::LocalFile(_, _) => {
                state.status_message = "Cannot drop local file onto local pane".into();
            }
        }
    }
}

fn render_tree(ui: &mut egui::Ui, state: &mut AppState) {
    let home = dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .min_scrolled_width(120.0)
        .show(ui, |ui| {
            ui.strong("Folders");
            ui.separator();

            let roots = [
                ("/", "Root"),
                (&home, "Home"),
                ("/tmp", "tmp"),
                ("/var", "var"),
                ("/etc", "etc"),
            ];

            for (root_path, label) in &roots {
                let is_current = state.local_path == *root_path
                    || state.local_path.starts_with(root_path);
                let mut btn = egui::Button::new(*label).min_size(egui::vec2(120.0, 20.0));
                if is_current {
                    btn = btn.fill(egui::Color32::from_rgb(50, 60, 80));
                }
                if ui.add(btn).clicked() {
                    state.local_path = root_path.to_string();
                    refresh_local(state);
                }
            }
        });
}

fn render_content(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        let mut path = state.local_path.clone();
        ui.add(
            TextEdit::singleline(&mut path)
                .id("local_path".into())
                .desired_width(f32::INFINITY),
        );
        if path != state.local_path {
            state.local_path = path;
            refresh_local(state);
        }
    });

    ui.separator();

    let entries = state.local_entries.clone();
    let mut selected = state.local_selected.clone();

    let FileTableResponse { double_clicked, .. } =
        file_table(ui, "local_table", &entries, &mut selected, |entry| {
            let has_connection = state
                .active_tab_ref()
                .map(|t| t.status == ConnectionStatus::Connected)
                .unwrap_or(false);
            if has_connection {
                make_local_payload(entry)
            } else {
                None
            }
        });

    state.local_selected = selected;

    if let Some(name) = double_clicked {
        if name == ".." {
            state.local_path = parent_path(&state.local_path);
        } else if let Some(entry) = state.local_entries.iter().find(|e| e.name == name)
            && entry.kind == EntryKind::Dir
        {
            state.local_path = entry.path.clone();
        }
        refresh_local(state);
    }

    ui.separator();
    ui.horizontal(|ui| {
        ui.label(format!("{} items", state.local_entries.len()));
    });
}

pub fn refresh_local(state: &mut AppState) {
    let mut entries = Vec::new();
    if state.local_path != "/" {
        entries.push(make_parent_entry(&state.local_path));
    }
    match local::list(&state.local_path) {
        Ok(mut list) => {
            entries.append(&mut list);
            state.local_entries = entries;
        }
        Err(e) => {
            state.status_message = format!("Local list error: {}", e);
        }
    }
}
