use crate::ui::state::AppState;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if ui.button("+ New Connection").clicked() {
            state.show_connect_dialog = true;
        }
    });
}
