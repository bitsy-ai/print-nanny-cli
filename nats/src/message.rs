use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use log::info;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use printnanny_dbus;
use printnanny_dbus::zbus;

use printnanny_services::printer_mgmt::octoprint::OctoPrintSettings;
use printnanny_services::settings::{PrintNannySettings, SettingsFormat};
use printnanny_services::vcs::VersionControlledSettings;

use crate::error::{ErrorMsg, ResultMsg};

#[async_trait]
pub trait NatsRequestReplyHandler {
    type Request: Serialize + DeserializeOwned + Clone + Debug;
    type Reply: Serialize + DeserializeOwned + Clone + Debug;
    async fn handle(&self) -> Result<Self::Reply>;
}

// pi.dbus.org.freedesktop.systemd1.Manager.StartUnit
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerStartUnitRequest {
    name: String,
    // mode: String, // "replace", "fail", "isolate", "ignore-dependencies", or "ignore-requirements" - but only "replace" mode is used by here, so omitting for simplicity
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerStartUnitReply {
    request: SystemdManagerStartUnitRequest,
    job: zbus::zvariant::OwnedObjectPath,
}

#[async_trait]
impl NatsRequestReplyHandler for SystemdManagerStartUnitRequest {
    type Request = SystemdManagerStartUnitRequest;
    type Reply = SystemdManagerStartUnitReply;

    async fn handle(&self) -> Result<Self::Reply> {
        let connection = zbus::Connection::system().await?;
        let proxy = printnanny_dbus::systemd1::manager::ManagerProxy::new(&connection).await?;
        let job = proxy.start_unit(&self.name, "replace").await?;
        let reply = Self::Reply {
            job,
            request: self.clone(),
        };
        Ok(reply)
    }
}

//  pi.dbus.org.freedesktop.systemd1.Manager.RestartUnit
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerRestartUnitRequest {
    name: String,
    // mode: String, // "replace", "fail", "isolate", "ignore-dependencies", or "ignore-requirements" - but only "replace" mode is used by here, so omitting for simplicity
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerRestartUnitReply {
    request: SystemdManagerRestartUnitRequest,
    job: zbus::zvariant::OwnedObjectPath,
}

#[async_trait]
impl NatsRequestReplyHandler for SystemdManagerRestartUnitRequest {
    type Request = SystemdManagerRestartUnitRequest;
    type Reply = SystemdManagerRestartUnitReply;
    async fn handle(&self) -> Result<Self::Reply> {
        let connection = zbus::Connection::system().await?;
        let proxy = printnanny_dbus::systemd1::manager::ManagerProxy::new(&connection).await?;
        let job = proxy.restart_unit(&self.name, "replace").await?;
        let reply = Self::Reply {
            job,
            request: self.clone(),
        };
        Ok(reply)
    }
}

//  pi.dbus.org.freedesktop.systemd1.Manager.StopUnit
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerStopUnitRequest {
    name: String,
    // mode: String, // "replace", "fail", "isolate", "ignore-dependencies", or "ignore-requirements" - but only "replace" mode is used by here, so omitting for simplicity
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerStopUnitReply {
    request: SystemdManagerStopUnitRequest,
    job: zbus::zvariant::OwnedObjectPath,
}

#[async_trait]
impl NatsRequestReplyHandler for SystemdManagerStopUnitRequest {
    type Request = SystemdManagerStopUnitRequest;
    type Reply = SystemdManagerStopUnitReply;
    async fn handle(&self) -> Result<Self::Reply> {
        let connection = zbus::Connection::system().await?;
        let proxy = printnanny_dbus::systemd1::manager::ManagerProxy::new(&connection).await?;
        let job = proxy.stop_unit(&self.name, "replace").await?;
        let reply = Self::Reply {
            job: job,
            request: self.clone(),
        };
        Ok(reply)
    }
}

//  pi.dbus.org.freedesktop.systemd1.Manager.EnableUnit
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerEnableUnitRequest {
    files: Vec<String>,
    // mode: String, // "replace", "fail", "isolate", "ignore-dependencies", or "ignore-requirements" - but only "replace" mode is used by here, so omitting for simplicity
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerEnableUnitReply {
    request: SystemdManagerEnableUnitRequest,
    changes: Vec<(String, String, String)>,
}

#[async_trait]
impl NatsRequestReplyHandler for SystemdManagerEnableUnitRequest {
    type Request = SystemdManagerEnableUnitRequest;
    type Reply = SystemdManagerEnableUnitReply;
    async fn handle(&self) -> Result<Self::Reply> {
        let connection = zbus::Connection::system().await?;
        let proxy = printnanny_dbus::systemd1::manager::ManagerProxy::new(&connection).await?;
        let files: Vec<&str> = self.files.iter().map(|s| s.as_str()).collect();
        let (_enablement_info, changes) = proxy.enable_unit_files(&files, false, false).await?;
        let reply = Self::Reply {
            changes,
            request: self.clone(),
        };
        Ok(reply)
    }
}

//  pi.dbus.org.freedesktop.systemd1.Manager.DisableUnit
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerDisableUnitRequest {
    files: Vec<String>,
    // mode: String, // "replace", "fail", "isolate", "ignore-dependencies", or "ignore-requirements" - but only "replace" mode is used by here, so omitting for simplicity
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerDisableUnitReply {
    request: SystemdManagerDisableUnitRequest,
    changes: Vec<(String, String, String)>,
}

#[async_trait]
impl NatsRequestReplyHandler for SystemdManagerDisableUnitRequest {
    type Request = SystemdManagerDisableUnitRequest;
    type Reply = SystemdManagerDisableUnitReply;
    async fn handle(&self) -> Result<Self::Reply> {
        let connection = zbus::Connection::system().await?;
        let proxy = printnanny_dbus::systemd1::manager::ManagerProxy::new(&connection).await?;
        let files: Vec<&str> = self.files.iter().map(|s| s.as_str()).collect();
        let changes = proxy.disable_unit_files(&files, false).await?;
        let reply = Self::Reply {
            changes,
            request: self.clone(),
        };
        Ok(reply)
    }
}

//  pi.dbus.org.freedesktop.systemd1.Manager.ReloadUnit
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerReloadUnitRequest {
    name: String, // mode: String, // "replace", "fail", "isolate", "ignore-dependencies", or "ignore-requirements" - but only "replace" mode is used by here, so omitting for simplicity
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemdManagerReloadUnitReply {
    request: SystemdManagerReloadUnitRequest,
    job: zbus::zvariant::OwnedObjectPath,
}

#[async_trait]
impl NatsRequestReplyHandler for SystemdManagerReloadUnitRequest {
    type Request = SystemdManagerReloadUnitRequest;
    type Reply = SystemdManagerReloadUnitReply;

    async fn handle(&self) -> Result<Self::Reply> {
        let connection = zbus::Connection::system().await?;
        let proxy = printnanny_dbus::systemd1::manager::ManagerProxy::new(&connection).await?;
        let job = proxy.restart_unit(&self.name, "replace").await?;
        let reply = Self::Reply {
            job,
            request: self.clone(),
        };
        Ok(reply)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConnectCloudAccountRequest {
    email: String,
    api_token: String,
    api_uri: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConnectCloudAccountReply {
    request: ConnectCloudAccountRequest,
    detail: String,
}

#[async_trait]
impl NatsRequestReplyHandler for ConnectCloudAccountRequest {
    type Request = ConnectCloudAccountRequest;
    type Reply = ConnectCloudAccountReply;

    async fn handle(&self) -> Result<Self::Reply> {
        let settings = PrintNannySettings::new()?;
        settings
            .connect_cloud_account(self.api_uri.clone(), self.api_token.clone())
            .await?;

        let res = Self::Reply {
            request: self.clone(),
            detail: format!(
                "Success! Connected PrintNanny Cloud account belonging to {}",
                self.email
            ),
        };
        Ok(res)
    }
}

//  pi.settings.gst_pipeline.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GstPipelineSettingsLoadRequest {
    format: SettingsFormat,
}

//  pi.settings.gst_pipeline.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GstPipelineSettingsLoadReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for GstPipelineSettingsLoadRequest {
    type Request = GstPipelineSettingsLoadRequest;
    type Reply = GstPipelineSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.gst_pipeline.apply
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GstPipelineSettingsApplyRequest {
    parent_commit: String,
    format: SettingsFormat,
}

//  pi.settings.gst_pipeline.apply
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GstPipelineSettingsApplyReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
    commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for GstPipelineSettingsApplyRequest {
    type Request = GstPipelineSettingsLoadRequest;
    type Reply = GstPipelineSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.gst_pipeline.revert
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GstPipelineSettingsRevertRequest {
    commit: String,
}

//  pi.settings.gst_pipeline.revert
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GstPipelineSettingsRevertReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for GstPipelineSettingsRevertRequest {
    type Request = GstPipelineSettingsLoadRequest;
    type Reply = GstPipelineSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.moonraker.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoonrakerSettingsLoadRequest {
    format: SettingsFormat,
}

//  pi.settings.moonraker.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoonrakerSettingsLoadReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for MoonrakerSettingsLoadRequest {
    type Request = MoonrakerSettingsLoadRequest;
    type Reply = MoonrakerSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.moonraker.apply
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoonrakerSettingsApplyRequest {
    parent_commit: String,
    format: SettingsFormat,
}

//  pi.settings.moonraker.apply
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoonrakerSettingsApplyReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
    commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for MoonrakerSettingsApplyRequest {
    type Request = MoonrakerSettingsLoadRequest;
    type Reply = MoonrakerSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.moonraker.revert
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoonrakerSettingsRevertRequest {
    commit: String,
}

//  pi.settings.moonraker.revert
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoonrakerSettingsRevertReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for MoonrakerSettingsRevertRequest {
    type Request = MoonrakerSettingsLoadRequest;
    type Reply = MoonrakerSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.klipper.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlipperSettingsLoadRequest {
    format: SettingsFormat,
}

//  pi.settings.klipper.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlipperSettingsLoadReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for KlipperSettingsLoadRequest {
    type Request = KlipperSettingsLoadRequest;
    type Reply = KlipperSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.klipper.apply
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlipperSettingsApplyRequest {
    parent_commit: String,
    format: SettingsFormat,
}

//  pi.settings.klipper.apply
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlipperSettingsApplyReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
    commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for KlipperSettingsApplyRequest {
    type Request = KlipperSettingsLoadRequest;
    type Reply = KlipperSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.klipper.revert
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlipperSettingsRevertRequest {
    commit: String,
}

//  pi.settings.klipper.revert
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlipperSettingsRevertReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for KlipperSettingsRevertRequest {
    type Request = KlipperSettingsLoadRequest;
    type Reply = KlipperSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.gst_pipeline.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OctoPrintSettingsLoadRequest {
    format: SettingsFormat,
}

//  pi.settings.octoprint.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OctoPrintSettingsLoadReply {
    request: OctoPrintSettingsLoadRequest,
    data: String,
    format: SettingsFormat,
    parent_commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for OctoPrintSettingsLoadRequest {
    type Request = OctoPrintSettingsLoadRequest;
    type Reply = OctoPrintSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        let settings = PrintNannySettings::new()?;

        let parent_commit = settings.octoprint.get_git_parent_commit()?.to_string();
        let data = settings.octoprint.read_settings()?;

        Ok(Self::Reply {
            request: self.clone(),
            parent_commit,
            data,
            format: SettingsFormat::Yaml,
        })
    }
}

//  pi.settings.octoprint.apply
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OctoPrintSettingsApplyRequest {
    data: String,
    parent_commit: String,
    format: SettingsFormat,
}

//  pi.settings.octoprint.apply
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OctoPrintSettingsApplyReply {
    request: OctoPrintSettingsApplyRequest,
    data: String,
    format: SettingsFormat,
    parent_commit: String,
    commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for OctoPrintSettingsApplyRequest {
    type Request = OctoPrintSettingsApplyRequest;
    type Reply = OctoPrintSettingsApplyReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

//  pi.settings.octoprint.revert
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OctoPrintSettingsRevertRequest {
    commit: String,
}

//  pi.settings.octoprint.revert
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OctoPrintSettingsRevertReply {
    data: String,
    format: SettingsFormat,
    parent_commit: String,
}

#[async_trait]
impl NatsRequestReplyHandler for OctoPrintSettingsRevertRequest {
    type Request = OctoPrintSettingsLoadRequest;
    type Reply = OctoPrintSettingsLoadReply;

    async fn handle(&self) -> Result<Self::Reply> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "subject")]
pub enum NatsRequest {
    // pi.command.*
    #[serde(rename = "pi.command.connect_printnanny_cloud_account")]
    ConnectPrintNannyCloudRequest(SystemdManagerStopUnitRequest),

    // pi.dbus.org.freedesktop.systemd1.*
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.DisableUnit")]
    SystemdManagerDisableUnitRequest(SystemdManagerDisableUnitRequest),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.EnableUnit")]
    SystemdManagerEnableUnitRequest(SystemdManagerEnableUnitRequest),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.ReloadUnit")]
    SystemdManagerReloadUnitRequest(SystemdManagerReloadUnitRequest),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.RestartUnit")]
    SystemdManagerRestartUnitRequest(SystemdManagerRestartUnitRequest),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.StartUnit")]
    SystemdManagerStartUnitRequest(SystemdManagerStartUnitRequest),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.StopUnit")]
    SystemdManagerStopUnitRequest(SystemdManagerStopUnitRequest),

    // pi.settings.*
    #[serde(rename = "pi.settings.gst_pipeline.load")]
    GstPipelineSettingsLoadRequest(GstPipelineSettingsLoadRequest),
    #[serde(rename = "pi.settings.gst_pipeline.apply")]
    GstPipelineSettingsApplyRequest(GstPipelineSettingsApplyRequest),
    #[serde(rename = "pi.settings.gst_pipeline.revert")]
    GstPipelineSettingsRevertRequest(GstPipelineSettingsRevertRequest),

    #[serde(rename = "pi.settings.klipper.load")]
    KlipperSettingsLoadRequest(KlipperSettingsLoadRequest),
    #[serde(rename = "pi.settings.klipper.apply")]
    KlipperSettingsApplyRequest(KlipperSettingsApplyRequest),
    #[serde(rename = "pi.settings.klipper.revert")]
    KlipperSettingsRevertRequest(KlipperSettingsRevertRequest),

    #[serde(rename = "pi.settings.moonraker.load")]
    MoonrakerSettingsLoadRequest(MoonrakerSettingsLoadRequest),
    #[serde(rename = "pi.settings.moonraker.apply")]
    MoonrakerSettingsApplyRequest(MoonrakerSettingsApplyRequest),
    #[serde(rename = "pi.settings.moonraker.revert")]
    MoonrakerSettingsRevertRequest(MoonrakerSettingsRevertRequest),

    #[serde(rename = "pi.settings.octoprint.load")]
    OctoPrintSettingsLoadRequest(OctoPrintSettingsLoadRequest),
    #[serde(rename = "pi.settings.octoprint.apply")]
    OctoPrintSettingsApplyRequest(OctoPrintSettingsApplyRequest),
    #[serde(rename = "pi.settings.octoprint.revert")]
    OctoPrintSettingsRevertRequest(OctoPrintSettingsRevertRequest),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "subject")]
pub enum NatsReply {
    // pi.command.*
    #[serde(rename = "pi.command.connect_printnanny_cloud_account")]
    ConnectPrintNannyCloudReply(SystemdManagerStopUnitReply),

    // pi.dbus.org.freedesktop.systemd1.*
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.DisableUnit")]
    SystemdManagerDisableUnitReply(SystemdManagerDisableUnitReply),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.EnableUnit")]
    SystemdManagerEnableUnitReply(SystemdManagerEnableUnitReply),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.ReloadUnit")]
    SystemdManagerReloadUnitReply(SystemdManagerReloadUnitReply),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.RestartUnit")]
    SystemdManagerRestartUnitReply(SystemdManagerRestartUnitReply),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.StartUnit")]
    SystemdManagerStartUnitReply(SystemdManagerStartUnitReply),
    #[serde(rename = "pi.dbus.org.freedesktop.systemd1.Manager.StopUnit")]
    SystemdManagerStopUnitReply(SystemdManagerStopUnitReply),

    // pi.settings.*
    #[serde(rename = "pi.settings.gst_pipeline.load")]
    GstPipelineSettingsLoadReply(GstPipelineSettingsLoadReply),
    #[serde(rename = "pi.settings.gst_pipeline.apply")]
    GstPipelineSettingsApplyReply(GstPipelineSettingsApplyReply),
    #[serde(rename = "pi.settings.gst_pipeline.revert")]
    GstPipelineSettingsRevertReply(GstPipelineSettingsRevertReply),

    #[serde(rename = "pi.settings.klipper.load")]
    KlipperSettingsLoadReply(KlipperSettingsLoadReply),
    #[serde(rename = "pi.settings.klipper.apply")]
    KlipperSettingsApplyReply(KlipperSettingsApplyReply),
    #[serde(rename = "pi.settings.klipper.revert")]
    KlipperSettingsRevertReply(KlipperSettingsRevertReply),

    #[serde(rename = "pi.settings.moonraker.load")]
    MoonrakerSettingsLoadReply(MoonrakerSettingsLoadReply),
    #[serde(rename = "pi.settings.moonraker.apply")]
    MoonrakerSettingsApplyReply(MoonrakerSettingsApplyReply),
    #[serde(rename = "pi.settings.moonraker.revert")]
    MoonrakerSettingsRevertReply(MoonrakerSettingsRevertReply),

    #[serde(rename = "pi.settings.octoprint.load")]
    OctoPrintSettingsLoadReply(OctoPrintSettingsLoadReply),
    #[serde(rename = "pi.settings.octoprint.apply")]
    OctoPrintSettingsApplyReply(OctoPrintSettingsApplyReply),
    #[serde(rename = "pi.settings.octoprint.revert")]
    OctoPrintSettingsRevertReply(OctoPrintSettingsRevertReply),
}

//  pi.settings.octoprint.load
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NatsError<T> {
    request: T,
    error: String,
}

#[async_trait]
impl NatsRequestReplyHandler for NatsRequest {
    type Request = NatsRequest;
    type Reply = NatsReply;

    async fn handle(&self) -> Result<NatsReply> {
        let reply = match self {
            NatsRequest::SystemdManagerDisableUnitRequest(request) => {
                match request.handle().await {
                    Ok(r) => Ok(NatsReply::SystemdManagerDisableUnitReply(r)),
                    Err(e) => Err(e),
                }
            }
            NatsRequest::SystemdManagerEnableUnitRequest(request) => match request.handle().await {
                Ok(r) => Ok(NatsReply::SystemdManagerEnableUnitReply(r)),
                Err(e) => Err(e),
            },
            NatsRequest::SystemdManagerReloadUnitRequest(request) => match request.handle().await {
                Ok(r) => Ok(NatsReply::SystemdManagerReloadUnitReply(r)),
                Err(e) => Err(e),
            },
            NatsRequest::SystemdManagerRestartUnitRequest(request) => {
                match request.handle().await {
                    Ok(r) => Ok(NatsReply::SystemdManagerRestartUnitReply(r)),
                    Err(e) => Err(e),
                }
            }
            NatsRequest::SystemdManagerStartUnitRequest(request) => match request.handle().await {
                Ok(r) => Ok(NatsReply::SystemdManagerStartUnitReply(r)),
                Err(e) => Err(e),
            },
            NatsRequest::SystemdManagerStopUnitRequest(request) => match request.handle().await {
                Ok(r) => Ok(NatsReply::SystemdManagerStopUnitReply(r)),
                Err(e) => Err(e),
            },
            NatsRequest::ConnectPrintNannyCloudRequest(_) => todo!(),
            NatsRequest::GstPipelineSettingsLoadRequest(_) => todo!(),
            NatsRequest::GstPipelineSettingsApplyRequest(_) => todo!(),
            NatsRequest::GstPipelineSettingsRevertRequest(_) => todo!(),
            NatsRequest::KlipperSettingsLoadRequest(_) => todo!(),
            NatsRequest::KlipperSettingsApplyRequest(_) => todo!(),
            NatsRequest::KlipperSettingsRevertRequest(_) => todo!(),
            NatsRequest::MoonrakerSettingsLoadRequest(_) => todo!(),
            NatsRequest::MoonrakerSettingsApplyRequest(_) => todo!(),
            NatsRequest::MoonrakerSettingsRevertRequest(_) => todo!(),
            NatsRequest::OctoPrintSettingsLoadRequest(request) => match request.handle().await {
                Ok(r) => Ok(NatsReply::OctoPrintSettingsLoadReply(r)),
                Err(e) => Err(e),
            },
            NatsRequest::OctoPrintSettingsApplyRequest(request) => match request.handle().await {
                Ok(r) => Ok(NatsReply::OctoPrintSettingsApplyReply(r)),
                Err(e) => Err(e),
            },
            NatsRequest::OctoPrintSettingsRevertRequest(_) => todo!(),
        };

        info!("Sending NatsReply: {:?}", reply);
        reply
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use test_log::test;

    use printnanny_services::settings::jail::Jail;

    fn make_settings_repo() -> Jail {
        let mut jail = Jail::new().unwrap();
        let output = jail.directory().to_str().unwrap();

        jail.create_file(
            "PrintNannySettingsTest.toml",
            &format!(
                r#"
            [paths]
            settings_dir = "{output}/settings"
            log_dir = "{output}/log"
            "#,
                output = &output
            ),
        )
        .unwrap();
        jail.set_env("PRINTNANNY_SETTINGS", "PrintNannySettingsTest.toml");
        let settings = PrintNannySettings::new().unwrap();
        settings.git_clone().unwrap();
        jail
    }

    #[test(tokio::test)] // async test
    async fn test_load_octoprint_settings() {
        let jail = make_settings_repo();

        let settings = PrintNannySettings::new().unwrap();

        let expected =
            fs::read_to_string(settings.paths.settings_dir.join("octoprint/octoprint.yaml"))
                .unwrap();

        let request = OctoPrintSettingsLoadRequest {
            format: SettingsFormat::Yaml,
        };

        let natsrequest = NatsRequest::OctoPrintSettingsLoadRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::OctoPrintSettingsLoadReply(reply) = natsreply {
            assert_eq!(reply.request, request);
            assert_eq!(reply.data, expected);
        } else {
            panic!("Expected NatsReply::OctoPrintSettingsLoadReply")
        }
        drop(jail)
    }

    // #[test(tokio::test)] // async test
    async fn test_apply_octoprint_settings() {
        let jail = make_settings_repo();

        let settings = PrintNannySettings::new().unwrap();

        let parent_commit = settings.octoprint.get_git_parent_commit().unwrap();

        let before =
            fs::read_to_string(settings.paths.settings_dir.join("octoprint/octoprint.yaml"))
                .unwrap();
        let expected = r#"
        ---
        server:
          commands:
            systemShutdownCommand: sudo shutdown -h now
            systemRestartCommand: sudo shutdown -r now
            serverRestartCommand: sudo systemctl restart octoprint.service
        
        api:
          disabled: true
        
        system:
          actions:
            - name: Start PrintNanny Cam
              action: printnanny_cam_start
              command: sudo systemctl restart printnanny-vision.service
            - name: Stop PrintNanny Cam
              action: printnanny_cam_stop
              command: sudo systemctl stop printnanny-vision.service
        events:
          subscriptions:
            - command: sudo systemctl start printnanny-vision.service
              debug: false
              event: plugin_octoprint_nanny_vision_start
              type: system
              enabled: true
            - command: sudo systemctl stop printnanny-vision.service
              enabled: true
              debug: false
              event: plugin_octoprint_nanny_vision_stop
              type: system
        
        webcam:
          stream: /printnanny-hls/playlist.m3u8
        "#;

        let request = OctoPrintSettingsApplyRequest {
            format: SettingsFormat::Yaml,
            data: expected.to_string(),
            parent_commit: parent_commit.to_string(),
        };

        let natsrequest = NatsRequest::OctoPrintSettingsApplyRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::OctoPrintSettingsApplyReply(reply) = natsreply {
            assert_eq!(reply.request, request);
            assert_eq!(reply.data, expected);
        } else {
            panic!("Expected NatsReply::OctoPrintSettingsLoadReply")
        }
        drop(jail)
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_disable_unit_ok() {
        let request = SystemdManagerDisableUnitRequest {
            files: vec!["octoprint.service".into()],
        };
        let natsrequest = NatsRequest::SystemdManagerDisableUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::SystemdManagerDisableUnitReply(reply) = natsreply {
            assert_eq!(reply.request, request);
        } else {
            panic!("Expected NatsReply::SystemdManagerDisableUnitReply")
        }
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_disable_unit_error() {
        let request = SystemdManagerDisableUnitRequest {
            files: vec!["doesnotexist.service".into()],
        };
        let natsrequest = NatsRequest::SystemdManagerDisableUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::SystemdManagerDisableUnitReply(reply) = natsreply {
            assert_eq!(reply.request, request);
        } else {
            panic!("Expected NatsReply::SystemdManagerDisableUnitReply")
        }
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_enable_unit_ok() {
        let request = SystemdManagerEnableUnitRequest {
            files: vec!["octoprint.service".into()],
        };
        let natsrequest = NatsRequest::SystemdManagerEnableUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::SystemdManagerEnableUnitReply(reply) = natsreply {
            assert_eq!(reply.request, request);
        } else {
            panic!("Expected NatsReply::SystemdManagerEnableUnitReply")
        }
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_enable_unit_error() {
        let request = SystemdManagerEnableUnitRequest {
            files: vec!["doesnotexist.service".into()],
        };
        let natsrequest = NatsRequest::SystemdManagerEnableUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await;
        assert!(natsreply.is_err());
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_start_unit_ok() {
        let request = SystemdManagerStartUnitRequest {
            name: "octoprint.service".into(),
        };
        let natsrequest = NatsRequest::SystemdManagerStartUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::SystemdManagerStartUnitReply(reply) = natsreply {
            assert_eq!(reply.request, request);
        } else {
            panic!("Expected NatsReply::SystemdManagerStartUnitReply")
        }
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_start_unit_error() {
        let request = SystemdManagerStartUnitRequest {
            name: "doesnotexist.service".into(),
        };
        let natsrequest = NatsRequest::SystemdManagerStartUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await;
        assert!(natsreply.is_err());
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_restart_unit_ok() {
        let request = SystemdManagerRestartUnitRequest {
            name: "octoprint.service".into(),
        };
        let natsrequest = NatsRequest::SystemdManagerRestartUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::SystemdManagerRestartUnitReply(reply) = natsreply {
            assert_eq!(reply.request, request);
        } else {
            panic!("Expected NatsReply::SystemdManagerRestartUnitReply")
        }
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_restart_unit_error() {
        let request = SystemdManagerRestartUnitRequest {
            name: "doesnotexist.service".into(),
        };
        let natsrequest = NatsRequest::SystemdManagerRestartUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await;
        assert!(natsreply.is_err());
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_stop_unit_ok() {
        let request = SystemdManagerStopUnitRequest {
            name: "octoprint.service".into(),
        };
        let natsrequest = NatsRequest::SystemdManagerStopUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::SystemdManagerStopUnitReply(reply) = natsreply {
            assert_eq!(reply.request, request);
        } else {
            panic!("Expected NatsReply::SystemdManagerStopUnitReply")
        }
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_stop_unit_error() {
        let request = SystemdManagerStopUnitRequest {
            name: "doesnotexist.service".into(),
        };
        let natsrequest = NatsRequest::SystemdManagerStopUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await;
        assert!(natsreply.is_err());
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_reload_unit_ok() {
        let request = SystemdManagerReloadUnitRequest {
            name: "octoprint.service".into(),
        };
        let natsrequest = NatsRequest::SystemdManagerReloadUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await.unwrap();
        if let NatsReply::SystemdManagerReloadUnitReply(reply) = natsreply {
            assert_eq!(reply.request, request);
            // assert_eq!(reply.status, "ok")
        } else {
            panic!("Expected NatsReply::SystemdManagerReloadUnitReply")
        }
    }

    #[cfg(feature = "systemd")]
    #[test(tokio::test)] // async test
    async fn test_dbus_systemd_manager_reload_unit_error() {
        let request = SystemdManagerReloadUnitRequest {
            name: "doesnotexist.service".into(),
        };
        let natsrequest = NatsRequest::SystemdManagerReloadUnitRequest(request.clone());
        let natsreply = natsrequest.handle().await;
        assert!(natsreply.is_err());
    }

    // fn test_gst_pipeline_settings_update_handler() {
    //     figment::Jail::expect_with(|jail| {
    //         let output = jail.directory().join("test.toml");

    //         jail.create_file(
    //             "test.toml",
    //             &format!(
    //                 r#"

    //             [tflite_model]
    //             tensor_width = 720
    //             "#,
    //             ),
    //         )?;
    //         jail.set_env("PRINTNANNY_GST_CONFIG", output.display());

    //         let src = "https://cdn.printnanny.ai/gst-demo-videos/demo_video_1.mp4";

    //         let request_toml = r#"
    //             video_src = "https://cdn.printnanny.ai/gst-demo-videos/demo_video_1.mp4"
    //             video_src_type = "Uri"
    //         "#;

    //         let request = SettingsRequest {
    //             data: request_toml.into(),
    //             format: SettingsFormat::Toml,
    //             subject: SettingsSubject::GstPipeline,
    //             pre_save: vec![],
    //             post_save: vec![],
    //         };

    //         let res = request.handle();

    //         assert_eq!(res.status, ReplyStatus::Ok);

    //         let saved_config = PrintNannyGstPipelineConfig::new().unwrap();
    //         assert_eq!(saved_config.video_src, src);
    //         assert_eq!(saved_config.video_src_type, VideoSrcType::Uri);
    //         Ok(())
    //     });
    // }

    // fn test_gst_octoprint_settings_update_handler() {
    //     figment::Jail::expect_with(|jail| {
    //         let output = jail.directory().join("test.toml");

    //         // configuration reference: https://docs.octoprint.org/en/master/configuration/config_yaml.html
    //         jail.create_file(
    //             "config.yaml",
    //             &format!(
    //                 r#"
    //             feature:
    //                 # Whether to enable the gcode viewer in the UI or not
    //                 gCodeVisualizer: true
    //             "#,
    //             ),
    //         )?;
    //         jail.set_env("OCTOPRINT_SETTINGS_FILE", output.display());

    //         let content = r#"
    //         feature:
    //             # Whether to enable the gcode viewer in the UI or not
    //             gCodeVisualizer: false
    //         "#;

    //         let request = SettingsRequest {
    //             data: content.into(),
    //             format: SettingsFormat::Yaml,
    //             subject: SettingsSubject::OctoPrint,
    //             pre_save: vec![],
    //             post_save: vec![],
    //         };

    //         let res = request.handle();

    //         assert_eq!(res.status, ReplyStatus::Ok);

    //         let saved_config = OctoPrintSettings::default().read_settings().unwrap();
    //         assert_eq!(saved_config, content);
    //         Ok(())
    //     });
    // }
}
