use std::{env::VarError, path::PathBuf};

pub struct Env {
    pub pkexec_id: u32,
    pub xdg_config_home: PathBuf,
    pub xdg_runtime_socket: PathBuf,
    pub xdg_runtime_dir: PathBuf,
}

#[derive(Debug)]
pub enum EnvError {
    PkexecNotFound,
    XdgConfigNotFound,
    XdgRuntimeNotFound,
    GenericError(String),
}

impl Env {
    pub fn contruct() -> Self {
        let pkexec_id = match Self::get_env("PKEXEC_UID") {
            Ok(val) => match val.parse::<u32>() {
                Ok(val) => val,
                Err(_) => {
                    log::error!("Failed to launch swhkd!!!");
                    log::error!("Make sure to launch the binary with pkexec.");
                    std::process::exit(1);
                }
            },
            Err(_) => {
                log::error!("Failed to launch swhkd!!!");
                log::error!("Make sure to launch the binary with pkexec.");
                std::process::exit(1);
            }
        };

        let xdg_config_home = match Self::get_env("XDG_CONFIG_HOME") {
            Ok(val) => PathBuf::from(val),
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

        let xdg_runtime_socket = match Self::get_env("XDG_RUNTIME_DIR") {
            Ok(val) => PathBuf::from(val),
            Err(e) => match e {
                EnvError::XdgRuntimeNotFound => {
                    log::warn!("XDG_RUNTIME_DIR not found, using hardcoded /run/user");
                    PathBuf::from(format!("/run/user/{}", pkexec_id))
                }
                _ => {
                    eprintln!("Failed to get XDG_RUNTIME_DIR: {:?}", e);
                    std::process::exit(1);
                }
            },
        };

        let xdg_runtime_dir = match Self::get_env("XDG_RUNTIME_DIR") {
            Ok(val) => PathBuf::from(val),
            Err(e) => match e {
                EnvError::XdgRuntimeNotFound => {
                    log::warn!("XDG_RUNTIME_DIR not found, using hardcoded /run/swhkd");
                    PathBuf::from("/run/swhkd")
                }
                _ => {
                    eprintln!("Failed to get XDG_RUNTIME_DIR: {:?}", e);
                    std::process::exit(1);
                }
            },
        };

        Self { pkexec_id, xdg_config_home, xdg_runtime_dir, xdg_runtime_socket }
    }

    fn get_env(name: &str) -> Result<String, EnvError> {
        match std::env::var(name) {
            Ok(val) => Ok(val),
            Err(e) => match e {
                VarError::NotPresent => match name {
                    "PKEXEC_UID" => Err(EnvError::PkexecNotFound),
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
        PathBuf::from(&self.xdg_config_home).join("swhkd/swhkdrc")
    }

    pub fn fetch_xdg_runtime_socket_path(&self) -> PathBuf {
        PathBuf::from(&self.xdg_runtime_dir).join("swhkd.sock")
    }
}
