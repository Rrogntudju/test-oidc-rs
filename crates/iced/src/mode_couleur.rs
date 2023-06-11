use iced_native::Subscription;

#[derive(Debug, Clone)]
pub enum ModeCouleur {
    Claire,
    Sombre
}

pub fn stream_event_mode_couleur() -> Subscription<ModeCouleur> {
    Subscription::none()
}