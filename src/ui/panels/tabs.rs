use crate::domain::connection::ConnectionStatus;
use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        let mut to_close: Option<usize> = None;

        for (i, tab) in state.tabs.iter().enumerate() {
            let is_active = i == state.active_tab;

            let dot = match &tab.status {
                ConnectionStatus::Connected => "\u{25CF}",
                ConnectionStatus::Disconnected => "\u{25CB}",
                ConnectionStatus::Connecting => "\u{25D0}",
                ConnectionStatus::Error(_) => "\u{2716}",
            };

            let label = format!("{} {}  ", dot, tab.label);

            let mut tab_btn = egui::Button::new(&label)
                .min_size(egui::vec2(60.0, 28.0));
            if is_active {
                tab_btn = tab_btn.fill(egui::Color32::from_rgb(60, 60, 70));
            }
            if ui.add(tab_btn).clicked() {
                state.active_tab = i;
            }

            if ui.small_button("x").clicked() {
                to_close = Some(i);
            }
        }

        if let Some(idx) = to_close {
            state.tabs.remove(idx);
            if state.active_tab >= state.tabs.len() && !state.tabs.is_empty() {
                state.active_tab = state.tabs.len() - 1;
            }
        }

        if ui.button("+").clicked() {
            state.show_connect_dialog = true;
        }
    });
}
