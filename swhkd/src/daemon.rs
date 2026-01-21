use crate::config::Value;
use clap::Parser;
use config::Hotkey;
use evdev::{AttributeSet, Device, InputEventKind, Key, RelativeAxisType};
use nix::{
    sys::stat::{umask, Mode},
    unistd::{setgid, setuid, Gid, Uid, User},
};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    ffi::CString,
    fs::{self, File, OpenOptions, Permissions},
    io::{Read, Write},
    os::unix::{fs::PermissionsExt, net::UnixStream, process::CommandExt},
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
    #[arg(short = 'C', long, default_value_t = 250)]
    cooldown: u64,

    /// Enable Debug Mode
    #[arg(short, long)]
    debug: bool,

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
    env::set_var("RUST_LOG", "swhkd=warn");

    if args.debug {
        env::set_var("RUST_LOG", "swhkd=trace");
    }

    env_logger::init();
    log::trace!("Logger initialized.");

    // Just to double check that we are in root
    perms::raise_privileges();

    // Get the UID of the user that is not a system user
    let invoking_uid = get_uid()?;

    // Calculate a server cooldown at which the server will be pinged to check for env changes.
    let cooldown = args.cooldown;
    let delta = (cooldown as f64 * 0.1) as u64;
    let server_cooldown = std::cmp::max(0, cooldown - delta);

    log::debug!("Waiting for swhks socket...");
    // The first and the most important request for the env
    // Without this request, the environmental variables responsible for the reading for the config
    // file will not be available.
    // Thus, it is important to wait for the server to start before proceeding.
    let env;
    let mut env_hash;
    loop {
        match refresh_env(invoking_uid, 0) {
            Ok((Some(new_env), hash)) => {
                env_hash = hash;
                env = new_env;
                break;
            }
            Ok((None, _)) => {
                sleep(Duration::from_millis(server_cooldown)).await;
                continue;
            }
            Err(_) => {}
        }
    }
    log::trace!("Environment Aquired");

    // Now that we have the env, we can safely proceed with the rest of the program.
    // Log handling
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

    // Set up a channel to communicate with the server
    // The channel can have upto 100 commands in the queue
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // We use a arc mutex to make sure that our pairs are valid and also concurrent
    // while being used by the threads.
    let pairs = Arc::new(Mutex::new(env.pairs.clone()));
    let pairs_clone = Arc::clone(&pairs);
    let log = log_path.clone();

    // We spawn a new thread in the user space to act as the execution thread
    // This again has a thread for running the env refresh module when a change is detected from
    // the server.
    tokio::spawn(async move {
        // This is the thread that is responsible for refreshing the env
        // It's sleep time is determined by the server cooldown.
        tokio::spawn(async move {
            loop {
                {
                    let mut pairs = pairs_clone.lock().unwrap();
                    match refresh_env(invoking_uid, env_hash) {
                        Ok((Some(env), hash)) => {
                            pairs.clone_from(&env.pairs);
                            env_hash = hash;
                        }
                        Ok((None, hash)) => {
                            env_hash = hash;
                        }
                        Err(e) => {
                            log::error!("Error: {}", e);
                            _ = Command::new("notify-send").arg(format!("ERROR {}", e)).spawn();
                            exit(1);
                        }
                    }
                }
                sleep(Duration::from_millis(server_cooldown)).await;
            }
        });

        // When we do receive a command, we spawn a new thread to execute the command
        // This thread is spawned in the user space and is used to execute the command and it
        // exits after the command is executed.
        while let Some(command) = rx.recv().await {
            // Clone the arc references to be used in the thread
            let pairs = pairs.clone();
            let log = log.clone();

            // Command execution with user privileges
            let user_uid = Uid::from_raw(invoking_uid);
            let user = User::from_uid(user_uid)
                .expect("Failed to get user info")
                .expect(&format!("User with UID {} not found", invoking_uid));

            let username =
                CString::new(user.name.as_str()).expect("Failed to convert username to CString");

            let mut cmd = Command::new("sh");
            cmd.arg("-c")
                .arg(command)
                .uid(invoking_uid)
                .gid(user.gid.as_raw())
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

            // Set the environment variables for the command
            for (key, value) in pairs.lock().unwrap().iter() {
                cmd.env(key, value);
            }

            match cmd.spawn() {
                Ok(_) => {
                    log::info!("Command executed successfully.");
                }
                Err(e) => log::error!("Failed to execute command: {}", e),
            }
        }
    });

    // With the threads responsible for refresh and execution being in place, we can finally
    // start the main loop of the program.
    setup_swhkd(invoking_uid, env.xdg_runtime_dir(invoking_uid));

    let config_file_path: PathBuf =
        args.config.as_ref().map_or_else(|| env.fetch_xdg_config_path(), |file| file.clone());
    let load_config = || {
        log::debug!("Using config file path: {:#?}", config_file_path);

        match config::load(&config_file_path) {
            Err(e) => {
                log::error!("Config Error: {}", e);
                if let Some(error_source) = e.source() {
                    log::error!("{}", error_source);
                }
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
            log::error!("Failed to create uinput device: \nErr: {:#?}", e);
            exit(1);
        }
    };

    let mut uinput_switches_device = match uinput::create_uinput_switches_device() {
        Ok(dev) => dev,
        Err(e) => {
            log::error!("Failed to create uinput switches device: \nErr: {:#?}", e);
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

    let repeat_cooldown_duration: u64 = args.cooldown;

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
    if device.supported_keys().is_some_and(|keys| keys.contains(Key::KEY_ENTER)) {
        if device.name() == Some("swhkd virtual output") {
            return false;
        }
        // Exclude devices with relative axes (mice)
        if device.supported_relative_axes().is_some_and(|axes| axes.contains(RelativeAxisType::REL_X) && axes.contains(RelativeAxisType::REL_Y)) {
            log::trace!("Mouse: {}", device.name().unwrap(),);
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
    let mut command = hotkey.command;
    if modes[*mode_stack.last().unwrap()].options.oneoff {
        mode_stack.pop();
    }
    for mode in hotkey.mode_instructions.iter() {
        match mode {
            sweet::ModeInstruction::Enter(name) => {
                if let Some(mode_index) = modes.iter().position(|modename| modename.name.eq(name)) {
                    mode_stack.push(mode_index);
                    log::info!("Entering mode: {}", name);
                }
            }
            sweet::ModeInstruction::Escape => {
                mode_stack.pop();
            }
        }
    }
    if command.ends_with(" &&") {
        command = command.strip_suffix(" &&").unwrap().to_string();
    }

    match tx.send(command).await {
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
    if uid == u32::MAX {
        // loginuid == u32::MAX ('-1' in unsigned long), means the value is unset
        return Err(format!("loginuid not set for process {}", std::process::id()).into());
    }

    Ok(uid)
}

fn get_file_paths(runtime_dir: &str) -> (String, String) {
    let pid_file_path = format!("{}/swhks.pid", runtime_dir);
    let sock_file_path = format!("{}/swhkd.sock", runtime_dir);

    (pid_file_path, sock_file_path)
}

/// Refreshes the environment variables from the server
pub fn refresh_env(
    invoking_uid: u32,
    prev_hash: u64,
) -> Result<(Option<environ::Env>, u64), Box<dyn Error>> {
    // A simple placeholder for the env that is to be refreshed
    let env = environ::Env::construct(None);

    let (_pid_path, sock_path) =
        get_file_paths(env.xdg_runtime_dir(invoking_uid).to_str().unwrap());

    let mut buff: String = String::new();

    // Follows a two part process to recieve the env hash and the env itself
    // First part: Send a "1" as a byte to the socket to request the hash
    log::debug!("reading from socket {}", sock_path);
    match UnixStream::connect(&sock_path) {
        Ok(mut stream) => {
            let n = stream.write(&[1])?;
            if n != 1 {
                log::error!("Failed to write to socket {}", sock_path);
                return Ok((None, prev_hash));
            }
            stream.read_to_string(&mut buff)?;
        }
        Err(e) => {
            log::warn!("Failed to connect to socket {}: {}", sock_path, e);
            return Ok((None, prev_hash));
        }
    }

    let env_hash = buff.parse().unwrap_or_default();

    // If the hash is the same as the previous hash, return early
    // no need to refresh the env
    if env_hash == prev_hash {
        return Ok((None, prev_hash));
    }

    // Now that we know the env hash is different, we can request the env
    // Second part: Send a "2" as a byte to the socket to request the env
    if let Ok(mut stream) = UnixStream::connect(&sock_path) {
        let n = stream.write(&[2])?;
        if n != 1 {
            log::error!("Failed to write to socket.");
            return Ok((None, prev_hash));
        }

        // Clear the buffer before reading
        buff.clear();
        stream.read_to_string(&mut buff)?;
    }

    log::info!("Env refreshed");

    // Construct the env from the recieved env and return it
    Ok((Some(environ::Env::construct(Some(&buff))), env_hash))
}
