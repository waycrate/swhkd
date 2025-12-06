use std::{collections::HashMap, env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Env {
    pub pairs: HashMap<String, String>,
}

impl Env {
    /// Parses an environment string into key-value pairs.
    fn parse_env(env: &str) -> HashMap<String, String> {
        env.lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, '=');
                Some((parts.next()?.to_string(), parts.next()?.to_string()))
            })
            .collect()
    }

    /// Constructs an environment structure from the given string or system environment variables.
    pub fn construct(env: Option<&str>) -> Self {
        let pairs = env.map(Self::parse_env).unwrap_or_else(|| env::vars().collect());
        Self { pairs }
    }

    /// Fetches the HOME directory path.
    pub fn fetch_home(&self) -> Option<PathBuf> {
        self.pairs.get("HOME").map(PathBuf::from)
    }

    /// Fetches the XDG config path.
    pub fn fetch_xdg_config_path(&self) -> PathBuf {
        let default = self
            .fetch_home()
            .map(|home| home.join(".config"))
            .unwrap_or_else(|| PathBuf::from("/etc"))
            .to_string_lossy() // Convert PathBuf -> Cow<'_, str>
            .into_owned();

        let xdg_config_home =
            self.pairs.get("XDG_CONFIG_HOME").map(String::as_str).unwrap_or(&default);

        PathBuf::from(xdg_config_home).join("swhkd").join("swhkdrc")
    }

    /// Fetches the XDG data path.
    pub fn fetch_xdg_data_path(&self) -> PathBuf {
        let default = self
            .fetch_home()
            .map(|home| home.join(".local/share"))
            .unwrap_or_else(|| PathBuf::from("/etc"))
            .to_string_lossy()
            .into_owned();

        let xdg_data_home = self.pairs.get("XDG_DATA_HOME").map(String::as_str).unwrap_or(&default);

        PathBuf::from(xdg_data_home)
    }

    /// Fetches the XDG runtime directory path for the given user ID.
    pub fn xdg_runtime_dir(&self, uid: u32) -> PathBuf {
        let default = format!("/run/user/{}", uid);

        let xdg_runtime_dir =
            self.pairs.get("XDG_RUNTIME_DIR").map(String::as_str).unwrap_or(&default);

        PathBuf::from(xdg_runtime_dir)
    }
}
