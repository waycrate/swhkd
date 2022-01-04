use evdev::Device;
use evdev::Key;
use glob::glob;

pub fn main() {
    for entry in glob("/dev/input/event*").expect("Failed to read /dev/input/event*.") {
        match entry {
            Ok(path) => {
                check_keyboard(path.into_os_string().into_string().unwrap());
            }
            Err(_) => continue,
        }
    }
}

fn check_keyboard(input_path: String) -> bool {
    let device = Device::open(&input_path).expect("Failed to open device.");
    if device
        .supported_keys()
        .map_or(false, |keys| keys.contains(Key::KEY_ENTER))
    {
        println!("{} is a keyboard.", input_path);
        return true;
    } else {
        println!("{} is not a keyboard.", input_path);
        return false;
    }
}
