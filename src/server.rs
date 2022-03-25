use nix::unistd;
use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::{
    env, fs,
    path::Path,
    process::{exit, id, Command, Stdio},
};
use sysinfo::{ProcessExt, System, SystemExt};

fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "swhks=trace");
    env_logger::init();

    let pid_file_path = String::from("/tmp/swhks.pid");
    let sock_file_path = String::from(format!("/run/user/{}/swhkd.sock", unistd::Uid::current()));

    if Path::new(&pid_file_path).exists() {
        log::trace!("Reading {} file and checking for running instances.", pid_file_path);
        let swhkd_pid = match fs::read_to_string(&pid_file_path) {
            Ok(swhkd_pid) => swhkd_pid,
            Err(e) => {
                log::error!("Unable to read {} to check all running instances", e);
                exit(1);
            }
        };
        log::debug!("Previous PID: {}", swhkd_pid);

        let mut sys = System::new_all();
        sys.refresh_all();
        for (pid, process) in sys.processes() {
            if pid.to_string() == swhkd_pid && process.exe() == env::current_exe().unwrap() {
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
