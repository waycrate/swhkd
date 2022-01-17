use clap::{arg, App};
use evdev::{Device, Key};
use nix::unistd;
use std::env;
use std::path::Path;
use std::process::exit;

pub fn main() {
    /* Clap builder for flag handling */
    let args = App::new("swhkd")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Simple Wayland HotKey Daemon")
        .arg(
            arg!(-c --config <CONFIG_FILE_PATH>)
                .required(false)
                .help("Set a custom config file path"),
        )
        .arg(arg!(-d - -debug).required(false).help("Enable debug mode"))
        .get_matches();

    /* Set log level to debug if flag is present */
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
        if !config_file_path.exists() {
            log::error!("{:#?} path doesn't exist", config_file_path);
            exit(1);
        }
    } else {
        match env::var("XDG_CONFIG_HOME") {
            Ok(val) => {
                config_file_path = Path::new(&val).join("swhkd/swhkdrc");
                log::debug!("XDG_CONFIG_HOME exists: {:#?}", val);
            }
            Err(_) => {
                log::error!("XDG_CONFIG_HOME has not been set.");
                match env::var("HOME") {
                    Ok(val) => {
                        config_file_path = Path::new(&val).join(".config/swhkd/swhkdrc");
                    }
                    Err(_) => {
                        log::error!("HOME env var has not been set");
                        exit(1);
                    }
                }
            }
        }
    }
    log::debug!("Using config file path: {:#?}", config_file_path);

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
}

pub fn permission_check() -> bool {
    if unistd::Uid::current().is_root() {
        log::error!("Refusing to run swhkd as root.");
        return false;
    }

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
    log::error!("Invoking user is NOT in input group.");
    return false;
}

pub fn check_keyboard(device: &Device) -> bool {
    /* Check for the presence of enter key. */
    if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        log::debug!(
            "{} ({}) is a keyboard.",
            device.name().unwrap(),
            device.physical_path().unwrap()
        );
        return true;
    } else {
        log::trace!(
            "{} ({}) is not a keyboard.",
            device.name().unwrap(),
            device.physical_path().unwrap()
        );
        return false;
    }
}
