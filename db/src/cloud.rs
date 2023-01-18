use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use log::info;

use crate::schema::pis;
use crate::sql_types::*;

use crate::connection::establish_sqlite_connection;

#[derive(
    Queryable, Identifiable, Insertable, Clone, Debug, PartialEq, Default, Serialize, Deserialize,
)]
#[diesel(table_name = pis)]
pub struct Pi {
    pub id: i32,
    pub last_boot: Option<String>,
    pub hostname: String,
    pub created_dt: String,
    pub moonraker_api_url: String,
    pub mission_control_url: String,
    pub octoprint_url: String,
    pub swupdate_url: String,
    pub syncthing_url: String,
    pub preferred_dns: String,
    pub octoprint_server_id: Option<i32>,
    pub system_info_id: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, AsChangeset)]
#[diesel(table_name = pis)]
pub struct UpdatePi {
    pub last_boot: Option<String>,
    pub hostname: Option<String>,
    pub created_dt: Option<String>,
    pub moonraker_api_url: Option<String>,
    pub mission_control_url: Option<String>,
    pub octoprint_url: Option<String>,
    pub swupdate_url: Option<String>,
    pub syncthing_url: Option<String>,
    pub preferred_dns: Option<String>,
    pub octoprint_server_id: Option<i32>,
    pub system_info_id: Option<i32>,
}

impl From<printnanny_api_client::models::Pi> for UpdatePi {
    fn from(obj: printnanny_api_client::models::Pi) -> UpdatePi {
        let urls = *obj.urls;
        let preferred_dns = match obj.network_settings {
            Some(network_settings) => match network_settings.preferred_dns {
                Some(result) => result.into(),
                None => printnanny_api_client::models::PreferredDnsType::Multicast,
            },
            None => printnanny_api_client::models::PreferredDnsType::Multicast,
        }
        .to_string();
        let octoprint_server_id = match obj.octoprint_server {
            Some(octoprint_server) => Some(octoprint_server.id),
            None => None,
        };
        let system_info_id = match obj.system_info {
            Some(system_info) => Some(system_info.id),
            None => None,
        };
        UpdatePi {
            last_boot: obj.last_boot,
            hostname: None,
            created_dt: None,
            moonraker_api_url: Some(urls.moonraker_api),
            mission_control_url: Some(urls.mission_control),
            octoprint_url: Some(urls.octoprint),
            swupdate_url: Some(urls.swupdate),
            syncthing_url: Some(urls.syncthing),
            preferred_dns: Some(preferred_dns),
            octoprint_server_id,
            system_info_id,
        }
    }
}

impl From<printnanny_api_client::models::Pi> for Pi {
    fn from(obj: printnanny_api_client::models::Pi) -> Pi {
        let urls = *obj.urls;
        let preferred_dns = match obj.network_settings {
            Some(network_settings) => match network_settings.preferred_dns {
                Some(result) => result.into(),
                None => printnanny_api_client::models::PreferredDnsType::Multicast,
            },
            None => printnanny_api_client::models::PreferredDnsType::Multicast,
        };
        let octoprint_server_id = match obj.octoprint_server {
            Some(octoprint_server) => Some(octoprint_server.id),
            None => None,
        };

        let system_info_id = match obj.system_info {
            Some(system_info) => Some(system_info.id),
            None => None,
        };

        Pi {
            id: obj.id,
            last_boot: obj.last_boot,
            hostname: obj.hostname,
            created_dt: obj.created_dt,
            moonraker_api_url: urls.moonraker_api,
            mission_control_url: urls.mission_control,
            octoprint_url: urls.octoprint,
            swupdate_url: urls.swupdate,
            syncthing_url: urls.syncthing,
            preferred_dns: preferred_dns.to_string(),
            octoprint_server_id,
            system_info_id,
        }
    }
}

// #[derive(AsChangeset)]
// #[diesel(table_name = pi)]
// pub struct UpdatePi {
//     pub id: i32,
//     pub last_boot: Option<String>,
//     pub hostname: String,
//     pub sbc: String,
//     pub created_dt: String,
//     pub moonraker_api_url: String,
//     pub mission_control_url: String,
//     pub octoprint_url: String,
//     pub swupdate_url: String,
//     pub syncthing_url: String,
//     pub preferred_dns: String,
//     pub octoprint_server_id: Option<i32>,
//     pub system_info_id: Option<i32>,
// }

// impl From<printnanny_api_client::models::Pi> for UpdatePi {
//     fn from(obj: printnanny_api_client::models::Pi) -> UpdatePi {
//         let urls = *obj.urls;
//         let preferred_dns = match obj.network_settings {
//             Some(network_settings) => match network_settings.preferred_dns {
//                 Some(result) => result.into(),
//                 None => PreferredDnsType::Multicast,
//             },
//             None => PreferredDnsType::Multicast,
//         };
//         let octoprint_server_id = match obj.octoprint_server {
//             Some(octoprint_server) => Some(octoprint_server.id),
//             None => None,
//         };

//         let system_info_id = match obj.system_info {
//             Some(system_info) => Some(system_info.id),
//             None => None,
//         };

//         UpdatePi {
//             id: obj.id,
//             last_boot: obj.last_boot,
//             hostname: obj.hostname,
//             sbc: obj.sbc.to_string(),
//             created_dt: obj.created_dt,
//             moonraker_api_url: urls.moonraker_api,
//             mission_control_url: urls.mission_control,
//             octoprint_url: urls.octoprint,
//             swupdate_url: urls.swupdate,
//             syncthing_url: urls.syncthing,
//             preferred_dns,
//             octoprint_server_id,
//             system_info_id,
//         }
//     }
// }

impl Pi {
    pub fn get_id() -> Result<i32, diesel::result::Error> {
        use crate::schema::pis::dsl::*;
        let connection = &mut establish_sqlite_connection();
        let result: i32 = pis.select(id).first(connection)?;
        Ok(result)
    }
    pub fn get() -> Result<Pi, diesel::result::Error> {
        use crate::schema::pis::dsl::*;

        let connection = &mut establish_sqlite_connection();
        let result: Pi = pis.order_by(id).first::<Pi>(connection)?;
        // let result = pis.order_by(id).first(&mut connection)?;
        info!("printnanny_edge_db::cloud::Pi get {:#?}", &result);
        Ok(result)
    }
    pub fn insert(row: Pi) -> Result<(), diesel::result::Error> {
        let mut connection = establish_sqlite_connection();

        let updated = diesel::insert_into(pis::dsl::pis)
            .values(row)
            .execute(&mut connection)?;
        info!("printnanny_edge_db::cloud::Pi created {}", &updated);
        Ok(())
    }
    pub fn update(pi_id: i32, changeset: UpdatePi) -> Result<(), diesel::result::Error> {
        let mut connection = establish_sqlite_connection();
        let result = diesel::update(pis::table.filter(pis::id.eq(pi_id)))
            .set(changeset)
            .execute(&mut connection)?;
        info!("printnanny_edge_db::cloud::Pi updated {}", &result);
        Ok(())
    }
}

// #[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, Queryable, Identifiable)]
// #[diesel(table_name = printnanny_cloud_api_config)]
// pub struct PrintNannyCloudApi {
//     pub id: i32,
//     pub base_url: String,
//     pub bearer_access_token: Option<String>,
// }

// impl PrintNannyCloudApiConfig {
//     pub fn insert_ignore(
//         base_url: &str,
//         bearer_access_token: &str,
//     ) -> Result<(), diesel::result::Error> {
//         let mut connection = establish_sqlite_connection();

//         diesel::insert_or_ignore_into(printnanny_cloud_api_config::table)
//             .values((
//                 printnanny_cloud_api_config::dsl::bearer_access_token.eq(bearer_access_token),
//                 printnanny_cloud_api_config::dsl::base_url.eq(base_url),
//             ))
//             .execute(&mut connection)?;

//         Ok(())
//     }
// }