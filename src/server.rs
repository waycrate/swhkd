use nix::{
    libc::daemon,
    sys::stat::{umask, Mode},
    unistd,
};
use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::{
    env, fs,
    path::Path,
    process::{exit, id, Command, Stdio},
};
use sysinfo::{ProcessExt, System, SystemExt};

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

fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "swhks=trace");
    env_logger::init();

    log::trace!("Setting process umask.");
    umask(Mode::S_IWGRP | Mode::S_IWOTH);

    let (pid_file_path, sock_file_path) = get_file_paths();

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
                run_system_command(&response);
                log::debug!("Socket: {:?} Address: {:?} Response: {}", socket, address, response);
            }
            Err(e) => log::error!("accept function failed: {:?}", e),
        }
    }
}

fn run_system_command(command: &str) {
    unsafe {
        daemon(1, 0);
    }
    match Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to execute {}", command);
            log::error!("Error, {}", e);
        }
    }
}
