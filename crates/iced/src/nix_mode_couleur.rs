use anyhow::{Context, Result};
use iced::{subscription, Subscription};
use iced_futures::futures;
use zbus::{dbus_proxy, zvariant::OwnedValue};
use iced_futures::futures::StreamExt;

static APPEARANCE: &str = "org.freedesktop.appearance";
static SCHEME: &str = "color-scheme";

#[derive(Debug, Clone)]
pub enum ModeCouleur {
    Clair,
    Sombre,
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

fn build_portal_settings_proxy<'a>() -> Result<PortalSettingsProxy<'a>> {
    let proxy = futures::executor::block_on(async {
        let connection = zbus::ConnectionBuilder::session()?.build().await?;
        PortalSettingsProxy::new(&connection).await.context("building proxy")
    })?;

    Ok(proxy)
}

fn get_mode_couleur(value: OwnedValue) -> Result<ModeCouleur, String> {
    match value.downcast_ref::<u32>() {
        Some(1) => Ok(ModeCouleur::Sombre),
        Some(_) => Ok(ModeCouleur::Clair),
        None => Err(format!("get_mode_couleur: u32 attendu mais reçu {value:#?}")),
    }
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

    enum State<'a> {
        Init(PortalSettingsProxy<'a>),
        Receiving((PortalSettingsProxy<'a>, SettingChangedStream<'a>)),
        End,
    }

    subscription::run_with_id(std::any::TypeId::of::<EventModeCouleurId>(), {
        futures::stream::unfold(State::Init(proxy), |state| async {
            match state {
                State::Init(proxy) => match proxy.Read(APPEARANCE, SCHEME).await {
                    Ok(value) => {
                        let mode = get_mode_couleur(value);
                        match proxy.receive_SettingChanged().await {
                            Ok(setting_changed) => Some((mode.map_err(|e| format!("{e:#}")), State::Receiving((proxy, setting_changed)))),
                            Err(e) => Some((Err(format!("{e:#}")), State::End)),
                        }
                    }
                    Err(e) => Some((Err(format!("{e:#}")), State::End)),
                },
                State::Receiving((proxy, setting_changed)) => match setting_changed.next().await {
                    Some(signal) => Some((mode.map_err(|e| format!("{e:#}")), State::Receiving((rx, revoker)))),
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
