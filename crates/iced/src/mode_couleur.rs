use anyhow::{Context, Result};
use iced_native::Subscription;
use std::sync::mpsc::{self, Sender};
use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows::UI::ViewManagement::{UIColorType, UISettings};

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
    fn new(sender: Sender<Result<ModeCouleur>>) -> Result<Self> {
        let settings = UISettings::new().context("Initialisation UISettings")?;
        let token = settings
            .ColorValuesChanged(&TypedEventHandler::new(move |settings: &Option<UISettings>, _| {
                let settings: &UISettings = settings.as_ref().unwrap();
                sender.send(mode_couleur_(settings)).unwrap_or_default();
                Ok(())
            }))
            .context("Initialisation ColorValuesChanged")?;
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
    let settings = &UISettings::new().context("Initialisation UISettings")?;
    mode_couleur_(settings)
}

pub fn stream_event_mode_couleur() -> Subscription<ModeCouleur> {
    let (sender, receiver) = mpsc::channel::<Result<ModeCouleur>>();
    let revoker = match EventModeCouleur::new(sender) {
        Ok(revoker) => revoker,
        Err(e) => {
            eprintln!("{e:#}");
            return Subscription::none()
        }
    };
    Subscription::none()
}
