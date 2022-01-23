use clap::{arg, App};
use evdev::{Device, Key};
use interprocess::local_socket::LocalSocketStream;
use nix::unistd;
use std::{env, io::prelude::*, path::Path, process::exit};

pub fn main() {
    let args = set_flags().get_matches();
    env::set_var("RUST_LOG", "swhkd=warn");

    if args.is_present("debug") {
        env::set_var("RUST_LOG", "swhkd=trace");
    }

    env_logger::init();
    log::trace!("Logger initialized.");

    if !permission_check() {
        exit(1);
    }

    /* Get appropriate config file path */
    let config_file_path: std::path::PathBuf;
    if args.is_present("config") {
        config_file_path = Path::new(args.value_of("config").unwrap()).to_path_buf();
    } else {
        config_file_path = check_config_xdg();
    }
    log::debug!("Using config file path: {:#?}", config_file_path);

    if !config_file_path.exists() {
        log::error!("{:#?} doesn't exist", config_file_path);
        exit(1);
    }

    log::trace!("Attempting to find all keyboard file descriptors.");
    let mut keyboard_devices: Vec<Device> = Vec::new();
    for (_, device) in evdev::enumerate().enumerate() {
        if check_keyboard(&device) == true {
            keyboard_devices.push(device);
        }
    }

    if keyboard_devices.len() == 0 {
        log::error!("No valid keyboard device was detected!");
        exit(1);
    }
    log::debug!("{} Keyboard device(s) detected.", keyboard_devices.len());

    //TODO: IMPLEMENT KEYBOARD EVENT GRAB

    let mut conn = match LocalSocketStream::connect("/tmp/swhkd.sock") {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Unable to connect to hotkey server, is swhks running??");
            log::error!("Error: {}", e);
            exit(1);
        }
    };

    match conn.write_all(args.value_of("shell").unwrap().as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Unable to send command to hotkey server, is swhks running??");
            log::error!("Error: {}", e);
            exit(1);
        }
    };
}

pub fn permission_check() -> bool {
    let groups = unistd::getgroups();
    for (_, groups) in groups.iter().enumerate() {
        for group in groups {
            let group = unistd::Group::from_gid(*group);
            if group.unwrap().unwrap().name == "input" {
                log::debug!("Invoking user is in input group.");
                return true;
            }
        }
    }

    if unistd::Uid::current().is_root() {
        log::warn!("Running swhkd as root!!!");
        return true;
    } else {
        log::error!("Invoking user is NOT in input group.");
        return false;
    }
}

pub fn check_keyboard(device: &Device) -> bool {
    /* Check for the presence of enter key. */
    if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        log::debug!("{} is a keyboard.", device.name().unwrap(),);
        return true;
    } else {
        log::trace!("{} is not a keyboard.", device.name().unwrap(),);
        return false;
    }
}

pub fn set_flags() -> App<'static> {
    let app = App::new("swhkd")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Simple Wayland HotKey Daemon")
        .arg(
            arg!(-c --config <CONFIG_FILE_PATH>)
                .required(false)
                .help("Set a custom config file path"),
        )
        .arg(arg!(-d - -debug).required(false).help("Enable debug mode"))
        .arg(
            arg!(-s - -shell <SHELL_COMMAND>)
                .required(true)
                .help("Shell command to run on success"),
        );
    return app;
}

pub fn check_config_xdg() -> std::path::PathBuf {
    let config_file_path: std::path::PathBuf;
    match env::var("XDG_CONFIG_HOME") {
        Ok(val) => {
            config_file_path = Path::new(&val).join("swhkd/swhkdrc");
            log::debug!("XDG_CONFIG_HOME exists: {:#?}", val);
            return config_file_path;
        }
        Err(_) => {
            log::error!("XDG_CONFIG_HOME has not been set.");
            config_file_path = Path::new("/etc/swhkd/swhkdrc").to_path_buf();
            log::warn!(
                "Note: Due to the design of the application, the invoking user is always root."
            );
            log::warn!("You can set a custom config file with the -c option.");
            log::warn!("Adding your user to the input group could solve this.");
            log::warn!("However that's a massive security flaw and basically defeats the purpose of using wayland.");
            log::warn!("The following issue may be addressed in the future, but it is certainly not a priority right now.");
        }
    }
    return config_file_path;
}
