use crate::domain::connection::ConnectionStatus;
use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if state.tabs.is_empty() {
            ui.label("No connections");
            if ui.button("+ New Connection").clicked() {
                state.show_connect_dialog = true;
            }
            return;
        }

        let mut to_close: Option<usize> = None;

        for (i, tab) in state.tabs.iter().enumerate() {
            let is_active = i == state.active_tab;

            let dot = match &tab.status {
                ConnectionStatus::Connected => "\u{25CF}",
                ConnectionStatus::Disconnected => "\u{25CB}",
                ConnectionStatus::Connecting => "\u{25D0}",
                ConnectionStatus::Error(_) => "\u{2716}",
            };

            let label = format!("{} {}", dot, tab.label);

            let mut tab_button = egui::Button::new(&label);
            if is_active {
                tab_button = tab_button.fill(egui::Color32::from_rgb(60, 60, 80));
            }
            let response = ui.add(tab_button);
            if response.clicked() {
                state.active_tab = i;
            }

            if ui.small_button("\u{2715}").clicked() {
                to_close = Some(i);
            }
        }

        if let Some(idx) = to_close {
            state.tabs.remove(idx);
            if state.active_tab >= state.tabs.len() && !state.tabs.is_empty() {
                state.active_tab = state.tabs.len() - 1;
            }
        }

        ui.separator();

        if ui.button("+").clicked() {
            state.show_connect_dialog = true;
        }
    });
}
