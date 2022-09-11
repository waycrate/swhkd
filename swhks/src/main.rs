use clap::arg;
use nix::{
    sys::stat::{umask, Mode},
    unistd,
    unistd::daemon,
};
use std::{
    env, fs,
    fs::OpenOptions,
    io::prelude::*,
    os::unix::net::UnixListener,
    path::Path,
    process::{exit, id, Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::{ProcessExt, System, SystemExt};

fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "swhks=warn");

    let app = clap::Command::new("swhks")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("IPC Server for swhkd")
        .arg(arg!(-l --log <FILE>).required(false).takes_value(true).help(
            "Set a custom log file. (Defaults to ${XDG_DATA_HOME:-$HOME/.local/share}/swhks-current_unix_time.log)",
        ))
		.arg(arg!(-d --debug).required(false).takes_value(false).help(
				"Enable debug mode."
		));
    let args = app.get_matches();
    if args.is_present("debug") {
        env::set_var("RUST_LOG", "swhks=trace");
    }

    env_logger::init();

    log::trace!("Setting process umask.");
    umask(Mode::S_IWGRP | Mode::S_IWOTH);

    let (pid_file_path, sock_file_path) = get_file_paths();

    let log_file_name = if let Some(val) = args.value_of("log") {
        val.to_string()
    } else {
        let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs().to_string(),
            Err(_) => {
                log::error!("SystemTime before UnixEpoch!");
                exit(1);
            }
        };

        match env::var("XDG_DATA_HOME") {
            Ok(val) => {
                log::info!(
                    "XDG_DATA_HOME Variable is present, using it's value for default file path."
                );
                format!("{}/swhks/swhks-{}.log", val, time)
            }
            Err(e) => {
                log::trace!(
                "XDG_DATA_HOME Variable is not set, falling back on hardcoded path.\nError: {:#?}",
                e
            );
                match env::var("HOME") {
                    Ok(val) => format!("{}/.local/share/swhks/swhks-{}.log", val, time),
                    Err(_) => {
                        log::error!(
                            "HOME Variable is not set, cannot fall back on hardcoded path for XDG_DATA_HOME."
                        );
                        exit(1);
                    }
                }
            }
        }
    };

    let log_path = Path::new(&log_file_name);
    if let Some(p) = log_path.parent() {
        if !p.exists() {
            if let Err(e) = fs::create_dir_all(p) {
                log::error!("Failed to create log dir: {}", e);
            }
        }
    }

    if Path::new(&pid_file_path).exists() {
        log::trace!("Reading {} file and checking for running instances.", pid_file_path);
        let swhks_pid = match fs::read_to_string(&pid_file_path) {
            Ok(swhks_pid) => swhks_pid,
            Err(e) => {
                log::error!("Unable to read {} to check all running instances", e);
                exit(1);
            }
        };
        log::debug!("Previous PID: {}", swhks_pid);

        let mut sys = System::new_all();
        sys.refresh_all();
        for (pid, process) in sys.processes() {
            if pid.to_string() == swhks_pid && process.exe() == env::current_exe().unwrap() {
                log::error!("Server is already running!");
                exit(1);
            }
        }
    }

    if Path::new(&sock_file_path).exists() {
        log::trace!("Sockfile exists, attempting to remove it.");
        match fs::remove_file(&sock_file_path) {
            Ok(_) => {
                log::debug!("Removed old socket file");
            }
            Err(e) => {
                log::error!("Error removing the socket file!: {}", e);
                log::error!("You can manually remove the socket file: {}", sock_file_path);
                exit(1);
            }
        };
    }

    match fs::write(&pid_file_path, id().to_string()) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Unable to write to {}: {}", pid_file_path, e);
            exit(1);
        }
    }

    let listener = UnixListener::bind(sock_file_path)?;
    loop {
        match listener.accept() {
            Ok((mut socket, address)) => {
                let mut response = String::new();
                socket.read_to_string(&mut response)?;
                run_system_command(&response, log_path);
                log::debug!("Socket: {:?} Address: {:?} Response: {}", socket, address, response);
            }
            Err(e) => log::error!("accept function failed: {:?}", e),
        }
    }
}

fn get_file_paths() -> (String, String) {
    match env::var("XDG_RUNTIME_DIR") {
        Ok(val) => {
            log::info!(
                "XDG_RUNTIME_DIR Variable is present, using it's value as default file path."
            );

            let pid_file_path = format!("{}/swhks.pid", val);
            let sock_file_path = format!("{}/swhkd.sock", val);

            (pid_file_path, sock_file_path)
        }
        Err(e) => {
            log::trace!("XDG_RUNTIME_DIR Variable is not set, falling back on hardcoded path.\nError: {:#?}", e);

            let pid_file_path = format!("/run/user/{}/swhks.pid", unistd::Uid::current());
            let sock_file_path = format!("/run/user/{}/swhkd.sock", unistd::Uid::current());

            (pid_file_path, sock_file_path)
        }
    }
}

fn run_system_command(command: &str, log_path: &Path) {
    _ = daemon(true, false);

    if let Err(e) = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(match OpenOptions::new().append(true).create(true).open(log_path) {
            Ok(file) => file,
            Err(e) => {
                _ = Command::new("notify-send").arg(format!("ERROR {}", e)).spawn();
                exit(1);
            }
        })
        .stderr(match OpenOptions::new().append(true).create(true).open(log_path) {
            Ok(file) => file,
            Err(e) => {
                _ = Command::new("notify-send").arg(format!("ERROR {}", e)).spawn();
                exit(1);
            }
        })
        .spawn()
    {
        log::error!("Failed to execute {}", command);
        log::error!("Error: {}", e);
    }
}
