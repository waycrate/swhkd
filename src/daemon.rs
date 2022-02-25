use clap::{arg, Command};
use evdev::{AttributeSet, Device, Key};
use nix::unistd::{Group, Uid};
use std::{
    collections::HashMap,
    env, fs,
    io::prelude::*,
    os::unix::net::UnixStream,
    path::Path,
    process::{exit, id},
    thread::sleep,
    time::Duration,
    time::SystemTime,
};
use sysinfo::{ProcessExt, System, SystemExt};

mod config;
mod tests;

#[derive(PartialEq)]
pub struct LastHotkey {
    hotkey: config::Hotkey,
    ran_at: SystemTime,
}

pub fn main() {
    let args = set_flags().get_matches();
    env::set_var("RUST_LOG", "swhkd=warn");

    if args.is_present("debug") {
        env::set_var("RUST_LOG", "swhkd=trace");
    }

    env_logger::init();
    log::trace!("Logger initialized.");

    let pidfile: String = String::from("/tmp/swhkd.pid");
    if Path::new(&pidfile).exists() {
        log::trace!("Reading {} file and checking for running instances.", pidfile);
        let swhkd_pid = match fs::read_to_string(&pidfile) {
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
                log::error!("Swhkd is already running!");
                exit(1);
            }
        }
    }

    match fs::write(&pidfile, id().to_string()) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Unable to write to {}: {}", pidfile, e);
            exit(1);
        }
    }

    permission_check();

    let config_file_path: std::path::PathBuf = if args.is_present("config") {
        Path::new(args.value_of("config").unwrap()).to_path_buf()
    } else {
        check_config_xdg()
    };
    log::debug!("Using config file path: {:#?}", config_file_path);

    if !config_file_path.exists() {
        log::error!("{:#?} doesn't exist", config_file_path);
        exit(1);
    }

    log::trace!("Attempting to find all keyboard file descriptors.");
    let mut keyboard_devices: Vec<Device> = Vec::new();
    for (_, device) in evdev::enumerate().enumerate() {
        if check_keyboard(&device) {
            keyboard_devices.push(device);
        }
    }

    if keyboard_devices.is_empty() {
        log::error!("No valid keyboard device was detected!");
        exit(1);
    }
    log::debug!("{} Keyboard device(s) detected.", keyboard_devices.len());

    let hotkeys = match config::load(config_file_path) {
        Err(e) => {
            log::error!("Config Error: {}", e);
            exit(1);
        }
        Ok(out) => out,
    };

    for hotkey in &hotkeys {
        log::debug!("hotkey: {:#?}", hotkey);
    }

    let modifiers_map: HashMap<Key, config::Modifier> = HashMap::from([
        (Key::KEY_LEFTMETA, config::Modifier::Super),
        (Key::KEY_RIGHTMETA, config::Modifier::Super),
        (Key::KEY_LEFTMETA, config::Modifier::Super),
        (Key::KEY_RIGHTMETA, config::Modifier::Super),
        (Key::KEY_LEFTALT, config::Modifier::Alt),
        (Key::KEY_RIGHTALT, config::Modifier::Alt),
        (Key::KEY_LEFTCTRL, config::Modifier::Control),
        (Key::KEY_RIGHTCTRL, config::Modifier::Control),
        (Key::KEY_LEFTSHIFT, config::Modifier::Shift),
        (Key::KEY_RIGHTSHIFT, config::Modifier::Shift),
    ]);

    let repeat_cooldown_duration: u128 = if args.is_present("cooldown") {
        args.value_of("cooldown").unwrap().parse::<u128>().unwrap()
    } else {
        250
    };

    let mut key_states: Vec<AttributeSet<Key>> = Vec::new();
    let mut possible_hotkeys: Vec<config::Hotkey> = Vec::new();

    let mut last_hotkey: Option<LastHotkey> = None;

    fn send_command(hotkey: config::Hotkey) {
        log::info!("Hotkey pressed: {:#?}", hotkey);
        if let Err(e) = sock_send(&hotkey.command) {
            log::error!("Failed to send command over IPC.");
            log::error!("Is swhks running?");
            log::error!("{:#?}", e)
        }
    }

    loop {
        for device in &keyboard_devices {
            key_states.push(device.get_key_state().unwrap());
        }
        // check if a hotkey in hotkeys is pressed
        for state in &key_states {
            for hotkey in &hotkeys {
                if hotkey.modifiers.len() < state.iter().count() {
                    possible_hotkeys.push(hotkey.clone());
                } else {
                    continue;
                }
            }

            if possible_hotkeys.is_empty() {
                continue;
            }

            let mut state_modifiers: Vec<config::Modifier> = Vec::new();
            let mut state_keysyms: Vec<evdev::Key> = Vec::new();
            for key in state.iter() {
                if let Some(modifier) = modifiers_map.get(&key) {
                    state_modifiers.push(*modifier);
                } else {
                    state_keysyms.push(key);
                }
            }
            log::debug!("state_modifiers: {:#?}", state_modifiers);
            log::debug!("state_keysyms: {:#?}", state_keysyms);
            log::debug!("hotkey: {:#?}", possible_hotkeys);
            for hotkey in &possible_hotkeys {
                // this should check if state_modifiers and hotkey.modifiers have the same elements
                if state_modifiers.iter().all(|x| hotkey.modifiers.contains(x))
                    && state_modifiers.len() == hotkey.modifiers.len()
                    && state_keysyms.contains(&hotkey.keysym)
                {
                    if last_hotkey == None {
                        last_hotkey =
                            Some(LastHotkey { hotkey: hotkey.clone(), ran_at: SystemTime::now() });
                        send_command(hotkey.clone());
                        continue;
                    }
                    if last_hotkey.as_ref().unwrap().hotkey != hotkey.clone() {
                        last_hotkey =
                            Some(LastHotkey { hotkey: hotkey.clone(), ran_at: SystemTime::now() });
                        send_command(hotkey.clone());
                        continue;
                    }
                    let time_since_ran_at = match SystemTime::now()
                        .duration_since(last_hotkey.as_ref().unwrap().ran_at)
                    {
                        Ok(n) => n.as_millis(),
                        Err(e) => {
                            log::error!("Error: {:#?}", e);
                            exit(1);
                        }
                    };
                    if time_since_ran_at <= repeat_cooldown_duration {
                        log::error!(
                            "In cooldown: {:#?} \nTime Remaining: {:#?}ms",
                            hotkey,
                            repeat_cooldown_duration - time_since_ran_at
                        );
                        continue;
                    } else {
                        last_hotkey =
                            Some(LastHotkey { hotkey: hotkey.clone(), ran_at: SystemTime::now() });
                    }
                    send_command(hotkey.clone());
                }
            }
        }

        key_states.clear();
        possible_hotkeys.clear();
        sleep(Duration::from_millis(10)); // without this, swhkd will start to chew through your cpu.
    }
}
pub fn permission_check() {
    if !Uid::current().is_root() {
        let groups = nix::unistd::getgroups();
        for (_, groups) in groups.iter().enumerate() {
            for group in groups {
                let group = Group::from_gid(*group);
                if group.unwrap().unwrap().name == "input" {
                    log::error!("Note: INVOKING USER IS IN INPUT GROUP!!!!");
                    log::error!("THIS IS A HUGE SECURITY RISK!!!!");
                }
            }
        }
        log::error!("Consider using `pkexec swhkd ...`");
        exit(1);
    } else {
        log::warn!("Running swhkd as root!");
    }
}

pub fn check_keyboard(device: &Device) -> bool {
    if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        log::debug!("{} is a keyboard.", device.name().unwrap(),);
        true
    } else {
        log::trace!("{} is not a keyboard.", device.name().unwrap(),);
        false
    }
}

pub fn set_flags() -> Command<'static> {
    let app = Command::new("swhkd")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Simple Wayland HotKey Daemon")
        .arg(
            arg!(-c --config <CONFIG_FILE_PATH>)
                .required(false)
                .takes_value(true)
                .help("Set a custom config file path."),
        )
        .arg(
            arg!(-C --cooldown <COOLDOWN_IN_MS>)
                .required(false)
                .takes_value(true)
                .help("Set a custom repeat cooldown duration. Default is 250ms."),
        )
        .arg(arg!(-d - -debug).required(false).help("Enable debug mode."));
    app
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
    config_file_path
}

fn sock_send(command: &str) -> std::io::Result<()> {
    let mut stream = UnixStream::connect("/tmp/swhkd.sock")?;
    stream.write_all(command.as_bytes())?;
    Ok(())
}
