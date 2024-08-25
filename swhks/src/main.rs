use std::collections::HashMap;
use std::error::Error;
use std::fs::Permissions;
use std::{
    fs::{self},
    io::Write,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use nix::unistd::daemon;
use std::{
    env,
    os::unix::fs::PermissionsExt,
    process::{exit, id},
};
use sysinfo::System;
use sysinfo::{ProcessExt, SystemExt};

/// IPC Server for swhkd
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set a custom log file. (Defaults to ${XDG_DATA_HOME:-$HOME/.local/share}/swhks-current_unix_time.log)
    #[arg(short, long, value_name = "FILE")]
    log: Option<PathBuf>,

    /// Enable Debug Mode
    #[arg(short, long)]
    debug: bool,
}

fn get_env() -> Result<String, Box<dyn std::error::Error>> {
    let shell = std::env::var("SHELL")?;
    let cmd = Command::new(shell).arg("-c").arg("env").output()?;
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
fn main() -> std::io::Result<()> {
    let args = Args::parse();
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("swhks=trace"))
            .init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("swhks=warn"))
            .init();
    }

    let env_raw = match get_env() {
        Ok(env) => env,
        Err(_) => "".to_string(),
    };

    let env = parse_env(&env_raw);

    let invoking_uid = get_uid().unwrap();
    let default_runtime_dir = format!("/run/user/{}", invoking_uid);
    let runtime_dir = env.get("XDG_RUNTIME_DIR").unwrap_or(&default_runtime_dir);

    let (_pid_file_path, sock_file_path) = get_file_paths(runtime_dir);

    log::info!("Started SWHKS placeholder server");
    let _ = daemon(true, false);
    setup_swhks(invoking_uid, PathBuf::from(runtime_dir));
    loop {
        if let Ok(mut stream) = UnixStream::connect(&sock_file_path) {
            let _ = stream.write_all(env_raw.as_bytes());
        };
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}

pub fn setup_swhks(invoking_uid: u32, runtime_path: PathBuf) {
    // Get the runtime path and create it if needed.
    if !Path::new(&runtime_path).exists() {
        match fs::create_dir_all(Path::new(&runtime_path)) {
            Ok(_) => {
                log::debug!("Created runtime directory.");
                match fs::set_permissions(Path::new(&runtime_path), Permissions::from_mode(0o600)) {
                    Ok(_) => log::debug!("Set runtime directory to readonly."),
                    Err(e) => log::error!("Failed to set runtime directory to readonly: {}", e),
                }
            }
            Err(e) => log::error!("Failed to create runtime directory: {}", e),
        }
    }

    // Get the PID file path for instance tracking.
    let pidfile: String = format!("{}/swhks_{}.pid", runtime_path.to_string_lossy(), invoking_uid);
    if Path::new(&pidfile).exists() {
        log::trace!("Reading {} file and checking for running instances.", pidfile);
        let swhks_pid = match fs::read_to_string(&pidfile) {
            Ok(swhks_pid) => swhks_pid,
            Err(e) => {
                log::error!("Unable to read {} to check all running instances", e);
                exit(1);
            }
        };
        log::debug!("Previous PID: {}", swhks_pid);

        // Check if swhkd is already running!
        let mut sys = System::new_all();
        sys.refresh_all();
        for (pid, process) in sys.processes() {
            if pid.to_string() == swhks_pid && process.exe() == env::current_exe().unwrap() {
                log::error!("Swhks is already running!");
                log::error!("There is no need to run another instance since there is already one running with PID: {}", swhks_pid);
                exit(1);
            }
        }
    }

    // Write to the pid file.
    match fs::write(&pidfile, id().to_string()) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Unable to write to {}: {}", pidfile, e);
            exit(1);
        }
    }
}

fn get_file_paths(runtime_dir: &str) -> (String, String) {
    let pid_file_path = format!("{}/swhks.pid", runtime_dir);
    let sock_file_path = format!("{}/swhkd.sock", runtime_dir);

    (pid_file_path, sock_file_path)
}

/// Get the UID of the user that is not a system user
fn get_uid() -> Result<u32, Box<dyn Error>> {
    let status_content = fs::read_to_string(format!("/proc/{}/loginuid", std::process::id()))?;
    let uid = status_content.trim().parse::<u32>()?;
    Ok(uid)
}
