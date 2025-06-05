use anyhow::{Context, Result};
use iced::futures::StreamExt;
use iced::Subscription;
use iced_futures::futures;
use zbus::{proxy, zvariant::OwnedValue, Connection};

const APPEARANCE: &str = "org.freedesktop.appearance";
const SCHEME: &str = "color-scheme";

#[derive(Debug, Clone)]
pub enum ModeCouleur {
    Clair,
    Sombre,
}

#[proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait PortalSettings {
    async fn read(&self, namespace: &str, key: &str) -> zbus::Result<OwnedValue>;

    #[zbus(signal)]
    async fn setting_changed(&self, namespace: &str, key: &str, value: OwnedValue) -> zbus::Result<()>;
}

enum State {
    Init,
    Receiving(SettingChangedStream),
    End,
}

async fn build_portal_settings_proxy<'a>() -> Result<PortalSettingsProxy<'a>> {
    let connection = Connection::session().await.context("build connection")?;
    PortalSettingsProxy::new(&connection).await.context("build proxy")
}

fn get_mode_couleur(value: &OwnedValue) -> Result<ModeCouleur, String> {
    match value.downcast_ref::<u32>() {
        Ok(1) => Ok(ModeCouleur::Sombre),
        Ok(0) => Ok(ModeCouleur::Clair),
        Ok(x) => Err(format!("get_mode_couleur: 0 ou 1 attendu mais reçu {x}")),
        Err(e) => Err(format!("get_mode_couleur: {e}")),
    }
}

pub fn stream_event_mode_couleur() -> Subscription<Result<ModeCouleur, String>> {
    struct EventModeCouleurId;

    Subscription::run_with_id(std::any::TypeId::of::<EventModeCouleurId>(), {
        futures::stream::unfold(State::Init, |state| async {
            match state {
                State::Init => match build_portal_settings_proxy().await {
                    Ok(proxy) => match proxy.read(APPEARANCE, SCHEME).await {
                        Ok(value) => {
                            let mode = get_mode_couleur(&value);
                            match proxy.receive_setting_changed().await {
                                Ok(setting_changed) => Some((mode.map_err(|e| format!("{e:#}")), State::Receiving(setting_changed))),
                                Err(e) => Some((Err(format!("{e:#}")), State::End)),
                            }
                        }
                        Err(e) => Some((Err(format!("{e:#}")), State::End)),
                    },
                    Err(e) => Some((Err(format!("{e:#}")), State::End)),
                },
                State::Receiving(mut setting_changed) => loop {
                    match setting_changed.next().await {
                        Some(signal) => match signal.args() {
                            Ok(args) => {
                                if *args.namespace() == APPEARANCE && *args.key() == SCHEME {
                                    let mode = get_mode_couleur(args.value());
                                    break Some((mode.map_err(|e| format!("{e:#}")), State::Receiving(setting_changed)));
                                }
                            }
                            Err(e) => break Some((Err(format!("{e:#}")), State::End)),
                        },
                        None => {
                            let erreur: Result<ModeCouleur, String> = Err("Échec du changement de mode couleur".to_string());
                            break Some((erreur, State::End));
                        }
                    }
                },
                State::End => None,
            }
        })
    })
}
