use crate::domain::connection::ConnectionStatus;
use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, state: &AppState) {
    ui.horizontal(|ui| {
        ui.label(&state.status_message);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let queued = state.queue_tasks.len();
            ui.label(format!("Queue: {}", queued));
            ui.separator();
            let connected = state
                .tabs
                .iter()
                .filter(|t| t.status == ConnectionStatus::Connected)
                .count();
            let total = state.tabs.len();
            ui.label(format!("Connections: {}/{}", connected, total));
        });
    });
}
