use clap::{arg, Command};
use evdev::{AttributeSet, Device, InputEventKind, Key};
use nix::unistd::{Group, Uid};
use signal_hook_tokio::Signals;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::prelude::*,
    os::unix::net::UnixStream,
    path::Path,
    process::{exit, id},
};
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::select;
use tokio::time::Duration;
use tokio::time::{sleep, Instant};
use tokio_stream::{StreamExt, StreamMap};

use signal_hook::consts::signal::*;

mod config;
use crate::config::Value;
mod uinput;

#[cfg(test)]
mod tests;

struct KeyboardState {
    state_modifiers: HashSet<config::Modifier>,
    state_keysyms: AttributeSet<evdev::Key>,
}

impl KeyboardState {
    fn new() -> KeyboardState {
        KeyboardState { state_modifiers: HashSet::new(), state_keysyms: AttributeSet::new() }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = set_command_line_args().get_matches();
    let invoking_uid = env::var("PKEXEC_UID").unwrap().parse::<u32>().unwrap();
    env::set_var("RUST_LOG", "swhkd=warn");

    if args.is_present("debug") {
        env::set_var("RUST_LOG", "swhkd=trace");
    }

    env_logger::init();
    log::trace!("Logger initialized.");

    let pidfile: String = String::from(format!("/etc/swhkd/runtime/swhkd_{}.pid", invoking_uid));
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
                log::error!("pid of existing swhkd process: {}", pid.to_string());
                log::error!("To close the existing swhkd process, run `sudo killall swhkd`");
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

    if check_user_permissions().is_err() {
        exit(1);
    }

    let load_config = || {
        seteuid(invoking_uid); // Dropping privileges to invoking user.
        let config_file_path: std::path::PathBuf = if args.is_present("config") {
            Path::new(args.value_of("config").unwrap()).to_path_buf()
        } else {
            fetch_xdg_config_path()
        };

        log::debug!("Using config file path: {:#?}", config_file_path);

        let hotkeys = match config::load(&config_file_path) {
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
    seteuid(0); // Escalating back to root after reading config file.
    log::trace!("Attempting to find all keyboard file descriptors.");
    let keyboard_devices: Vec<Device> =
        evdev::enumerate().filter(check_device_is_keyboard).collect();

    let mut uinput_device = match uinput::create_uinput_device() {
        Ok(dev) => dev,
        Err(e) => {
            log::error!("Err: {:#?}", e);
            exit(1);
        }
    };

    if keyboard_devices.is_empty() {
        log::error!("No valid keyboard device was detected!");
        exit(1);
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

    let repeat_cooldown_duration: u64 = if args.is_present("cooldown") {
        args.value_of("cooldown").unwrap().parse::<u64>().unwrap()
    } else {
        250
    };

    let mut signals = Signals::new(&[
        SIGUSR1, SIGUSR2, SIGHUP, SIGABRT, SIGBUS, SIGCHLD, SIGCONT, SIGINT, SIGPIPE, SIGQUIT,
        SIGSYS, SIGTERM, SIGTRAP, SIGTSTP, SIGVTALRM, SIGXCPU, SIGXFSZ,
    ])?;

    let mut execution_is_paused = false;
    let mut last_hotkey: Option<config::Hotkey> = None;
    let mut pending_release: bool = false;
    let mut keyboard_states: Vec<KeyboardState> = Vec::new();
    let mut keyboard_stream_map = StreamMap::new();

    for (i, mut device) in keyboard_devices.into_iter().enumerate() {
        let _ = device.grab();
        keyboard_stream_map.insert(i, device.into_event_stream()?);
        keyboard_states.push(KeyboardState::new());
    }

    // The initial sleep duration is never read because last_hotkey is initialized to None
    let hotkey_repeat_timer = sleep(Duration::from_millis(0));
    tokio::pin!(hotkey_repeat_timer);

    loop {
        select! {
            _ = &mut hotkey_repeat_timer, if &last_hotkey.is_some() => {
                let hotkey = last_hotkey.clone().unwrap();
                if hotkey.keybinding.on_release {
                    continue;
                }
                send_command(hotkey.clone());
                hotkey_repeat_timer.as_mut().reset(Instant::now() + Duration::from_millis(repeat_cooldown_duration));
            }

            Some(signal) = signals.next() => {
                match signal {
                    SIGUSR1 => {
                        execution_is_paused = true;
                        for mut device in evdev::enumerate().filter(check_device_is_keyboard) {
                            let _ = device.ungrab();
                        }
                    }

                    SIGUSR2 => {
                        execution_is_paused = false;
                        for mut device in evdev::enumerate().filter(check_device_is_keyboard) {
                            let _ = device.grab();
                        }
                    }

                    SIGHUP => {
                        hotkeys = load_config();
                    }

                    SIGINT => {
                        for mut device in evdev::enumerate().filter(check_device_is_keyboard) {
                            let _ = device.ungrab();
                        }
                        log::warn!("Received SIGINT signal, exiting...");
                        exit(1);
                    }

                    _ => {
                        for mut device in evdev::enumerate().filter(check_device_is_keyboard) {
                            let _ = device.ungrab();
                        }

                        log::warn!("Received signal: {:#?}", signal);
                        log::warn!("Exiting...");
                        exit(1);
                    }
                }
            }

            Some((i, Ok(event))) = keyboard_stream_map.next() => {
                let keyboard_state = &mut keyboard_states[i];

                let key = match event.kind() {
                    InputEventKind::Key(keycode) => keycode,
                    _ => continue
                };

                match event.value() {
                    // Key press
                    1 => {
                        if let Some(modifier) = modifiers_map.get(&key) {
                            keyboard_state.state_modifiers.insert(*modifier);
                        } else {
                            keyboard_state.state_keysyms.insert(key);
                        }
                    }

                    // Key release
                    0 => {
                        if last_hotkey.is_some() && pending_release {
                            pending_release = false;
                            send_command(last_hotkey.clone().unwrap());
                            last_hotkey = None;
                        }
                        if let Some(modifier) = modifiers_map.get(&key) {
                            if let Some(hotkey) = &last_hotkey {
                                if hotkey.modifiers().contains(modifier) {
                                    last_hotkey = None;
                                }
                            }
                            keyboard_state.state_modifiers.remove(modifier);
                        } else if keyboard_state.state_keysyms.contains(key) {
                            if let Some(hotkey) = &last_hotkey {
                                if key == hotkey.keysym() {
                                    last_hotkey = None;
                                }
                            }
                            keyboard_state.state_keysyms.remove(key);
                        }
                    }

                    _ => {}
                }

                let possible_hotkeys: Vec<&config::Hotkey> = hotkeys.iter()
                    .filter(|hotkey| hotkey.modifiers().len() == keyboard_state.state_modifiers.len())
                    .collect();

                let event_in_hotkeys = hotkeys.iter().any(|hotkey| {
                    hotkey.keysym().code() == event.code() &&
                    keyboard_state.state_modifiers
                        .iter()
                        .all(|x| hotkey.modifiers().contains(x)) &&
                    keyboard_state.state_modifiers.len() == hotkey.modifiers().len()
                    && !hotkey.is_send()
                        });

                // Don't emit event to virtual device if it's from a valid hotkey
                if !event_in_hotkeys {
                    uinput_device.emit(&[event]).unwrap();
                }

                if execution_is_paused || possible_hotkeys.is_empty() || last_hotkey.is_some() {
                    continue;
                }

                log::debug!("state_modifiers: {:#?}", keyboard_state.state_modifiers);
                log::debug!("state_keysyms: {:#?}", keyboard_state.state_keysyms);
                log::debug!("hotkey: {:#?}", possible_hotkeys);

                for hotkey in possible_hotkeys {
                    // this should check if state_modifiers and hotkey.modifiers have the same elements
                    if keyboard_state.state_modifiers.iter().all(|x| hotkey.modifiers().contains(x))
                        && keyboard_state.state_modifiers.len() == hotkey.modifiers().len()
                        && keyboard_state.state_keysyms.contains(hotkey.keysym())
                    {
                        last_hotkey = Some(hotkey.clone());
                        if pending_release { break; }
                        if hotkey.is_on_release() {
                            pending_release = true;
                            break;
                        }
                        send_command(hotkey.clone());
                        hotkey_repeat_timer.as_mut().reset(Instant::now() + Duration::from_millis(repeat_cooldown_duration));
                        break;
                    }
                }
            }
        }
    }
}

fn sock_send(command: &str) -> std::io::Result<()> {
    let sock_file_path =
        String::from(format!("/run/user/{}/swhkd.sock", env::var("PKEXEC_UID").unwrap()));
    let mut stream = UnixStream::connect(sock_file_path)?;
    stream.write_all(command.as_bytes())?;
    Ok(())
}

fn send_command(hotkey: config::Hotkey) {
    log::info!("Hotkey pressed: {:#?}", hotkey);
    if let Err(e) = sock_send(&hotkey.command) {
        log::error!("Failed to send command to swhks through IPC.");
        log::error!("Please make sure that swhks is running.");
        log::error!("Err: {:#?}", e)
    }
}

pub fn check_user_permissions() -> Result<(), ()> {
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
        Err(())
    } else {
        log::warn!("Running swhkd as root!");
        Ok(())
    }
}

pub fn check_device_is_keyboard(device: &Device) -> bool {
    if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        if device.name() == Some("swhkd virtual output") {
            return false;
        }
        log::debug!("Keyboard: {}", device.name().unwrap(),);
        true
    } else {
        log::trace!("Other: {}", device.name().unwrap(),);
        false
    }
}

pub fn set_command_line_args() -> Command<'static> {
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

pub fn fetch_xdg_config_path() -> std::path::PathBuf {
    let config_file_path: std::path::PathBuf = match env::var("XDG_CONFIG_HOME") {
        Ok(val) => {
            log::debug!("XDG_CONFIG_HOME exists: {:#?}", val);
            Path::new(&val).join("swhkd/swhkdrc")
        }
        Err(_) => {
            log::error!("XDG_CONFIG_HOME has not been set.");
            Path::new("/etc/swhkd/swhkdrc").to_path_buf()
        }
    };
    config_file_path
}

pub fn seteuid(uid: u32) {
    let uid = Uid::from_raw(uid);
    match nix::unistd::seteuid(uid) {
        Ok(_) => log::debug!("Dropping privileges..."),
        Err(e) => {
            log::error!("Failed to set EUID: {:#?}", e);
            exit(1);
        }
    }
}
