use std::{
    collections::HashMap, io::Write, os::unix::net::UnixStream, path::PathBuf, process::Command,
};

use clap::Parser;
use nix::unistd::daemon;

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

    let runtime_dir = env.get("XDG_RUNTIME_DIR").unwrap();

    let (_pid_file_path, sock_file_path) = get_file_paths(runtime_dir);

    log::info!("Started SWHKS placeholder server");
    let _ = daemon(true, false);
    loop{
        match UnixStream::connect(&sock_file_path){
            Ok(mut stream) => {
                let _ = stream.write_all(env_raw.as_bytes());
            },
            Err(_) => {
                println!("Waiting...");
            },
        };
    }
}

fn get_file_paths(runtime_dir: &str) -> (String, String) {
    let pid_file_path = format!("{}/swhks.pid", runtime_dir);
    let sock_file_path = format!("{}/swhkd.sock", runtime_dir);

    (pid_file_path, sock_file_path)
}
