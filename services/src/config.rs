use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use clap::{ArgEnum, PossibleValue};
use figment::providers::{Env, Format, Json, Serialized, Toml};
use figment::value::{Dict, Map};
use figment::{Figment, Metadata, Profile, Provider};
use glob::glob;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::PrintNannyConfigError;
use super::keys::PrintNannyKeys;
use printnanny_api_client::models;

pub const OCTOPRINT_DIR: &str = "/home/octoprint/.octoprint";
pub const PRINTNANNY_CONFIG_FILENAME: &str = "default.toml";
pub const PRINTNANNY_CONFIG_DEFAULT: &str = "/etc/printnanny/default.toml";

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum ConfigFormat {
    Json,
    Toml,
}

impl ConfigFormat {
    pub fn possible_values() -> impl Iterator<Item = PossibleValue<'static>> {
        ConfigFormat::value_variants()
            .iter()
            .filter_map(ArgEnum::to_possible_value)
    }
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl std::str::FromStr for ConfigFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("Invalid variant: {}", s))
    }
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct CmdConfig {
//     pub cmd: PathBuf,
//     pub queue_dir: String,
//     pub success_dir: String,
//     pub error_dir: String,
// }

// impl Default for CmdConfig {
//     fn default() -> Self {
//         Self {
//             queue_dir: "/var/run/printnanny/cmd/queue".into(),
//             success_dir: "/var/run/printnanny/cmd/success".into(),
//             error_dir: "/var/run/printnanny/cmd/error".into(),
//         }
//     }
// }

// impl CmdConfig {
//     pub fn enqueue(&self, event: models::PolymorphicCommand) {
//         let (event_id, event_name) = match &event {
//             models::PolymorphicCommand::WebRtcCommand(e) => (e.id, e.event_name.to_string()),
//         };
//         let filename = format!("{}/{}_{}", self.queue_dir, event_name, event_id);
//         let result = serde_json::to_writer(
//             &File::create(&filename).expect(&format!("Failed to create file {}", &filename)),
//             &event,
//         );
//         match result {
//             Ok(_) => info!(
//                 "Wrote event={:?} to file={:?} to await processing",
//                 event, filename
//             ),
//             Err(e) => error!("Failed to serialize event {:?} with error {:?}", event, e),
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashConfig {
    pub base_url: String,
    pub base_path: String,
    pub port: i32,
}

impl Default for DashConfig {
    fn default() -> Self {
        let hostname = sys_info::hostname().unwrap_or("localhost".to_string());
        Self {
            base_url: format!("http://{}/", hostname),
            base_path: "/".into(),
            port: 9001,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MQTTConfig {
    pub cmd: PathBuf,
    pub cipher: String,
    pub keepalive: u64,
    pub ca_certs: Vec<String>,
}

impl Default for MQTTConfig {
    fn default() -> Self {
        Self {
            cmd: "/var/run/printnanny/cmd".into(),
            ca_certs: vec![
                "/etc/ca-certificates/gtsltsr.crt".into(),
                "/etc/ca-certificates/GSR4.crt".into(),
            ],
            cipher: "secp256r1".into(),
            keepalive: 300, // seconds
        }
    }
}

impl MQTTConfig {
    pub fn cmd_queue(&self) -> PathBuf {
        self.cmd.join("queue")
    }
    pub fn cmd_error(&self) -> PathBuf {
        self.cmd.join("error")
    }
    pub fn cmd_success(&self) -> PathBuf {
        self.cmd.join("success")
    }
    pub fn enqueue_cmd(&self, event: models::PolymorphicCommand) {
        let (event_id, event_name) = match &event {
            models::PolymorphicCommand::WebRtcCommand(e) => (e.id, e.event_name.to_string()),
        };
        let filename = format!("{:?}/{}_{}", self.cmd_queue(), event_name, event_id);
        let result = serde_json::to_writer(
            &File::create(&filename).expect(&format!("Failed to create file {}", &filename)),
            &event,
        );
        match result {
            Ok(_) => info!(
                "Wrote event={:?} to file={:?} to await processing",
                event, filename
            ),
            Err(e) => error!("Failed to serialize event {:?} with error {:?}", event, e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintNannyCloudProxy {
    pub hostname: String,
    pub base_path: String,
    pub url: String,
}

impl Default for PrintNannyCloudProxy {
    fn default() -> Self {
        let hostname = sys_info::hostname().unwrap_or("localhost".to_string());
        let base_path = "/printnanny-cloud".into();
        let url = format!("http://{}{}", hostname, base_path);
        Self {
            hostname,
            base_path,
            url,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PrintNannyPaths {
    pub etc: PathBuf,
    pub confd: PathBuf,
    pub events_socket: PathBuf,
    pub issue_txt: PathBuf,
    pub log: PathBuf,
    pub octoprint: PathBuf,
    pub run: PathBuf,
}

impl Default for PrintNannyPaths {
    fn default() -> Self {
        // /etc is mounted as an r/w overlay fs
        let etc: PathBuf = "/etc/printnanny".into();
        let confd: PathBuf = "/etc/printnanny/conf.d".into();
        let issue_txt: PathBuf = "/boot/issue.txt".into();
        let run: PathBuf = "/var/run/printnanny".into();
        let log: PathBuf = "/var/log/printnanny".into();
        let events_socket = run.join("events.socket").into();
        let octoprint = OCTOPRINT_DIR.into();
        Self {
            etc,
            confd,
            run,
            issue_txt,
            log,
            events_socket,
            octoprint,
        }
    }
}

impl PrintNannyPaths {
    pub fn data(&self) -> PathBuf {
        self.etc.join("data")
    }
    pub fn octoprint_venv(&self) -> PathBuf {
        self.octoprint.join("venv")
    }

    pub fn octoprint_pip(&self) -> PathBuf {
        self.octoprint_venv().join("bin/pip")
    }

    pub fn octoprint_python(&self) -> PathBuf {
        self.octoprint_venv().join("bin/pip")
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PrintNannyConfig {
    pub printnanny_cloud_proxy: PrintNannyCloudProxy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<models::Device>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<models::User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloudiot_device: Option<models::CloudiotDevice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub janus_edge_stream: Option<models::JanusEdgeStream>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub janus_cloud_stream: Option<models::JanusCloudStream>,
    pub paths: PrintNannyPaths,
    pub api: models::PrintNannyApiConfig,
    pub dash: DashConfig,
    pub mqtt: MQTTConfig,
    pub keys: PrintNannyKeys,
}

const FACTORY_RESET: [&'static str; 7] = [
    "api",
    "cloudiot_device",
    "device",
    "janus_edge",
    "janus_cloud",
    "octoprint_install",
    "user",
];

impl Default for PrintNannyConfig {
    fn default() -> Self {
        let api = models::PrintNannyApiConfig {
            base_path: "https://printnanny.ai".into(),
            bearer_access_token: None,
            static_url: "https://printnanny.ai/static/".into(),
            dashboard_url: "https://printnanny.ai/dashboard/".into(),
        };

        let paths = PrintNannyPaths::default();
        let mqtt = MQTTConfig::default();
        let dash = DashConfig::default();
        let printnanny_cloud_proxy = PrintNannyCloudProxy::default();
        let keys = PrintNannyKeys::default();
        PrintNannyConfig {
            api,
            dash,
            mqtt,
            paths,
            printnanny_cloud_proxy,
            keys,
            cloudiot_device: None,
            device: None,
            user: None,
            janus_cloud_stream: None,
            janus_edge_stream: None,
        }
    }
}

impl PrintNannyConfig {
    // See example: https://docs.rs/figment/latest/figment/index.html#extracting-and-profiles
    // Note the `nested` option on both `file` providers. This makes each
    // top-level dictionary act as a profile
    pub fn new() -> figment::error::Result<Self> {
        let figment = Self::figment();
        let result = figment.extract()?;
        info!("Initialized config {:?}", result);
        Ok(result)
    }

    pub fn find_value(key: &str) -> Result<figment::value::Value, figment::Error> {
        let figment = Self::figment();
        figment.find_value(key)
    }

    // intended for use with Rocket's figmment
    pub fn from_figment(figment: Figment) -> Figment {
        figment.merge(Self::figment())
    }

    pub fn figment() -> Figment {
        let result = Figment::from(Self {
            // profile,
            ..Self::default()
        })
        .merge(Toml::file(Env::var_or(
            "PRINTNANNY_CONFIG",
            PRINTNANNY_CONFIG_DEFAULT,
        )))
        .merge(Env::prefixed("PRINTNANNY_").global());

        let path: String = result
            .find_value("paths.confd")
            .unwrap()
            .deserialize::<String>()
            .unwrap();

        let toml_glob = format!("{}/*.toml", &path);
        let json_glob = format!("{}/*.json", &path);

        let result = Self::read_path_glob::<Json>(&json_glob, result);
        let result = Self::read_path_glob::<Toml>(&toml_glob, result);
        info!("Finalized PrintNannyConfig: \n {:?}", result);
        result
    }

    fn read_path_glob<T: 'static + figment::providers::Format>(
        pattern: &str,
        figment: Figment,
    ) -> Figment {
        info!("Merging config from {}", &pattern);
        let mut result = figment;
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    info!("Merging config from {:?}", &path);
                    result = result.clone().merge(T::file(path))
                }
                Err(e) => error!("{:?}", e),
            }
        }
        result
    }

    pub fn try_factory_reset(&self) -> Result<(), PrintNannyConfigError> {
        // for each key/value pair in FACTORY_RESET, remove file
        for key in FACTORY_RESET.iter() {
            let filename = format!("{}.toml", key);
            let filename = self.paths.data().join(filename);
            fs::remove_file(&filename)?;
            info!("Removed {} data {:?}", key, filename);
        }
        Ok(())
    }

    /// Save FACTORY_RESET field as <field>.toml Figment fragments
    ///
    /// # Panics
    ///
    /// If serialization or fs write fails, prints an error message indicating the failure and
    /// panics. For a version that doesn't panic, use [`PrintNannyConfig::try_save_by_key()`].
    pub fn save_by_key(&self) {
        unimplemented!()
    }

    /// Save FACTORY_RESET field as <field>.toml Figment fragments
    ///
    /// If serialization or fs write fails, prints an error message indicating the failure
    pub fn try_save_by_key(&self, key: &str) -> Result<PathBuf, PrintNannyConfigError> {
        let filename = format!("{}.toml", key);
        let filename = self.paths.confd.join(filename);
        self.try_save_fragment(key, &filename)?;
        info!("Saved config fragment: {:?}", &filename);
        Ok(filename)
    }

    pub fn try_save_fragment(
        &self,
        key: &str,
        filename: &PathBuf,
    ) -> Result<(), PrintNannyConfigError> {
        let content = match key {
            "api" => toml::Value::try_from(figment::util::map! { key => &self.api}),
            "cloudiot_device" => {
                toml::Value::try_from(figment::util::map! { key => &self.cloudiot_device})
            }
            "device" => toml::Value::try_from(figment::util::map! {key => &self.device }),
            "janus_cloud_stream" => {
                toml::Value::try_from(figment::util::map! {key =>  &self.janus_cloud_stream })
            }
            "janus_edge_stream" => {
                toml::Value::try_from(figment::util::map! {key =>  &self.janus_edge_stream })
            }
            "user" => toml::Value::try_from(figment::util::map! {key =>  &self.user }),
            _ => {
                warn!("try_save_fragment received unhandled key={:?} - serializing entire PrintNannyConfig", key);
                toml::Value::try_from(self)
            }
        }?;
        info!("Saving {}.toml to {:?}", &key, &filename);
        fs::write(&filename, content.to_string())?;
        info!("Wrote {} to {:?}", key, filename);
        Ok(())
    }

    /// Save FACTORY_RESET fields as <field>.toml Figment fragments
    ///
    /// If extraction fails, prints an error message indicating the failure
    ///
    pub fn try_save(&self) -> Result<(), PrintNannyConfigError> {
        // for each key/value pair in FACTORY_RESET vec, write a separate .toml
        for key in FACTORY_RESET.iter() {
            self.try_save_by_key(key)?;
        }
        Ok(())
    }

    // Save ::Default() to output file
    pub fn try_init(
        &self,
        filename: &str,
        format: &ConfigFormat,
    ) -> Result<(), PrintNannyConfigError> {
        let content: String = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(self)?,
            ConfigFormat::Toml => toml::ser::to_string_pretty(self)?,
        };
        fs::write(&filename, content.to_string())?;
        Ok(())
    }

    /// Save FACTORY_RESET fields as <field>.toml Figment fragments
    ///
    /// # Panics
    ///
    /// If extraction fails, prints an error message indicating the failure and
    /// panics. For a version that doesn't panic, use [`PrintNannyConfig::try_save()`].
    ///
    pub fn save(&self) {
        unimplemented!()
    }

    /// Extract a `Config` from `provider`, panicking if extraction fails.
    ///
    /// # Panics
    ///
    /// If extraction fails, prints an error message indicating the failure and
    /// panics. For a version that doesn't panic, use [`Config::try_from()`].
    ///
    /// # Example
    pub fn from<T: Provider>(provider: T) -> Self {
        Self::try_from(provider).unwrap_or_else(|e| {
            error!("{:?}", e);
            panic!("aborting due to configuration error(s)")
        })
    }

    /// Attempts to extract a `Config` from `provider`, returning the result.
    ///
    /// # Example
    pub fn try_from<T: Provider>(provider: T) -> figment::error::Result<Self> {
        let figment = Figment::from(provider);
        let config = figment.extract::<Self>()?;
        Ok(config)
    }

    // Parse /etc/os-release into Map
    pub fn os_release(&self) -> Result<HashMap<String, Value>, std::io::Error> {
        let content = fs::read_to_string("/etc/os-release")?;
        let mut map = HashMap::<String, Value>::new();
        let lines = content.split("\n");
        for line in (lines).step_by(1) {
            if line.contains("=") {
                let mut pair = line.split("=");
                let key = pair.nth(0).unwrap_or("unknown").to_string();
                let value = pair
                    .nth(0)
                    .unwrap_or("unknown")
                    .replace("\"", "")
                    .to_string();
                map.insert(key, Value::from(value));
            }
        }
        info!("Parsed Map from /etc/os-release: {:?}", map);
        Ok(map)
    }
}

impl Provider for PrintNannyConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("PrintNannyConfig")
    }

    fn data(&self) -> figment::error::Result<Map<Profile, Dict>> {
        let map: Map<Profile, Dict> = Serialized::defaults(self).data()?;
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test_log::test]
    fn test_paths() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "PrintNanny.toml",
                r#"
                profile = "default"

                [paths]
                install = "/opt/printnanny/default"
                data = "/opt/printnanny/default/data"
                octoprint = "/home/leigh/projects/printnanny-cli/.tmp/test"

                
                [api]
                base_path = "https://print-nanny.com"
                "#,
            )?;
            let figment = PrintNannyConfig::figment();
            let config: PrintNannyConfig = figment.extract()?;
            assert_eq!(
                config.paths.octoprint_venv(),
                PathBuf::from("/home/leigh/projects/printnanny-cli/.tmp/test/venv")
            );
            Ok(())
        });
    }
    #[test_log::test]
    fn test_env_merged() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                PRINTNANNY_CONFIG_FILENAME,
                r#"

                [paths]
                install = "/opt/printnanny/default"
                data = "/opt/printnanny/default/data"

                
                [api]
                base_path = "https://print-nanny.com"
                "#,
            )?;
            jail.set_env("PRINTNANNY_CONFIG", PRINTNANNY_CONFIG_FILENAME);
            let config = PrintNannyConfig::new()?;
            assert_eq!(
                config.api,
                models::PrintNannyApiConfig {
                    base_path: "https://print-nanny.com".into(),
                    bearer_access_token: None,
                    static_url: "https://printnanny.ai/static/".into(),
                    dashboard_url: "https://printnanny.ai/dashboard/".into(),
                }
            );
            jail.set_env("PRINTNANNY_API.BEARER_ACCESS_TOKEN", "secret");
            let figment = PrintNannyConfig::figment();
            let config: PrintNannyConfig = figment.extract()?;
            assert_eq!(
                config.api,
                models::PrintNannyApiConfig {
                    base_path: "https://print-nanny.com".into(),
                    bearer_access_token: Some("secret".into()),
                    static_url: "https://printnanny.ai/static/".into(),
                    dashboard_url: "https://printnanny.ai/dashboard/".into(),
                }
            );
            Ok(())
        });
    }

    #[test_log::test]
    fn test_custom_confd() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "Local.toml",
                r#"
                profile = "local"

                [paths]
                confd = ".tmp/"
                
                [api]
                base_path = "http://aurora:8000"
                "#,
            )?;
            jail.set_env("PRINTNANNY_CONFIG", "Local.toml");

            let figment = PrintNannyConfig::figment();
            let config: PrintNannyConfig = figment.extract()?;

            let base_path = "http://aurora:8000".into();
            assert_eq!(config.paths.confd, PathBuf::from(".tmp/"));
            assert_eq!(config.api.base_path, base_path);

            assert_eq!(
                config.api,
                models::PrintNannyApiConfig {
                    base_path: base_path,
                    bearer_access_token: None,
                    static_url: "https://printnanny.ai/static/".into(),
                    dashboard_url: "https://printnanny.ai/dashboard/".into(),
                }
            );
            Ok(())
        });
    }
    #[test_log::test]
    fn test_save_fragment() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "Local.toml",
                r#"
                profile = "local"
                [api]
                base_path = "http://aurora:8000"
                "#,
            )?;
            jail.set_env("PRINTNANNY_CONFIG", "Local.toml");
            jail.set_env("PRINTNANNY_PATHS.confd", format!("{:?}", jail.directory()));

            let figment = PrintNannyConfig::figment();
            let mut config: PrintNannyConfig = figment.extract()?;
            config.paths.etc = jail.directory().into();

            let expected = models::PrintNannyApiConfig {
                base_path: config.api.base_path,
                bearer_access_token: Some("secret_token".to_string()),
                static_url: "https://printnanny.ai/static/".into(),
                dashboard_url: "https://printnanny.ai/dashboard/".into(),
            };
            config.api = expected.clone();
            config.try_save().unwrap();
            let figment = PrintNannyConfig::figment();
            let new: PrintNannyConfig = figment.extract()?;
            assert_eq!(new.api, expected);
            Ok(())
        });
    }

    #[test_log::test]
    fn test_find_value() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "Local.toml",
                r#"
                profile = "local"
                [api]
                base_path = "http://aurora:8000"
                "#,
            )?;
            jail.set_env("PRINTNANNY_CONFIG", "Local.toml");
            jail.set_env("PRINTNANNY_PATHS.confd", format!("{:?}", jail.directory()));

            let expected: Option<String> = Some("http://aurora:8000".into());
            let value: Option<String> =
                PrintNannyConfig::find_value("api.base_path")?.into_string();
            assert_eq!(value, expected);
            Ok(())
        });
    }

    #[test_log::test]
    fn test_os_release() {
        let config = PrintNannyConfig::new().unwrap();
        let os_release = config.os_release().unwrap();
        assert_eq!(true, os_release.contains_key("VERSION_ID"));
    }
}
