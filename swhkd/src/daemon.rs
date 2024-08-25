use crate::config::Value;
use clap::Parser;
use config::Hotkey;
use evdev::{AttributeSet, Device, InputEventKind, Key};
use nix::{
    sys::stat::{umask, Mode},
    unistd::{setgid, setuid, Gid, Uid},
};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    fs::{self, File, OpenOptions, Permissions},
    io::Read,
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::{Path, PathBuf},
    process::{exit, id, Command, Stdio},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::time::Duration;
use tokio::time::{sleep, Instant};
use tokio::{select, sync::mpsc};
use tokio_stream::{StreamExt, StreamMap};
use tokio_udev::{AsyncMonitorSocket, EventType, MonitorBuilder};

mod config;
mod environ;
mod perms;
mod uinput;

struct KeyboardState {
    state_modifiers: HashSet<config::Modifier>,
    state_keysyms: AttributeSet<evdev::Key>,
}

impl KeyboardState {
    fn new() -> KeyboardState {
        KeyboardState { state_modifiers: HashSet::new(), state_keysyms: AttributeSet::new() }
    }
}

/// Simple Wayland Hotkey Daemon
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set a custom config file path.
    #[arg(short = 'c', long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Set a custom repeat cooldown duration. Default is 250ms.
    #[arg(short = 'C', long)]
    cooldown: Option<u64>,

    /// Enable Debug Mode
    #[arg(short, long)]
    debug: bool,

    /// Set Server Refresh Time in milliseconds
    #[arg(short, long)]
    refresh: Option<u64>,

    /// Take a list of devices from the user
    #[arg(short = 'D', long, num_args = 0.., value_delimiter = ' ')]
    device: Vec<String>,

    /// Set a custom log file. (Defaults to ${XDG_DATA_HOME:-$HOME/.local/share}/swhks-current_unix_time.log)
    #[arg(short, long, value_name = "FILE")]
    log: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let default_cooldown: u64 = 650;
    env::set_var("RUST_LOG", "swhkd=warn");

    if args.debug {
        env::set_var("RUST_LOG", "swhkd=trace");
    }

    env_logger::init();
    log::trace!("Logger initialized.");
    perms::raise_privileges();

    let invoking_uid = get_uid()?;
    let uname = get_uname_from_uid(invoking_uid)?;

    let env = refresh_env(&uname, invoking_uid).unwrap();
    log::trace!("Environment Aquired");
    let log_file_name = if let Some(val) = args.log {
        val
    } else {
        let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs().to_string(),
            Err(_) => {
                log::error!("SystemTime before UnixEpoch!");
                exit(1);
            }
        };

        format!("{}/swhkd/swhkd-{}.log", env.fetch_xdg_data_path().to_string_lossy(), time).into()
    };

    let log_path = PathBuf::from(&log_file_name);
    if let Some(p) = log_path.parent() {
        if !p.exists() {
            if let Err(e) = fs::create_dir_all(p) {
                log::error!("Failed to create log dir: {}", e);
            }
        }
    }
    // if file doesnt exist, create it with 0666 permissions
    if !log_path.exists() {
        if let Err(e) = OpenOptions::new().append(true).create(true).open(&log_path) {
            log::error!("Failed to create log file: {}", e);
            exit(1);
        }
        fs::set_permissions(&log_path, Permissions::from_mode(0o666)).unwrap();
    }

    // The server cool down is set to 650ms by default
    // which is calculated based on the default repeat cooldown
    // along with it, an additional 120ms is added to it, just to be safe.
    let server_cooldown = args.refresh.unwrap_or(default_cooldown + 1024);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    let pairs = Arc::new(Mutex::new(env.pairs.clone()));
    let pairs_clone = Arc::clone(&pairs);
    let log = log_path.clone();

    tokio::spawn(async move {
        tokio::spawn(async move {
            loop {
                {
                    let mut pairs = pairs_clone.lock().unwrap();
                    pairs.clone_from(&refresh_env(&uname, invoking_uid).unwrap().pairs);
                }
                sleep(Duration::from_millis(server_cooldown)).await;
            }
        });

        while let Some(command) = rx.recv().await {
            let pairs = pairs.clone();
            let log = log.clone();
            tokio::spawn(async move {
                setgid(Gid::from_raw(invoking_uid)).unwrap();
                setuid(Uid::from_raw(invoking_uid)).unwrap();

                let mut cmd = Command::new("sh");
                cmd.arg("-c")
                    .arg(command)
                    .stdin(Stdio::null())
                    .stdout(match File::open(&log) {
                        Ok(file) => file,
                        Err(e) => {
                            println!("Error: {}", e);
                            _ = Command::new("notify-send").arg(format!("ERROR {}", e)).spawn();
                            exit(1);
                        }
                    })
                    .stderr(match File::open(&log) {
                        Ok(file) => file,
                        Err(e) => {
                            println!("Error: {}", e);
                            _ = Command::new("notify-send").arg(format!("ERROR {}", e)).spawn();
                            exit(1);
                        }
                    });

                for (key, value) in pairs.lock().unwrap().iter() {
                    cmd.env(key, value);
                }

                match cmd.spawn() {
                    Ok(_) => {
                        log::info!("Command executed successfully.");
                    }
                    Err(e) => log::error!("Failed to execute command: {}", e),
                }
            });
        }
    });

    setup_swhkd(invoking_uid, env.xdg_runtime_dir(invoking_uid));

    let config_file_path: PathBuf =
        args.config.as_ref().map_or_else(|| env.fetch_xdg_config_path(), |file| file.clone());
    let load_config = || {
        log::debug!("Using config file path: {:#?}", config_file_path);

        match config::load(&config_file_path) {
            Err(e) => {
                log::error!("Config Error: {}", e);
                exit(1)
            }
            Ok(out) => out,
        }
    };

    let mut modes = load_config();
    let mut mode_stack: Vec<usize> = vec![0];
    let arg_devices: Vec<String> = args.device;

    let keyboard_devices: Vec<_> = {
        if arg_devices.is_empty() {
            log::trace!("Attempting to find all keyboard file descriptors.");
            evdev::enumerate().filter(|(_, dev)| check_device_is_keyboard(dev)).collect()
        } else {
            evdev::enumerate()
                .filter(|(_, dev)| arg_devices.contains(&dev.name().unwrap_or("").to_string()))
                .collect()
        }
    };

    if keyboard_devices.is_empty() {
        log::error!("No valid keyboard device was detected!");
        exit(1);
    }

    log::debug!("{} Keyboard device(s) detected.", keyboard_devices.len());

    // Apparently, having a single uinput device with keys, relative axes and switches
    // prevents some libraries to listen to these events. The easy fix is to have separate
    // virtual devices, one for keys and relative axes (`uinput_device`) and another one
    // just for switches (`uinput_switches_device`).
    let mut uinput_device = match uinput::create_uinput_device() {
        Ok(dev) => dev,
        Err(e) => {
            log::error!("Err: {:#?}", e);
            exit(1);
        }
    };

    let mut uinput_switches_device = match uinput::create_uinput_switches_device() {
        Ok(dev) => dev,
        Err(e) => {
            log::error!("Err: {:#?}", e);
            exit(1);
        }
    };

    let mut udev =
        AsyncMonitorSocket::new(MonitorBuilder::new()?.match_subsystem("input")?.listen()?)?;

    let modifiers_map: HashMap<Key, config::Modifier> = HashMap::from([
        (Key::KEY_LEFTMETA, config::Modifier::Super),
        (Key::KEY_RIGHTMETA, config::Modifier::Super),
        (Key::KEY_LEFTALT, config::Modifier::Alt),
        (Key::KEY_RIGHTALT, config::Modifier::Altgr),
        (Key::KEY_LEFTCTRL, config::Modifier::Control),
        (Key::KEY_RIGHTCTRL, config::Modifier::Control),
        (Key::KEY_LEFTSHIFT, config::Modifier::Shift),
        (Key::KEY_RIGHTSHIFT, config::Modifier::Shift),
    ]);

    let repeat_cooldown_duration: u64 = args.cooldown.unwrap_or(default_cooldown);

    let mut signals = Signals::new([
        SIGUSR1, SIGUSR2, SIGHUP, SIGABRT, SIGBUS, SIGCONT, SIGINT, SIGPIPE, SIGQUIT, SIGSYS,
        SIGTERM, SIGTRAP, SIGTSTP, SIGVTALRM, SIGXCPU, SIGXFSZ,
    ])?;

    let mut execution_is_paused = false;
    let mut last_hotkey: Option<config::Hotkey> = None;
    let mut pending_release: bool = false;
    let mut keyboard_states = HashMap::new();
    let mut keyboard_stream_map = StreamMap::new();

    for (path, mut device) in keyboard_devices.into_iter() {
        let _ = device.grab();
        let path = match path.to_str() {
            Some(p) => p,
            None => {
                continue;
            }
        };
        keyboard_states.insert(path.to_string(), KeyboardState::new());
        keyboard_stream_map.insert(path.to_string(), device.into_event_stream()?);
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
                send_command(hotkey.clone(), &modes, &mut mode_stack, tx.clone()).await;
                hotkey_repeat_timer.as_mut().reset(Instant::now() + Duration::from_millis(repeat_cooldown_duration));
            }



            Some(signal) = signals.next() => {
                match signal {
                    SIGUSR1 => {
                        execution_is_paused = true;
                        for mut device in evdev::enumerate().map(|(_, device)| device).filter(check_device_is_keyboard) {
                            let _ = device.ungrab();
                        }
                    }

                    SIGUSR2 => {
                        execution_is_paused = false;
                        for mut device in evdev::enumerate().map(|(_, device)| device).filter(check_device_is_keyboard) {
                            let _ = device.grab();
                        }
                    }

                    SIGHUP => {
                        modes = load_config();
                        mode_stack = vec![0];
                    }

                    SIGINT => {
                        for mut device in evdev::enumerate().map(|(_, device)| device).filter(check_device_is_keyboard) {
                            let _ = device.ungrab();
                        }
                        log::warn!("Received SIGINT signal, exiting...");
                        exit(1);
                    }

                    _ => {
                        for mut device in evdev::enumerate().map(|(_, device)| device).filter(check_device_is_keyboard) {
                            let _ = device.ungrab();
                        }

                        log::warn!("Received signal: {:#?}", signal);
                        log::warn!("Exiting...");
                        exit(1);
                    }
                }
            }

            Some(Ok(event)) = udev.next() => {
                if !event.is_initialized() {
                    log::warn!("Received udev event with uninitialized device.");
                }

                let node = match event.devnode() {
                    None => { continue; },
                    Some(node) => {
                        match node.to_str() {
                            None => { continue; },
                            Some(node) => node,
                        }
                    },
                };

                match event.event_type() {
                    EventType::Add => {
                        let mut device = match Device::open(node) {
                            Err(e) => {
                                log::error!("Could not open evdev device at {}: {}", node, e);
                                continue;
                            },
                            Ok(device) => device
                        };
                        let name = device.name().unwrap_or("[unknown]").to_string();
                        if arg_devices.contains(&name) || check_device_is_keyboard(&device) {
                            log::info!("Device '{}' at '{}' added.", name, node);
                            let _ = device.grab();
                            keyboard_states.insert(node.to_string(), KeyboardState::new());
                            keyboard_stream_map.insert(node.to_string(), device.into_event_stream()?);
                        }
                    }
                    EventType::Remove => {
                        if keyboard_stream_map.contains_key(node) {
                            keyboard_states.remove(node);
                            let stream = keyboard_stream_map.remove(node).expect("device not in stream_map");
                            let name = stream.device().name().unwrap_or("[unknown]");
                            log::info!("Device '{}' at '{}' removed", name, node);
                        }
                    }
                    _ => {
                        log::trace!("Ignored udev event of type: {:?}", event.event_type());
                    }
                }
            }

            Some((node, Ok(event))) = keyboard_stream_map.next() => {
                let keyboard_state = &mut keyboard_states.get_mut(&node).expect("device not in states map");

                let key = match event.kind() {
                    InputEventKind::Key(keycode) => keycode,
                    InputEventKind::Switch(_) => {
                        uinput_switches_device.emit(&[event]).unwrap();
                        continue
                    }
                    _ => {
                        uinput_device.emit(&[event]).unwrap();
                        continue
                    }
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
                            send_command(last_hotkey.clone().unwrap(), &modes, &mut mode_stack, tx.clone()).await;
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

                let possible_hotkeys: Vec<&config::Hotkey> = modes[mode_stack[mode_stack.len() - 1]].hotkeys.iter()
                    .filter(|hotkey| hotkey.modifiers().len() == keyboard_state.state_modifiers.len())
                    .collect();

                let event_in_hotkeys = modes[mode_stack[mode_stack.len() - 1]].hotkeys.iter().any(|hotkey| {
                    hotkey.keysym().code() == event.code() &&
                        (!keyboard_state.state_modifiers.is_empty() && hotkey.modifiers().contains(&config::Modifier::Any) || keyboard_state.state_modifiers
                        .iter()
                        .all(|x| hotkey.modifiers().contains(x)) &&
                    keyboard_state.state_modifiers.len() == hotkey.modifiers().len())
                    && !hotkey.is_send()
                        });

                // Only emit event to virtual device when swallow option is off
                if !modes[mode_stack[mode_stack.len()-1]].options.swallow
                // Don't emit event to virtual device if it's from a valid hotkey
                && !event_in_hotkeys {
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
                    if (!keyboard_state.state_modifiers.is_empty() && hotkey.modifiers().contains(&config::Modifier::Any) || keyboard_state.state_modifiers.iter().all(|x| hotkey.modifiers().contains(x))
                        && keyboard_state.state_modifiers.len() == hotkey.modifiers().len())
                        && keyboard_state.state_keysyms.contains(hotkey.keysym())
                    {
                        last_hotkey = Some(hotkey.clone());
                        if pending_release { break; }
                        if hotkey.is_on_release() {
                            pending_release = true;
                            break;
                        }
                        send_command(hotkey.clone(), &modes, &mut mode_stack, tx.clone()).await;
                        hotkey_repeat_timer.as_mut().reset(Instant::now() + Duration::from_millis(repeat_cooldown_duration));
                        continue;
                    }
                }
            }
        }
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

pub fn setup_swhkd(invoking_uid: u32, runtime_path: PathBuf) {
    // Set a sane process umask.
    log::trace!("Setting process umask.");
    umask(Mode::S_IWGRP | Mode::S_IWOTH);

    // Get the runtime path and create it if needed.
    if !Path::new(&runtime_path).exists() {
        match fs::create_dir_all(Path::new(&runtime_path)) {
            Ok(_) => {
                log::debug!("Created runtime directory.");
                match fs::set_permissions(Path::new(&runtime_path), Permissions::from_mode(0o600)) {
                    Ok(_) => log::debug!("Set runtime directory to readonly."),
                    Err(e) => log::error!("Failed to set runtime directory to readonly: {}", e),
                }
            }
            Err(e) => log::error!("Failed to create runtime directory: {}", e),
        }
    }

    // Get the PID file path for instance tracking.
    let pidfile: String = format!("{}/swhkd_{}.pid", runtime_path.to_string_lossy(), invoking_uid);
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

        // Check if swhkd is already running!
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

    // Write to the pid file.
    match fs::write(&pidfile, id().to_string()) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Unable to write to {}: {}", pidfile, e);
            exit(1);
        }
    }
}

pub async fn send_command(
    hotkey: Hotkey,
    modes: &[config::Mode],
    mode_stack: &mut Vec<usize>,
    tx: mpsc::Sender<String>,
) {
    log::info!("Hotkey pressed: {:#?}", hotkey);
    let command = hotkey.command;
    let mut commands_to_send = String::new();
    if modes[mode_stack[mode_stack.len() - 1]].options.oneoff {
        mode_stack.pop();
    }
    if command.contains('@') {
        let commands = command.split("&&").map(|s| s.trim()).collect::<Vec<_>>();
        for cmd in commands {
            let mut words = cmd.split_whitespace();
            match words.next().unwrap() {
                config::MODE_ENTER_STATEMENT => {
                    let enter_mode = cmd.split(' ').nth(1).unwrap();
                    for (i, mode) in modes.iter().enumerate() {
                        if mode.name == enter_mode {
                            mode_stack.push(i);
                            break;
                        }
                    }
                    log::info!("Entering mode: {}", modes[mode_stack[mode_stack.len() - 1]].name);
                }
                config::MODE_ESCAPE_STATEMENT => {
                    mode_stack.pop();
                }
                _ => commands_to_send.push_str(format!("{cmd} &&").as_str()),
            }
        }
    } else {
        commands_to_send = command;
    }
    if commands_to_send.ends_with(" &&") {
        commands_to_send = commands_to_send.strip_suffix(" &&").unwrap().to_string();
    }

    //launch(commands_to_send, user_id, env, log_path);
    match tx.send(commands_to_send).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to send command: {}", e);
        }
    }
}

/// Get the UID of the user that is not a system user
fn get_uid() -> Result<u32, Box<dyn Error>> {
    let status_content = fs::read_to_string(format!("/proc/{}/loginuid", std::process::id()))?;
    let uid = status_content.trim().parse::<u32>()?;
    Ok(uid)
}

fn get_uname_from_uid(uid: u32) -> Result<String, Box<dyn Error>> {
    let passwd = fs::read_to_string("/etc/passwd").unwrap();
    let lines: Vec<&str> = passwd.split('\n').collect();
    for line in lines {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() > 2 {
            let Ok(user_id) = parts[2].parse::<u32>() else {
                continue;
            };
            if user_id == uid {
                return Ok(parts[0].to_string());
            }
        }
    }
    Err("User not found".into())
}

fn get_file_paths(runtime_dir: &str) -> (String, String) {
    let pid_file_path = format!("{}/swhks.pid", runtime_dir);
    let sock_file_path = format!("{}/swhkd.sock", runtime_dir);

    (pid_file_path, sock_file_path)
}

fn refresh_env(uname: &str, invoking_uid: u32) -> Result<environ::Env, Box<dyn Error>> {
    let env = environ::Env::construct(uname, None);

    let (_pid_path, sock_path) =
        get_file_paths(env.xdg_runtime_dir(invoking_uid).to_str().unwrap());

    if Path::new(&sock_path).exists() {
        fs::remove_file(&sock_path)?;
    }

    let mut result: String = String::new();
    let listener = UnixListener::bind(&sock_path)?;
    fs::set_permissions(sock_path, fs::Permissions::from_mode(0o666))?;
    loop {
        log::warn!("Waiting for Server...");
        match listener.accept() {
            Ok((mut socket, _addr)) => {
                let mut buf = String::new();
                socket.read_to_string(&mut buf)?;
                if buf.is_empty() {
                    continue;
                }
                log::info!("Server Instance found!");
                result.push_str(&buf);
                break;
            }
            Err(e) => {
                log::info!("Sock Err: {}", e);
            }
        }
    }
    log::trace!("Environment Refreshed");
    Ok(environ::Env::construct(uname, Some(&result)))
}
