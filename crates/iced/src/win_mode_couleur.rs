use anyhow::{Context, Result};
use iced::Subscription;
use iced_futures::futures;
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

// https://learn.microsoft.com/fr-fr/windows/apps/desktop/modernize/apply-windows-themes
impl EventModeCouleur {
    fn new(tx: Sender<Result<ModeCouleur>>) -> Result<Self> {
        let settings = UISettings::new().context("Initialisation UISettings")?;
        let token = settings
            .ColorValuesChanged(&TypedEventHandler::new(move |settings: &Option<UISettings>, _| {
                let settings: &UISettings = settings.as_ref().unwrap();
                let _ = futures::executor::block_on(async { tx.send(mode_couleur(settings)).await });
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

enum State {
    Init,
    Receiving((Receiver<Result<ModeCouleur>>, EventModeCouleur)),
    End,
}

#[inline]
fn is_color_light(clr: &windows::UI::Color) -> bool {
    // https://www.w3.org/TR/AERT/#color-contrast
    (0.299 * clr.R as f32 + 0.587 * clr.G as f32 + 0.114 * clr.B as f32) > 128.0
}

fn mode_couleur(settings: &UISettings) -> Result<ModeCouleur> {
    let couleur = settings.GetColorValue(UIColorType::Foreground).context("GetColorValue")?;
    Ok(if is_color_light(&couleur) {
        ModeCouleur::Sombre
    } else {
        ModeCouleur::Clair
    })
}

pub fn stream_event_mode_couleur() -> Subscription<Result<ModeCouleur, String>> {
    struct EventModeCouleurId;

    Subscription::run_with_id(std::any::TypeId::of::<EventModeCouleurId>(), {
        futures::stream::unfold(State::Init, |state| async {
            match state {
                State::Init => {
                    let (tx, rx) = channel::<Result<ModeCouleur>>(10);
                    match EventModeCouleur::new(tx) {
                        Ok(revoker) => {
                            let mode = mode_couleur(&revoker.settings);
                            Some((mode.map_err(|e| format!("{e:#}")), State::Receiving((rx, revoker))))
                        }
                        Err(e) => Some((Err(format!("{e:#}")), State::End)),
                    }
                }
                State::Receiving((mut rx, revoker)) => match rx.recv().await {
                    Some(mode) => Some((mode.map_err(|e| format!("{e:#}")), State::Receiving((rx, revoker)))),
                    None => {
                        let erreur: Result<ModeCouleur, String> = Err("Échec du changement de mode couleur".to_string());
                        Some((erreur, State::End))
                    }
                },
                State::End => None,
            }
        })
    })
}
