use crate::domain::connection::ConnectionStatus;
use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(&state.status_message);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let queued = state.queue_tasks.len();
            let queue_label = if state.show_queue {
                format!("Queue ({}) \u{25BC}", queued)
            } else {
                format!("Queue ({}) \u{25B2}", queued)
            };
            if ui.button(queue_label).clicked() {
                state.show_queue = !state.show_queue;
            }
            ui.separator();
            let connected = state
                .tabs
                .iter()
                .filter(|t| t.status == ConnectionStatus::Connected)
                .count();
            let total = state.tabs.len();
            ui.label(format!("{}/{}", connected, total));
        });
    });
}
