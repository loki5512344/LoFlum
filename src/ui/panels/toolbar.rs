use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if ui.button("+ New Connection").clicked() {
            state.show_connect_dialog = true;
        }

        ui.separator();

        if ui.button("Bookmarks").clicked() {
            state.show_bookmarks = !state.show_bookmarks;
            state.show_history = false;
        }

        if ui.button("History").clicked() {
            state.show_history = !state.show_history;
            state.show_bookmarks = false;
        }

        if state.show_bookmarks {
            egui::ComboBox::from_id_source("bookmarks_menu")
                .selected_text("Bookmarks")
                .width(150.0)
                .show_ui(ui, |ui| {
                    for bm in &state.bookmarks.clone() {
                        if ui.button(&bm.name).clicked() {
                            state.local_path = bm.path.clone();
                            crate::ui::panels::local_pane::refresh_local(state);
                            state.show_bookmarks = false;
                        }
                    }
                });
        }

        if state.show_history {
            egui::ComboBox::from_id_source("history_menu")
                .selected_text("History")
                .width(200.0)
                .show_ui(ui, |ui| {
                    if state.history.is_empty() {
                        ui.label("No history yet");
                    } else {
                        for entry in &state.history.clone() {
                            let label = format!("{}@{}:{} [{}]", entry.user, entry.host, entry.port, entry.time);
                            if ui.button(&label).clicked() {
                                state.connect_host = entry.host.clone();
                                state.connect_user = entry.user.clone();
                                state.connect_port = entry.port.to_string();
                                state.show_connect_dialog = true;
                                state.show_history = false;
                            }
                        }
                    }
                });
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let queued = state.queue_tasks.len();
            if queued > 0 {
                ui.label(format!("Queue: {}", queued));
            }
        });
    });
}
