use evdev::Device;
use evdev::Key;
use glob::glob;
use nix::unistd;
use std::process::exit;

pub fn main() {
    std::env::set_var("RUST_LOG", "trace");
    env_logger::init();
    log::trace!("Logger initialized.");

    if unistd::Uid::current().is_root() {
        log::error!("Refusing to run swhkd as root.");
        exit(1);
    }

    log::trace!("Checking if invoking user is in input group or not.");
    let groups = unistd::getgroups();
    let mut is_in_input_group: bool = false;
    'init_loop: for (_, groups) in groups.iter().enumerate() {
        for group in groups {
            let group = unistd::Group::from_gid(*group);
            if group.unwrap().unwrap().name == "input" {
                log::debug!("Invoking user is in input group.");
                is_in_input_group = true;
                break 'init_loop;
            }
        }
    }

    if is_in_input_group == false {
        log::error!("Invoking user is NOT in input group.");
        exit(1);
    }

    log::trace!("Attempting to find all keyboard file descriptors.");
    for entry in glob("/dev/input/event*").expect("Failed to read /dev/input/event*.") {
        match entry {
            Ok(path) => {
                check_keyboard(path.into_os_string().into_string().unwrap());
            }
            Err(error) => log::error!("Unexpected error occured: {}", error),
        }
    }
}

pub fn check_keyboard(input_path: String) -> bool {
    /* Try to open the device fd. */
    let device = Device::open(&input_path);
    match &device {
        Ok(_) => (),
        Err(error) => {
            log::error!("Failed to open device {}: {}", input_path, error);
        }
    }

    /* Check for the presence of enter key. */
    if device
        .unwrap()
        .supported_keys()
        .map_or(false, |keys| keys.contains(Key::KEY_ENTER))
    {
        log::debug!("{} is a keyboard.", input_path);
        return true;
    } else {
        log::debug!("{} is not a keyboard.", input_path);
        return false;
    }
}
