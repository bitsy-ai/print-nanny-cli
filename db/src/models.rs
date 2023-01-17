use diesel::prelude::*;
use diesel::{AsExpression, FromSqlRow};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use printnanny_api_client;

#[derive(Debug)]
pub struct EnumError {
    msg: String,
    status: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, DbEnum)]
pub enum Status {
    Pending,
    InProgress,
    Done,
}

impl Default for Status {
    fn default() -> Self {
        Status::Pending
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, Queryable)]
pub struct VideoRecording {
    pub id: String,
    pub recording_status: Status,
    pub recording_start: Option<u64>,
    pub recording_end: Option<u64>,
    pub recording_file_name: String,
    pub gcode_file_name: Option<String>,
    pub cloud_sync_status: Status,
    pub cloud_sync_start: Option<u64>,
    pub cloud_sync_end: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Deserialize, Serialize)]
pub enum SbcEnum {
    Rpi4,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, Queryable)]
pub struct PiUrls {
    pub id: i32,
    pub moonraker_api: String,
    pub mission_control: String,
    pub octoprint: String,
    pub swupdate: String,
    pub syncthing: String,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, Queryable)]
pub struct Pi {
    pub id: i32,
    pub last_boot: String,
    pub hostname: String,
    pub sbc: SbcEnum,
    pub created_dt: String,
}

impl From<printnanny_api_client::Pi> for Pi {
    fn from(obj: printnanny_api_client::models::Pi) -> Pi {
        Pi {
            id: obj.id,
            last_boot: obj.last_boot,
            hostname: obj.hostname,
            sbc: obj.sbc,
            created_dt: obj.created_dt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Deserialize, Serialize)]
pub enum PreferredDnsType {
    #[serde(rename = "multicast")]
    Multicast,
    #[serde(rename = "tailscale")]
    Tailscale,
}

impl From<printnanny_api_client::PreferredDnsType> for PreferredDnsType {
    fn from(obj: printnanny_api_client::models::PreferredDnsType) -> PreferredDnsType {
        match obj {
            printnanny_api_client::models::PreferredDnsType::Multicast => {
                PreferredDnsType::Multicast
            }
            printnanny_api_client::models::PreferredDnsType::Tailscale => {
                PreferredDnsType::Tailscale
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, Queryable)]
pub struct NetworkSettings {
    pub id: i32,
    pub updated_dt: String,
    pub preferred_dns: PreferredDnsType,
}

impl From<printnanny_api_client::models::NetworkSettings> for NetworkSettings {
    fn from(obj: printnanny_api_client::models::NetworkSettings) -> NetworkSettings {
        NetworkSettings {
            id: obj.id,
            updated_dt: obj.updated_dt,
            preferred_dns: obj.preferred_dns.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, Queryable)]
pub struct User {
    pub email: String,
    pub id: i32,
    #[serde(rename = "first_name", skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(rename = "last_name", skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
}

impl From<printnanny_api_client::models::User> for User {
    fn from(obj: printnanny_api_client::models::User) -> User {
        User {
            id: obj.id,
            email: obj.email,
            first_name: obj.first_name,
            last_name: obj.last_name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, Queryable)]
pub struct PrintNannyCloudApiConfig {
    pub base_url: String,
    pub bearer_access_token: Option<String>,
}
