use std::path::PathBuf;

use clap::Parser;

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

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("swhks=trace"))
            .init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("swhks=warn"))
            .init();
    }

    log::info!("Started SWHKS placeholder server");

    Ok(())
}
