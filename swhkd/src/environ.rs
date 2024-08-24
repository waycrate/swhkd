use std::{collections::HashMap, error::Error, path::PathBuf, process::Command};

#[derive(Debug)]
pub struct Env {
    pub pairs: HashMap<String, String>,
    pub uname: String,
}

impl Env {
    pub fn get_default_shell() -> Result<String, Box<dyn Error>> {
        // read from /etc/passwd
        let passwd = std::fs::read_to_string("/etc/passwd")?;
        let lines: Vec<&str> = passwd.split('\n').collect();
        for line in lines {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 2 {
                let user_shell = parts[6];
                return Ok(user_shell.to_string());
            }
        }
        Err("User shell not found".into())
    }

    fn get_env(uname: &str) -> Result<String, Box<dyn Error>> {
        // let shell = Self::get_default_shell()?;
        let cmd =
            Command::new("su").arg(uname).arg("-c").arg("-l").arg("env").output()?;
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

    pub fn construct(uname: &str, env: Option<&str>) -> Self {
        let env = match env {
            Some(env) => env.to_string(),
            None => Self::get_env(uname).unwrap(),
        };
        let pairs = Self::parse_env(&env);
        Self { pairs, uname: uname.to_string() }
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
