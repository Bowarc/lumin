#[derive(Debug, Clone)]
pub struct State<Inner> {
    pub text: String,
    pub color: eframe::egui::Color32,
    pub inner: Inner,
}

impl State<crate::ui::BackgroundPreviewActivationState> {
    pub fn set_not_sent(&mut self) {
        self.text = "Not yet sent".to_string(); // Initializing
        self.color = eframe::egui::Color32::RED;

        self.inner = crate::ui::BackgroundPreviewActivationState::NotConnected
    }

    pub fn set_connected(&mut self, id: crate::id::ID) {
        self.text = "Connected".to_string(); // Initializing
        self.color = eframe::egui::Color32::GREEN;
        self.inner = crate::ui::BackgroundPreviewActivationState::Running { id }
    }
}

impl Default for State<crate::ui::BackgroundPreviewActivationState> {
    fn default() -> Self {
        let mut o = Self {

            text: "Not yet synched".into(),
            color: eframe::egui::Color32::RED,
            inner: crate::ui::BackgroundPreviewActivationState::NotConnected,
        };
        o.set_not_sent();
        o
    }
}
