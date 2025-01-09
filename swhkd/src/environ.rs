use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Env {
    pub pairs: HashMap<String, String>,
}

impl Env {
    fn parse_env(env: &str) -> HashMap<String, String> {
        let mut pairs = HashMap::new();
        for line in env.lines() {
            let mut parts = line.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                pairs.insert(key.to_string(), value.to_string());
            }
        }
        pairs
    }

    /// Construct the env from the environment variables
    pub fn construct(env: Option<&str>) -> Self {
        let env = match env {
            Some(env) => env.to_string(),
            None => "".to_string(),
        };
        let pairs = Self::parse_env(&env);
        Self { pairs }
    }

    pub fn fetch_home(&self) -> Option<PathBuf> {
        let home = match self.pairs.get("HOME") {
            Some(it) => it,
            None => return None,
        };
        Some(PathBuf::from(home))
    }

    pub fn fetch_xdg_config_path(&self) -> PathBuf {
        let default = match self.fetch_home() {
            Some(x) => x.join(".config"),
            None => PathBuf::from("/etc"),
        }
        .to_string_lossy()
        .to_string();
        let xdg_config_home = self.pairs.get("XDG_CONFIG_HOME").unwrap_or(&default);

        PathBuf::from(xdg_config_home).join("swhkd").join("swhkdrc")
    }

    pub fn fetch_xdg_data_path(&self) -> PathBuf {
        let default = match self.fetch_home() {
            Some(x) => x.join(".local").join("share"),
            None => PathBuf::from("/etc"),
        }
        .to_string_lossy()
        .to_string();
        let xdg_config_home = self.pairs.get("XDG_DATA_HOME").unwrap_or(&default);

        PathBuf::from(xdg_config_home)
    }

    pub fn xdg_runtime_dir(&self, uid: u32) -> PathBuf {
        let default = format!("/run/user/{}", uid);
        let xdg_runtime_dir = self.pairs.get("XDG_RUNTIME_DIR").unwrap_or(&default);
        PathBuf::from(xdg_runtime_dir)
    }
}
