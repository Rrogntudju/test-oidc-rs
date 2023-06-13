use anyhow::{Result, Context};
use iced_native::Subscription;
use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows::UI::ViewManagement::{UIColorType, UISettings};

#[derive(Debug, Clone)]
pub enum ModeCouleur {
    Clair,
    Sombre,
}

struct EventModeCouleur {
    settings: UISettings,
    token: EventRegistrationToken,
}

impl EventModeCouleur {
    fn new() -> Result<Self> {
        let settings = UISettings::new().context("Iniatialisation UISettings")?;
        let token = settings.ColorValuesChanged(&TypedEventHandler::new(move |settings, _| {
             Ok(())
        })).context("Initialisation ColorValuesChanged")?;
        Ok(Self { settings, token })
    }
}

impl Drop for EventModeCouleur {
    fn drop(&mut self) {
        self.settings.RemoveColorValuesChanged(self.token).unwrap_or_default();
    }
}

#[inline]
fn is_color_light(clr: &windows::UI::Color) -> bool {
    ((5 * clr.G) + (2 * clr.R) + clr.B) > (8 * 128)
}

pub fn mode_couleur() -> Result<ModeCouleur> {
    let couleur = UISettings::new()?.GetColorValue(UIColorType::Foreground)?;
    Ok(if is_color_light(&couleur) {
        ModeCouleur::Clair
    } else {
        ModeCouleur::Sombre
    })
}

pub fn stream_event_mode_couleur() -> Subscription<ModeCouleur> {
    Subscription::none()
}
