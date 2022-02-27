use clap::{arg, Command};
use evdev::{AttributeSet, AutoRepeat, Device, InputEventKind, Key};
use nix::unistd::{Group, Uid};
use nix::{
    fcntl::{fcntl, FcntlArg, OFlag},
    poll::{PollFd, PollFlags},
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::prelude::*,
    os::unix::{net::UnixStream, prelude::AsRawFd},
    path::Path,
    process::{exit, id},
    time::SystemTime,
};
use sysinfo::{ProcessExt, System, SystemExt};

use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;

mod config;

#[cfg(test)]
mod tests;

#[derive(PartialEq)]
pub struct LastHotkey {
    hotkey: config::Hotkey,
    ran_at: SystemTime,
}

struct KeyboardDevice {
    device: Device,
    state_modifiers: HashSet<config::Modifier>,
    state_keysyms: AttributeSet<evdev::Key>,
}

impl KeyboardDevice {
    fn new(device: Device) -> KeyboardDevice {
        KeyboardDevice {
            device,
            state_modifiers: HashSet::new(),
            state_keysyms: AttributeSet::new(),
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let load_config = || {
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
        hotkeys
    };

    let mut hotkeys = load_config();

    log::trace!("Attempting to find all keyboard file descriptors.");
    let mut keyboard_devices: Vec<KeyboardDevice> =
        evdev::enumerate().filter(check_keyboard).map(KeyboardDevice::new).collect();

    if keyboard_devices.is_empty() {
        log::error!("No valid keyboard device was detected!");
        exit(1);
    }
    for device in keyboard_devices.iter_mut() {
        fcntl(device.device.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();
        device.device.update_auto_repeat(&AutoRepeat { delay: 0, period: 0 }).unwrap();
    }
    log::debug!("{} Keyboard device(s) detected.", keyboard_devices.len());

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

    fn send_command(hotkey: config::Hotkey) {
        log::info!("Hotkey pressed: {:#?}", hotkey);
        if let Err(e) = sock_send(&hotkey.command) {
            log::error!("Failed to send command over IPC.");
            log::error!("Is swhks running?");
            log::error!("{:#?}", e)
        }
    }

    let pause = Arc::new(AtomicBool::new(false));
    let resume = Arc::new(AtomicBool::new(false));
    let reload = Arc::new(AtomicBool::new(false));
    let pause_temp = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGUSR1, Arc::clone(&pause))?;
    signal_flag::register(SIGUSR2, Arc::clone(&resume))?;
    signal_flag::register(SIGHUP, Arc::clone(&reload))?;
    signal_flag::register(SIGINT, Arc::clone(&pause_temp))?;

    let mut possible_hotkeys: Vec<config::Hotkey> = Vec::new();
    let mut last_hotkey: Option<LastHotkey> = None;
    let mut pollfds = keyboard_devices
        .iter()
        .map(|device| PollFd::new(device.device.as_raw_fd(), PollFlags::POLLIN))
        .collect::<Vec<_>>();

    let mut poll_timeout: i32 = -1;
    let mut main_loop = || {
        loop {
            if pause.load(Ordering::Relaxed) {
                break;
            }
            if reload.load(Ordering::Relaxed) {
                hotkeys = load_config();
                reload.store(false, Ordering::Relaxed);
            }
            match nix::poll::poll(&mut pollfds, poll_timeout) {
                Ok(_) => {}
                Err(_) => {
                    continue;
                }
            }
            for (i, _) in pollfds
                .iter()
                .enumerate()
                .filter(|(_, pollfd)| pollfd.revents() == Some(PollFlags::POLLIN))
            {
                let device = &mut keyboard_devices[i];
                for event in device.device.fetch_events().unwrap() {
                    if let InputEventKind::Key(key) = event.kind() {
                        match event.value() {
                            1 => {
                                if let Some(modifier) = modifiers_map.get(&key) {
                                    device.state_modifiers.insert(*modifier);
                                } else {
                                    device.state_keysyms.insert(key);
                                }
                            }
                            0 => {
                                if let Some(modifier) = modifiers_map.get(&key) {
                                    if let Some(hotkey) = &last_hotkey {
                                        if hotkey.hotkey.modifiers.contains(modifier) {
                                            last_hotkey = None;
                                        }
                                    }
                                    device.state_modifiers.remove(modifier);
                                } else if device.state_keysyms.contains(key) {
                                    if let Some(hotkey) = &last_hotkey {
                                        if key == hotkey.hotkey.keysym {
                                            last_hotkey = None;
                                        }
                                    }
                                    device.state_keysyms.remove(key);
                                }
                            }
                            _ => {}
                        }

                        if last_hotkey.is_some() {
                            continue;
                        }

                        for hotkey in &hotkeys {
                            if hotkey.modifiers.len() == device.state_modifiers.len() {
                                possible_hotkeys.push(hotkey.clone());
                            } else {
                                continue;
                            }
                        }

                        if possible_hotkeys.is_empty() {
                            continue;
                        }

                        log::error!("state_modifiers: {:#?}", device.state_modifiers);
                        log::error!("state_keysyms: {:#?}", device.state_keysyms);
                        log::error!("hotkey: {:#?}", possible_hotkeys);
                        if pause_temp.load(Ordering::Relaxed) {
                            if device.state_modifiers.iter().all(|x| {
                                vec![config::Modifier::Shift, config::Modifier::Super].contains(x)
                            }) && device.state_keysyms.contains(evdev::Key::KEY_ESC)
                            {
                                pause_temp.store(false, Ordering::Relaxed);
                            }
                            continue;
                        }

                        for hotkey in &possible_hotkeys {
                            // this should check if state_modifiers and hotkey.modifiers have the same elements
                            if device.state_modifiers.iter().all(|x| hotkey.modifiers.contains(x))
                                && device.state_modifiers.len() == hotkey.modifiers.len()
                                && device.state_keysyms.contains(hotkey.keysym)
                            {
                                last_hotkey = Some(LastHotkey {
                                    hotkey: hotkey.clone(),
                                    ran_at: SystemTime::now(),
                                });
                                send_command(hotkey.clone());
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(hotkey) = &mut last_hotkey {
                let time_since_ran_at = match SystemTime::now().duration_since(hotkey.ran_at) {
                    Ok(n) => n.as_millis(),
                    Err(e) => {
                        log::error!("Error: {:#?}", e);
                        exit(1);
                    }
                };
                if time_since_ran_at < repeat_cooldown_duration {
                    poll_timeout = (repeat_cooldown_duration - time_since_ran_at) as i32;
                } else {
                    send_command(hotkey.hotkey.clone());
                    poll_timeout = repeat_cooldown_duration as i32;
                    hotkey.ran_at = SystemTime::now();
                }
            } else {
                poll_timeout = -1;
            }

            possible_hotkeys.clear();
        }
    };

    loop {
        main_loop();
        while !resume.load(Ordering::Relaxed) {}
        pause.store(false, Ordering::Relaxed);
        resume.store(false, Ordering::Relaxed);
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
