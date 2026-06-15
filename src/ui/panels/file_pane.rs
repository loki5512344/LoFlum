use crate::domain::file_entry::{EntryKind, FileEntry};
use crate::ui::drag::DragPayload;
use egui::{Grid, Id};

pub fn format_size(bytes: Option<u64>) -> String {
    match bytes {
        None => String::new(),
        Some(b) if b < 1024 => format!("{} B", b),
        Some(b) if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        Some(b) if b < 1024 * 1024 * 1024 => {
            format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
        }
        Some(b) => format!("{:.2} GB", b as f64 / (1024.0 * 1024.0 * 1024.0)),
    }
}

pub fn format_time(ts: Option<i64>) -> String {
    match ts.and_then(|t| chrono::DateTime::from_timestamp(t, 0)) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => String::new(),
    }
}

pub struct FileTableResponse {
    pub double_clicked: Option<String>,
    pub clicked: Option<String>,
}

pub fn file_table<F>(
    ui: &mut egui::Ui,
    id: &str,
    entries: &[FileEntry],
    selected: &mut Option<String>,
    make_drag_payload: F,
) -> FileTableResponse
where
    F: Fn(&FileEntry) -> Option<DragPayload>,
{
    let mut double_clicked: Option<String> = None;
    let mut clicked: Option<String> = None;

    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| match (&a.kind, &b.kind) {
        (EntryKind::Dir, EntryKind::Dir) => a.name.cmp(&b.name),
        (EntryKind::Dir, _) => std::cmp::Ordering::Less,
        (_, EntryKind::Dir) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            Grid::new(id)
                .striped(true)
                .min_col_width(60.0)
                .show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("Size");
                    ui.strong("Type");
                    ui.strong("Modified");
                    ui.end_row();

                    for entry in &sorted {
                        let is_selected = selected.as_ref() == Some(&entry.name);

                        let response = if let Some(payload) = make_drag_payload(entry) {
                            let drag_id = Id::new(("drag", id, &entry.name));
                            ui.dnd_drag_source(drag_id, payload, |ui| {
                                ui.selectable_label(is_selected, &entry.name)
                            })
                            .response
                        } else {
                            ui.selectable_label(is_selected, &entry.name)
                        };

                        if response.clicked() {
                            *selected = Some(entry.name.clone());
                            clicked = Some(entry.name.clone());
                        }
                        if response.double_clicked() {
                            double_clicked = Some(entry.name.clone());
                        }

                        ui.label(format_size(entry.size));
                        ui.label(match entry.kind {
                            EntryKind::Dir => "Dir",
                            EntryKind::File => "File",
                            EntryKind::Symlink => "Link",
                        });
                        ui.label(format_time(entry.modified));
                        ui.end_row();
                    }
                });
        });

    FileTableResponse {
        double_clicked,
        clicked,
    }
}
