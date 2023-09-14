use anyhow::{Context, Result};
use iced::{subscription, Subscription};
use iced_futures::futures;
use zbus::{dbus_proxy, zvariant::OwnedValue};

static APPEARANCE: &str = "org.freedesktop.appearance";
static SCHEME: &str = "color-scheme";

#[derive(Debug, Clone)]
pub enum ModeCouleur {
    Clair,
    Sombre,
}

impl From<OwnedValue> for ModeCouleur {
    fn from(value: OwnedValue) -> Self {
        match value.downcast_ref::<u32>() {
            Some(1) => Self::Sombre,
            _ => Self::Clair,
        }
    }
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait PortalSettings {
    fn Read(&self, namespace: &str, key: &str) -> zbus::Result<OwnedValue>;

    #[dbus_proxy(signal)]
    fn SettingChanged(&self, namespace: &str, key: &str, value: OwnedValue) -> Result<()>;
}

fn build_portal_settings_proxy<'c>() -> Result<PortalSettingsProxy<'c>> {
    let proxy = futures::executor::block_on(async {
        let connection = zbus::ConnectionBuilder::session()?.build().await?;
        PortalSettingsProxy::new(&connection).await.context("building proxy")
    })?;

    Ok(proxy)
}

pub fn stream_event_mode_couleur() -> Subscription<Result<ModeCouleur, String>> {
    struct EventModeCouleurId;

    let proxy = match build_portal_settings_proxy() {
        Ok(proxy) => proxy,
        Err(e) => {
            return subscription::run_with_id(
                std::any::TypeId::of::<EventModeCouleurId>(),
                futures::stream::once(async move { Err(format!("{e:#}")) }),
            )
        }
    };

    enum State<'c> {
        Init(PortalSettingsProxy<'c>),
        Receiving((PortalSettingsProxy<'c>, )),
        End,
    }

    subscription::run_with_id(std::any::TypeId::of::<EventModeCouleurId>(), {
        futures::stream::unfold(State::Init(proxy), |state| async {
            match state {
                State::Init(proxy) => {
                    let value = futures::executor::block_on(async { proxy.Read(APPEARANCE, SCHEME).await });

                    Some((mode.map_err(|e| format!("{e:#}")), State::Receiving((rx, revoker))))
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
