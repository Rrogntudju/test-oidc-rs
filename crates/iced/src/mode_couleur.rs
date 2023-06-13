use anyhow::Result;
use iced_native::Subscription;
use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows::UI::ViewManagement::{UIColorType, UISettings};

#[derive(Debug, Clone)]
pub enum ModeCouleur {
    Claire,
    Sombre,
}

struct EventModeCouleur {
    settings: UISettings,
    token: EventRegistrationToken,
}

impl EventModeCouleur {
    fn new() -> Self {
        let settings = UISettings::new().unwrap();
        let token = settings.ColorValuesChanged(&TypedEventHandler::new(move |settings, _| Ok(()))).unwrap();
        Self { settings, token }
    }
}

impl Drop for EventModeCouleur {
    fn drop(&mut self) {
        self.settings.RemoveColorValuesChanged(self.token).unwrap();
    }
}

#[inline]
fn is_color_light(clr: &windows::UI::Color) -> bool {
    ((5 * clr.G) + (2 * clr.R) + clr.B) > (8 * 128)
}

pub fn mode_couleur() -> Result<ModeCouleur> {
    let couleur = UISettings::new()?.GetColorValue(UIColorType::Foreground)?;
    Ok(if is_color_light(&couleur) {
        ModeCouleur::Claire
    } else {
        ModeCouleur::Sombre
    })
}

pub fn stream_event_mode_couleur() -> Subscription<ModeCouleur> {
    Subscription::none()
}
