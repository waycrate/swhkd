use std::{
    env::VarError,
    path::{Path, PathBuf},
};

pub struct Env {
    pub xdg_config_home: PathBuf,
    pub xdg_runtime_dir: PathBuf,
}

#[derive(Debug)]
pub enum EnvError {
    XdgConfigNotFound,
    XdgRuntimeNotFound,
    PathNotFound,
    GenericError(String),
}

impl Env {
    pub fn construct(uid: u32) -> Self {
        let xdg_config_home = match Self::get_env("XDG_CONFIG_HOME") {
            Ok(val) => match validate_path(&PathBuf::from(val)) {
                Ok(val) => val,
                Err(e) => match e {
                    EnvError::PathNotFound => {
                        log::warn!("XDG_CONFIG_HOME does not exist, using hardcoded /etc");
                        PathBuf::from("/etc")
                    }
                    _ => {
                        eprintln!("Failed to get XDG_CONFIG_HOME: {:?}", e);
                        std::process::exit(1);
                    }
                },
            },
            Err(e) => match e {
                EnvError::XdgConfigNotFound => {
                    log::warn!("XDG_CONFIG_HOME not found, using hardcoded /etc");
                    PathBuf::from("/etc")
                }
                _ => {
                    eprintln!("Failed to get XDG_CONFIG_HOME: {:?}", e);
                    std::process::exit(1);
                }
            },
        };

        let xdg_runtime_dir = match Self::get_env("XDG_RUNTIME_DIR") {
            Ok(val) => PathBuf::from(val),
            Err(e) => match e {
                EnvError::XdgRuntimeNotFound => {
                    log::warn!("XDG_RUNTIME_DIR not found, using hardcoded /run/user");
                    PathBuf::from(format!("/run/user/{}", uid))
                }
                _ => {
                    eprintln!("Failed to get XDG_RUNTIME_DIR: {:?}", e);
                    std::process::exit(1);
                }
            },
        };

        Self { xdg_config_home, xdg_runtime_dir }
    }

    fn get_env(name: &str) -> Result<String, EnvError> {
        match std::env::var(name) {
            Ok(val) => Ok(val),
            Err(e) => match e {
                VarError::NotPresent => match name {
                    "XDG_CONFIG_HOME" => Err(EnvError::XdgConfigNotFound),
                    "XDG_RUNTIME_DIR" => Err(EnvError::XdgRuntimeNotFound),
                    _ => Err(EnvError::GenericError(e.to_string())),
                },
                VarError::NotUnicode(_) => {
                    Err(EnvError::GenericError("Not a valid unicode".to_string()))
                }
            },
        }
    }

    pub fn fetch_xdg_config_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.xdg_config_home).join("swhkd/swhkdrc");
        // if path doesn't exist, create the file
        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, "# This is the default").unwrap();
        }
        path
    }
}

fn validate_path(path: &Path) -> Result<PathBuf, EnvError> {
    if path.exists() {
        Ok(path.to_path_buf())
    } else {
        Err(EnvError::PathNotFound)
    }
}
