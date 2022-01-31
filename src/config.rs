use std::{path, fs};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    ConfigNotFound,
    Io(std::io::Error),
    InvalidConfig
}

impl From<std::io::Error> for Error {
    fn from(val: std::io::Error) -> Self {
        if val.kind() == std::io::ErrorKind::NotFound {
            Error::ConfigNotFound
        } else {
            Error::Io(val)
        }
    }
}

#[derive(Debug)]
pub struct Keybind {
    pressed_keys: Vec<evdev::Key>,
    command: String
}

impl Keybind {
    fn new(pressed_keys: Vec<evdev::Key>, command: String) -> Self {
        Keybind {
            pressed_keys,
            command
        }
    }
}

pub fn parse_config(path: path::PathBuf) -> Result<Vec<Keybind>, Error> {

    // Find file
    let mut file = File::open(path)?;

    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    // Parse file line-by-line
    let key_to_evdev_key: HashMap<&str, evdev::Key> = HashMap::from([
        ("q", evdev::Key::KEY_Q),
        ("w", evdev::Key::KEY_W),
        ("e", evdev::Key::KEY_E),
        ("r", evdev::Key::KEY_R),
        ("t", evdev::Key::KEY_T),
        ("y", evdev::Key::KEY_Y),
        ("u", evdev::Key::KEY_U),
        ("i", evdev::Key::KEY_I),
        ("o", evdev::Key::KEY_O),
        ("p", evdev::Key::KEY_P),
    ]);

    let mut keybinds: Vec<Keybind> = Vec::new();
    let lines: Vec<&str> = contents.split("\n").collect();

    for i in 0..lines.len() {
        let mut key_presses: Vec<evdev::Key> = Vec::new();

        if key_to_evdev_key.contains_key(lines[i].trim()) {
            // Translate keypress into evdev key
            let key_press = key_to_evdev_key.get(lines[i].trim()).unwrap();

            key_presses.push(*key_press);

            // Then interpret the command (simple)
            let command = lines[i + 1].trim();

            // Push a new keybind to the keybinds vector
            keybinds.push(Keybind::new(key_presses, String::from(command)));
        }

        // If there are config errors, return a config ParseError
    }


    // If all is ok, return Vec<Keybind>
    return Ok(keybinds);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Implement a struct for a path used in tests
    // so that the test file will be automatically removed
    // no matter how the test goes
    struct TestPath {
        path: path::PathBuf
    }

    impl TestPath {
        fn new(path: &str) -> Self {
            TestPath {
                path: path::PathBuf::from(path)
            }
        }

        // Create a path method for a more succinct way
        // to deal with borrowing the path value
        fn path(&self) -> path::PathBuf {
            self.path.clone()
        }
    }

    impl Drop for TestPath {
        fn drop(self: &mut TestPath) {
            fs::remove_file(self.path());
        }
    }

    #[test]
    fn test_nonexistent_file() {
        let path = path::PathBuf::from(r"This File Doesn't Exit");

        let parse_result = parse_config(path);

        assert!(parse_result.is_err());
    }

    #[test]
    fn test_existing_file() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file1");
        // Build a dummy file in /tmp
        let mut f = File::create(setup.path())?;
        f.write_all(b"
x
    dmenu_run

q
    bspc node -q")?;

        let parse_result = parse_config(setup.path());
        assert!(parse_result.is_ok());

        Ok(())
    }

    #[test]
    fn test_basic_keybind() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file2");
        let mut f = File::create(setup.path())?;
        f.write_all(b"
r
    alacritty
        ")?;

        // Process the expected result
        let expected_keybind = Keybind::new(vec![evdev::Key::KEY_R],
                                            String::from("alacritty"));

        // Process the real result
        let parse_result = parse_config(setup.path());

        assert!(parse_result.is_ok());

        let parse_result = parse_result.unwrap();

        assert_eq!(parse_result[0].pressed_keys,
                   expected_keybind.pressed_keys);

        assert_eq!(parse_result[0].command,
                   expected_keybind.command);

        Ok(())
    }

    #[test]
    fn test_multiple_keybinds() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file3");
        let mut f = File::create(setup.path())?;
        f.write_all(b"
r
    alacritty

w
    kitty

t
    /bin/firefox
        ")?;

        let keybind_1 = Keybind::new(vec![evdev::Key::KEY_R],
                                     String::from("alacritty"));
        let keybind_2 = Keybind::new(vec![evdev::Key::KEY_W],
                                     String::from("kitty"));
        let keybind_3 = Keybind::new(vec![evdev::Key::KEY_T],
                                     String::from("/bin/firefox"));

        let result = parse_config(setup.path());
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(keybind_1.pressed_keys, result[0].pressed_keys);
        assert_eq!(keybind_1.command, result[0].command);
        assert_eq!(keybind_2.pressed_keys, result[1].pressed_keys);
        assert_eq!(keybind_2.command, result[1].command);
        assert_eq!(keybind_3.pressed_keys, result[2].pressed_keys);
        assert_eq!(keybind_3.command, result[2].command);

        Ok(())
    }

    #[test]
    fn test_comments() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file4");
        let mut f = File::create(setup.path())?;
        f.write_all(b"
r
    alacritty

w
    kitty

#t
    #/bin/firefox
        ")?;

        let expected_keybinds = vec![
            Keybind::new(vec![evdev::Key::KEY_R],
                         String::from("alacritty")),
            Keybind::new(vec![evdev::Key::KEY_W],
                         String::from("kitty")),
        ];

        let result = parse_config(setup.path());
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result.len(), 2);

        assert_eq!(result[0].pressed_keys, expected_keybinds[0].pressed_keys);
        assert_eq!(result[0].command, expected_keybinds[0].command);
        assert_eq!(result[1].pressed_keys, expected_keybinds[1].pressed_keys);
        assert_eq!(result[1].command, expected_keybinds[1].command);

        Ok(())
    }

    #[ignore]
    fn test_multiple_keypress() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file5");
        let mut f = File::create(setup.path())?;
        f.write_all(b"
super + 5
    alacritty
        ")?;

        let expected_keybinds = vec![

            // I don't know if the super key is macro or not,
            // please check
            Keybind::new(vec![evdev::Key::KEY_MACRO, evdev::Key::KEY_5],
                         String::from("alacritty")),
        ];

        let result = parse_config(setup.path());
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result[0].pressed_keys, expected_keybinds[0].pressed_keys);
        assert_eq!(result[0].command, expected_keybinds[0].command);

        Ok(())
    }

    #[test]
    fn test_command_with_many_spaces() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file6");
        let mut f = File::create(setup.path())?;
        f.write_all(b"
p
    xbacklight -inc 10 -fps 30 -time 200
        ")?;

        let expected_keybinds = vec![
            Keybind::new(vec![evdev::Key::KEY_P],
                         String::from("xbacklight -inc 10 -fps 30 -time 200")),
        ];

        let result = parse_config(setup.path());
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result[0].pressed_keys, expected_keybinds[0].pressed_keys);
        assert_eq!(result[0].command, expected_keybinds[0].command);

        Ok(())
    }
}
