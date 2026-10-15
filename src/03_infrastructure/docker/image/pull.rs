use crate::application::error::AppError;
use bollard::auth::DockerCredentials;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct DockerConfigFile {
    #[serde(rename = "auths")]
    auths: HashMap<String, DockerConfigAuthEntry>,
    #[serde(rename = "credHelpers")]
    cred_helpers: HashMap<String, String>,
    #[serde(rename = "credsStore")]
    creds_store: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct DockerConfigAuthEntry {
    #[serde(rename = "auth")]
    auth: Option<String>,
    username: Option<String>,
    password: Option<String>,
    email: Option<String>,
}

#[derive(Default)]
pub(crate) struct RegistryCredentials {
    entries: HashMap<String, DockerConfigAuthEntry>,
    cred_helpers: HashMap<String, String>,
    creds_store: Option<String>,
}

fn registry_domain(image_ref: &str) -> String {
    let first_segment = image_ref.split('/').next().unwrap_or(image_ref);
    let has_host = first_segment.contains('.') || first_segment == "localhost";
    let has_port = first_segment
        .rsplit_once(':')
        .is_some_and(|(_, port)| !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()));
    if has_host || has_port {
        first_segment.to_string()
    } else {
        "https://index.docker.io/v1/".to_string()
    }
}

fn decode_basic_auth(auth: &str) -> Option<(String, String)> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(auth)
        .ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (username, password) = decoded.split_once(':')?;
    Some((username.to_string(), password.to_string()))
}

impl RegistryCredentials {
    pub(crate) fn load() -> Self {
        Self::load_from(Self::path()).unwrap_or_default()
    }

    fn path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".docker").join("config.json"))
    }

    pub(crate) fn load_from(path: Option<PathBuf>) -> Result<Self, std::io::Error> {
        let Some(path) = path else {
            return Ok(RegistryCredentials::default());
        };
        let content = std::fs::read_to_string(path)?;
        let config: DockerConfigFile = serde_json::from_str(&content).unwrap_or_default();
        Ok(RegistryCredentials {
            entries: config.auths,
            cred_helpers: config.cred_helpers,
            creds_store: config.creds_store,
        })
    }

    pub(crate) fn for_image(&self, image_ref: &str) -> Option<DockerCredentials> {
        let domain = registry_domain(image_ref);
        if self.uses_credential_helper(&domain) {
            return None;
        }
        let entry = self.entries.get(&domain)?;
        let mut credentials = DockerCredentials {
            serveraddress: Some(domain),
            ..Default::default()
        };
        if let Some(auth) = &entry.auth {
            if let Some((username, password)) = decode_basic_auth(auth) {
                credentials.username = Some(username);
                credentials.password = Some(password);
            }
        }
        credentials.username = credentials.username.or_else(|| entry.username.clone());
        credentials.password = credentials.password.or_else(|| entry.password.clone());
        credentials.email = entry.email.clone();
        Some(credentials)
    }

    fn uses_credential_helper(&self, domain: &str) -> bool {
        self.cred_helpers.contains_key(domain)
            || self
                .creds_store
                .as_ref()
                .is_some_and(|store| !store.is_empty())
    }
}

pub(crate) fn pull_operation_error(operation: &str, image: &str, err: AppError) -> AppError {
    AppError::operation_failed(format!("Failed to {operation} image '{image}': {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_config(dir: &std::path::Path, content: &str) -> PathBuf {
        let docker_dir = dir.join(".docker");
        std::fs::create_dir_all(&docker_dir).unwrap();
        let path = docker_dir.join("config.json");
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_registry_domain_defaults_to_docker_hub() {
        assert_eq!(
            registry_domain("nginx:latest"),
            "https://index.docker.io/v1/"
        );
        assert_eq!(registry_domain("ghcr.io/org/app:1"), "ghcr.io");
        assert_eq!(registry_domain("localhost:5000/app"), "localhost:5000");
    }

    #[test]
    fn test_load_missing_file_yields_empty_credentials() {
        let creds = RegistryCredentials::load_from(None).unwrap();
        assert!(creds.for_image("nginx:latest").is_none());
    }

    #[test]
    fn test_load_parses_auth_entries() {
        use base64::Engine;
        let auth = base64::engine::general_purpose::STANDARD.encode("user:pass");
        let content = format!(r#"{{"auths": {{"ghcr.io": {{"auth": "{auth}"}}}}}}"#);
        let dir = std::env::temp_dir().join(format!("rc-test-{}", std::process::id()));
        let path = write_config(&dir, &content);
        let creds = RegistryCredentials::load_from(Some(path)).unwrap();
        let credentials = creds.for_image("ghcr.io/org/app:1").unwrap();
        assert_eq!(credentials.username.as_deref(), Some("user"));
        assert_eq!(credentials.password.as_deref(), Some("pass"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_credential_store_disables_inline_credentials() {
        let content =
            r#"{"credsStore": "desktop", "auths": {"ghcr.io": {"auth": "dXNlcjpwYXNz"}}}"#;
        let dir = std::env::temp_dir().join(format!("rc-test-cs-{}", std::process::id()));
        let path = write_config(&dir, content);
        let creds = RegistryCredentials::load_from(Some(path)).unwrap();
        assert!(creds.for_image("ghcr.io/org/app:1").is_none());
        std::fs::remove_dir_all(&dir).ok();
    }
}
