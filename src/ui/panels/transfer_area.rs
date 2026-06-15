use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label("\u{1F4E4} Transfer area");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let queued = state.queue_tasks.len();
            if queued > 0 {
                ui.label(format!("{} files", queued));
            }
        });
    });
}
