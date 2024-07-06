use std::{collections::HashMap, error::Error, path::PathBuf, process::Command};

#[derive(Debug)]
pub struct Env {
    pub pairs: HashMap<String, String>,
}

impl Env {
    fn get_env(uname: &str) -> Result<String, Box<dyn Error>> {
        let cmd =
            Command::new("su").arg(uname).arg("-c").arg("-l").arg("env").arg(uname).output()?;
        let stdout = String::from_utf8(cmd.stdout)?;
        Ok(stdout)
    }

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

    pub fn construct(uname: &str) -> Self {
        let env = Self::get_env(uname).unwrap();
        let pairs = Self::parse_env(&env);
        Self { pairs }
    }

    pub fn fetch_xdg_config_path(&self) -> PathBuf {
        let default = String::from("/etc");
        let xdg_config_home = self.pairs.get("XDG_CONFIG_HOME").unwrap_or(&default);

        PathBuf::from(xdg_config_home).join("swhkd").join("swhkdrc")
    }

    pub fn xdg_runtime_dir(&self, uid: u32) -> PathBuf {
        let default = format!("/run/user/{}", uid);
        let xdg_runtime_dir = self.pairs.get("XDG_RUNTIME_DIR").unwrap_or(&default);
        PathBuf::from(xdg_runtime_dir)
    }
}
