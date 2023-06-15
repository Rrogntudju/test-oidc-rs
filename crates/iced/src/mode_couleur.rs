use anyhow::{Context, Result};
use iced::futures::executor::block_on;
use iced_native::{subscription, Subscription};
use tokio::sync::mpsc::{channel, Receiver, Sender};
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
    fn new(tx: Sender<Result<ModeCouleur>>) -> Result<Self> {
        let settings = UISettings::new().context("Initialisation UISettings")?;
        let token = settings
            .ColorValuesChanged(&TypedEventHandler::new(move |settings: &Option<UISettings>, _| {
                let settings: &UISettings = settings.as_ref().unwrap();
                let _ = block_on(async { tx.send(mode_couleur(settings)).await });
                Ok(())
            }))
            .context("Initialisation ColorValuesChanged")?;
        Ok(Self { settings, token })
    }
}

impl Drop for EventModeCouleur {
    fn drop(&mut self) {
        let _ = self.settings.RemoveColorValuesChanged(self.token);
    }
}

#[inline]
fn is_color_light(clr: &windows::UI::Color) -> bool {
   (0.299 * clr.R as f32 + 0.587 * clr.G as f32 + 0.114 * clr.B as f32) > 128.0 // https://www.w3.org/TR/AERT/#color-contrast
}

// https://learn.microsoft.com/fr-fr/windows/apps/desktop/modernize/apply-windows-themes
fn mode_couleur(settings: &UISettings) -> Result<ModeCouleur> {
    let couleur = settings.GetColorValue(UIColorType::Foreground)?;
    Ok(if is_color_light(&couleur) {
        ModeCouleur::Sombre
    } else {
        ModeCouleur::Clair
    })
}

pub fn stream_event_mode_couleur() -> Subscription<Result<ModeCouleur, String>> {
    let (tx, rx) = channel::<Result<ModeCouleur>>(10);
    let revoker = match EventModeCouleur::new(tx) {
        Ok(revoker) => revoker,
        Err(e) => {
            eprintln!("{e:#}");
            return Subscription::none();
        }
    };

    enum State {
        Init((Receiver<Result<ModeCouleur>>, EventModeCouleur)),
        Receiving((Receiver<Result<ModeCouleur>>, EventModeCouleur)),
    }

    struct EventModeCouleurId;

    subscription::unfold(std::any::TypeId::of::<EventModeCouleurId>(), State::Init((rx, revoker)), |state| async {
        match state {
            State::Init((rx, revoker)) => {
                let mode = mode_couleur(&revoker.settings);
                (mode.map_err(|e| format!("{e:#}")), State::Receiving((rx, revoker)))
            }
            State::Receiving((mut rx, revoker)) => match rx.recv().await {
                Some(mode) => (mode.map_err(|e| format!("{e:#}")), State::Receiving((rx, revoker))),
                None => {
                    let erreur: Result<ModeCouleur, String> = Err("Échec du changement de mode couleur".to_string());
                    (erreur, State::Receiving((rx, revoker)))
                }
            },
        }
    })
}
