use anyhow::{Result, Context};
use iced_native::Subscription;
use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows::UI::ViewManagement::{UIColorType, UISettings};
use tokio::sync::oneshot::{self, Sender};

#[derive(Debug, Clone, Default)]
pub enum ModeCouleur {
    #[default]
    Clair,
    Sombre,
}

struct EventModeCouleur {
    settings: UISettings,
    token: EventRegistrationToken,
}

impl EventModeCouleur {
    fn new(sender: Sender<ModeCouleur>) -> Result<Self> {
        let settings = UISettings::new().context("Initialisation UISettings")?;
        let token = settings.ColorValuesChanged(&TypedEventHandler::new(move |settings: &Option<UISettings>, _| {
            let settings: &UISettings = settings.as_ref().unwrap();
            let mode = mode_couleur_(settings).unwrap_or_default();
            sender.send(mode).unwrap_or_default();
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

fn mode_couleur_(settings: &UISettings) -> Result<ModeCouleur> {
    let couleur = settings.GetColorValue(UIColorType::Foreground)?;
    Ok(if is_color_light(&couleur) {
        ModeCouleur::Clair
    } else {
        ModeCouleur::Sombre
    })
}

pub fn mode_couleur() -> Result<ModeCouleur> {
    let settings = &UISettings::new()?;
    mode_couleur_(settings)
}

pub fn stream_event_mode_couleur() -> Subscription<ModeCouleur> {
    let (sender, receiver) = oneshot::channel::<ModeCouleur>();
    let revoker = EventModeCouleur::new(sender).
    Subscription::none()
}
