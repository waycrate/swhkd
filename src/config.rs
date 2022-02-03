use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::{fs, path};

#[derive(Debug)]
pub enum Error {
    ConfigNotFound,
    Io(std::io::Error),
    InvalidConfig(ParseError),
}

#[derive(Debug)]
pub enum ParseError {
    // u32 is the line number where an error occured
    UnknownSymbol(u32),
    MissingCommand(u32),
    CommandWithoutWhitespace(u32),
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
pub struct Hotkey {
    keysyms: Vec<evdev::Key>,
    command: String,
}

impl Hotkey {
    fn new(keysyms: Vec<evdev::Key>, command: String) -> Self {
        Hotkey { keysyms, command }
    }
}

pub fn load(path: path::PathBuf) -> Result<Vec<Hotkey>, Error> {
    let file_contents = load_file_contents(path)?;
    parse_contents(file_contents)
}

fn load_file_contents(path: path::PathBuf) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_contents(contents: String) -> Result<Vec<Hotkey>, Error> {
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
        ("a", evdev::Key::KEY_A),
        ("s", evdev::Key::KEY_S),
        ("d", evdev::Key::KEY_D),
        ("f", evdev::Key::KEY_F),
        ("g", evdev::Key::KEY_G),
        ("h", evdev::Key::KEY_H),
        ("j", evdev::Key::KEY_J),
        ("k", evdev::Key::KEY_K),
        ("l", evdev::Key::KEY_L),
        ("z", evdev::Key::KEY_Z),
        ("x", evdev::Key::KEY_X),
        ("c", evdev::Key::KEY_C),
        ("v", evdev::Key::KEY_V),
        ("b", evdev::Key::KEY_B),
        ("n", evdev::Key::KEY_N),
        ("m", evdev::Key::KEY_M),
        ("1", evdev::Key::KEY_1),
        ("2", evdev::Key::KEY_2),
        ("3", evdev::Key::KEY_3),
        ("4", evdev::Key::KEY_4),
        ("5", evdev::Key::KEY_5),
        ("6", evdev::Key::KEY_6),
        ("7", evdev::Key::KEY_7),
        ("8", evdev::Key::KEY_8),
        ("9", evdev::Key::KEY_9),
        ("0", evdev::Key::KEY_0),
    ]);

    let lines: Vec<&str> = contents.split('\n').collect();
    let mut hotkeys: Vec<Hotkey> = Vec::new();

    let mut lines_to_skip: u32 = 0;

    // Parse file line-by-line
    for i in 0..lines.len() {
        if lines_to_skip > 0 {
            lines_to_skip -= 1;
            continue;
        }

        // Ignore blank lines and comments starting with #
        if lines[i].trim().is_empty() || lines[i].trim().starts_with('#') {
            continue;
        }

        // We need to get the real line number for errors
        // because arrays in Rust are zero-index while lines
        // in a file are of course counted from 1
        let real_line_no: u32 = (i + 1).try_into().unwrap();

        let mut key_presses: Vec<evdev::Key> = Vec::new();

        if key_to_evdev_key.contains_key(lines[i].trim()) {
            // If the keybind line is at the very last line,
            // it's impossible for there to be a command
            if i >= lines.len() - 1 {
                return Err(Error::InvalidConfig(ParseError::MissingCommand(real_line_no)));
            }

            // Translate keypress into evdev key
            let key_press = key_to_evdev_key.get(lines[i].trim()).unwrap();

            key_presses.push(*key_press);

            //// Find the command
            if lines[i + 1].trim().is_empty() {
                return Err(Error::InvalidConfig(ParseError::MissingCommand(real_line_no)));
            }

            let command = lines[i + 1].trim();

            // Push a new hotkey to the hotkeys vector
            hotkeys.push(Hotkey::new(key_presses, String::from(command)));

            // Skip trying to parse the next line (command)
            // because we already dealt with it
            lines_to_skip += 1;
        } else {
            return Err(Error::InvalidConfig(ParseError::UnknownSymbol(real_line_no)));
        }
    }

    // If all is ok, return Vec<Hotkeys>
    Ok(hotkeys)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Implement a struct for a path used in tests
    // so that the test file will be automatically removed
    // no matter how the test goes
    struct TestPath {
        path: path::PathBuf,
    }

    impl TestPath {
        fn new(path: &str) -> Self {
            TestPath { path: path::PathBuf::from(path) }
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
        let path = path::PathBuf::from(r"This File Doesn't Exist");

        let result = load_file_contents(path);

        assert!(result.is_err());

        match result.unwrap_err() {
            Error::ConfigNotFound => {
                return;
            }
            _ => {
                panic!("Error type for nonexistent file is wrong.");
            }
        }
    }

    #[test]
    fn test_existing_file() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file1");
        // Build a dummy file in /tmp
        let mut f = File::create(setup.path())?;
        f.write_all(
            b"
x
    dmenu_run

q
    bspc node -q",
        )?;

        let result = load_file_contents(setup.path());
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_basic_keybind() -> std::io::Result<()> {
        let contents = "
r
    alacritty
            ";

        let expected_hotkey = Hotkey::new(vec![evdev::Key::KEY_R], String::from("alacritty"));

        let parse_result = parse_contents(contents.to_string());

        assert!(parse_result.is_ok());

        let parse_result = parse_result.unwrap();

        assert_eq!(parse_result[0].keysyms, expected_hotkey.keysyms);

        assert_eq!(parse_result[0].command, expected_hotkey.command);

        Ok(())
    }

    #[test]
    fn test_multiple_keybinds() -> std::io::Result<()> {
        let contents = "
r
    alacritty

w
    kitty

t
    /bin/firefox
        ";

        let hotkey_1 = Hotkey::new(vec![evdev::Key::KEY_R], String::from("alacritty"));
        let hotkey_2 = Hotkey::new(vec![evdev::Key::KEY_W], String::from("kitty"));
        let hotkey_3 = Hotkey::new(vec![evdev::Key::KEY_T], String::from("/bin/firefox"));

        let result = parse_contents(contents.to_string());
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(hotkey_1.keysyms, result[0].keysyms);
        assert_eq!(hotkey_1.command, result[0].command);
        assert_eq!(hotkey_2.keysyms, result[1].keysyms);
        assert_eq!(hotkey_2.command, result[1].command);
        assert_eq!(hotkey_3.keysyms, result[2].keysyms);
        assert_eq!(hotkey_3.command, result[2].command);

        Ok(())
    }

    #[test]
    fn test_comments() -> std::io::Result<()> {
        let contents = "
r
    alacritty

w
    kitty

#t
    #/bin/firefox
        ";

        let expected_keybinds = vec![
            Hotkey::new(vec![evdev::Key::KEY_R], String::from("alacritty")),
            Hotkey::new(vec![evdev::Key::KEY_W], String::from("kitty")),
        ];

        let result = parse_contents(contents.to_string());
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result.len(), 2);

        assert_eq!(result[0].keysyms, expected_keybinds[0].keysyms);
        assert_eq!(result[0].command, expected_keybinds[0].command);
        assert_eq!(result[1].keysyms, expected_keybinds[1].keysyms);
        assert_eq!(result[1].command, expected_keybinds[1].command);

        Ok(())
    }

    #[ignore]
    fn test_multiple_keypress() -> std::io::Result<()> {
        let contents = "
super + 5
    alacritty
        ";

        let expected_keybinds = vec![
            // I don't know if the super key is macro or not,
            // please check
            Hotkey::new(vec![evdev::Key::KEY_MACRO, evdev::Key::KEY_5], String::from("alacritty")),
        ];

        let result = parse_contents(contents.to_string());
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result[0].keysyms, expected_keybinds[0].keysyms);
        assert_eq!(result[0].command, expected_keybinds[0].command);

        Ok(())
    }

    #[test]
    fn test_command_with_many_spaces() -> std::io::Result<()> {
        let contents = "
p
    xbacklight -inc 10 -fps 30 -time 200
        ";

        let expected_keybinds = vec![Hotkey::new(
            vec![evdev::Key::KEY_P],
            String::from("xbacklight -inc 10 -fps 30 -time 200"),
        )];

        let result = parse_contents(contents.to_string());
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result[0].keysyms, expected_keybinds[0].keysyms);
        assert_eq!(result[0].command, expected_keybinds[0].command);

        Ok(())
    }

    #[test]
    fn test_invalid_keybinding() -> std::io::Result<()> {
        let contents = "
p
    xbacklight -inc 10 -fps 30 -time 200

pesto
    xterm
                    ";

        let result = parse_contents(contents.to_string());

        assert!(result.is_err());

        let error = result.unwrap_err();

        match error {
            Error::InvalidConfig(parse_err) => match parse_err {
                ParseError::UnknownSymbol(line_nr) => {
                    if line_nr == 5 {
                        Ok(())
                    } else {
                        panic!(
                            "{}",
                            format!("Error line is wrong, expected 4 but actual: {}", line_nr)
                        );
                    }
                }
                _ => {
                    panic!("Error type is not Unknown Symbol");
                }
            },
            _ => {
                panic!("Error type is not InvalidConfig");
            }
        }
    }

    #[test]
    fn test_command_without_whitespace() -> std::io::Result<()> {
        let contents = "0
    firefox

1
brave
            ";
        let result = parse_contents(contents.to_string());

        let config_error = match result {
            Ok(_) => panic!(
                "❌ Commands without whitespaces at the start are invalid.
But the config parser still accepts commands without whitespaces.

The invalid file:
```
{}
``` ",
                contents
            ),
            Err(config_error) => config_error,
        };

        let parse_error = match config_error {
            Error::InvalidConfig(parse_error) => parse_error,
            other_error => panic!(
                "The Error enum type for a command without starting whitespace
is expected to be InvalidConfig, but it is instead {:?}",
                other_error
            ),
        };

        let line_number = match parse_error {
            ParseError::CommandWithoutWhitespace(line_nr) => line_nr,
            other_error => panic!(
                "The ParseError enum type for a command without starting whitespaces
is expected to be CommandWithoutWhitespace, but it is instead {:?}",
                other_error
            ),
        };

        if line_number == 5 {
            Ok(())
        } else {
            panic!(
                "The line number returned for the no-whitespace error is expected to be 5,
but what was returned was {}.

Invalid config file:
```
{}
```",
                line_number, contents
            )
        }
    }

    #[test]
    fn test_eofed_keybinding() -> std::io::Result<()> {
        let contents = "
k
    xbacklight -inc 10 -fps 30 -time 200

c ";

        assert!(parse_contents(contents.to_string()).is_err());

        Ok(())
    }

    #[test]
    fn test_no_command() -> std::io::Result<()> {
        let contents = "
k
    xbacklight -inc 10 -fps 30 -time 200

w

                    ";

        assert!(parse_contents(contents.to_string()).is_err());

        Ok(())
    }

    #[test]
    fn test_all_alphanumeric() -> std::io::Result<()> {
        let symbols: [&str; 36] = [
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7",
            "8", "9",
        ];

        let keysyms: [evdev::Key; 36] = [
            evdev::Key::KEY_A,
            evdev::Key::KEY_B,
            evdev::Key::KEY_C,
            evdev::Key::KEY_D,
            evdev::Key::KEY_E,
            evdev::Key::KEY_F,
            evdev::Key::KEY_G,
            evdev::Key::KEY_H,
            evdev::Key::KEY_I,
            evdev::Key::KEY_J,
            evdev::Key::KEY_K,
            evdev::Key::KEY_L,
            evdev::Key::KEY_M,
            evdev::Key::KEY_N,
            evdev::Key::KEY_O,
            evdev::Key::KEY_P,
            evdev::Key::KEY_Q,
            evdev::Key::KEY_R,
            evdev::Key::KEY_S,
            evdev::Key::KEY_T,
            evdev::Key::KEY_U,
            evdev::Key::KEY_V,
            evdev::Key::KEY_W,
            evdev::Key::KEY_X,
            evdev::Key::KEY_Y,
            evdev::Key::KEY_Z,
            evdev::Key::KEY_0,
            evdev::Key::KEY_1,
            evdev::Key::KEY_2,
            evdev::Key::KEY_3,
            evdev::Key::KEY_4,
            evdev::Key::KEY_5,
            evdev::Key::KEY_6,
            evdev::Key::KEY_7,
            evdev::Key::KEY_8,
            evdev::Key::KEY_9,
        ];

        let mut contents = String::new();

        for symbol in &symbols {
            contents.push_str(&format!("{}\n    st\n", symbol));
        }

        let parse_result = parse_contents(contents);

        assert!(parse_result.is_ok());

        let actual_keybinds = parse_result.unwrap();

        assert_eq!(actual_keybinds.len(), 36);

        for i in 0..actual_keybinds.len() {
            assert_eq!(actual_keybinds[i].keysyms.len(), 1);
            assert_eq!(actual_keybinds[i].keysyms[0], keysyms[i]);
            assert_eq!(actual_keybinds[i].command, "st");
        }

        Ok(())
    }

    #[test]
    fn test_nonsensical_file() -> std::io::Result<()> {
        let contents = "
WE WISH YOU A MERRY RUSTMAS

                    ";

        assert!(parse_contents(contents.to_string()).is_err());

        Ok(())
    }

    #[ignore]
    fn test_valid_keybind_but_commented_command() -> std::io::Result<()> {
        let contents = "
5
    takeshot --now --verbose

p
    #commented out command
                    ";

        assert!(parse_contents(contents.to_string()).is_err());

        Ok(())
    }

    #[ignore]
    fn test_real_config_snippet() -> std::io::Result<()> {
        let contents = "
# reloads sxhkd configuration:
super + Escape
    pkill -USR1 -x sxhkd ; sxhkd &

# Launch Terminal
super + Return
    alacritty -t \"Terminal\" -e \"$HOME/.config/sxhkd/new_tmux_terminal.sh\"

# terminal emulator (no tmux)
super + shift + Return
    alacritty -t \"Terminal\"

# terminal emulator (new tmux session)
alt + Return
    alacritty -t \"Terminal\" -e \"tmux\"

ctrl + 0
    play-song.sh

super + minus
    play-song.sh album
                    ";

        let expected_result: Vec<Hotkey> = vec![
            Hotkey::new(
                vec![evdev::Key::KEY_LEFTMETA, evdev::Key::KEY_ESC],
                String::from("pkill -USR1 -x sxhkd ; sxhkd &"),
            ),
            Hotkey::new(
                vec![evdev::Key::KEY_LEFTMETA, evdev::Key::KEY_ENTER],
                String::from(
                    "alacritty -t \"Terminal\" -e \"$HOME/.config/sxhkd/new_tmux_terminal.sh\"",
                ),
            ),
            Hotkey::new(
                vec![evdev::Key::KEY_LEFTMETA, evdev::Key::KEY_LEFTSHIFT, evdev::Key::KEY_ENTER],
                String::from("alacritty -t \"Terminal\""),
            ),
            Hotkey::new(
                vec![evdev::Key::KEY_LEFTALT, evdev::Key::KEY_ENTER],
                String::from("alacritty -t \"Terminal\" -e \"tmux\""),
            ),
            Hotkey::new(
                vec![evdev::Key::KEY_LEFTCTRL, evdev::Key::KEY_0],
                String::from("play-song.sh"),
            ),
            Hotkey::new(
                vec![evdev::Key::KEY_LEFTMETA, evdev::Key::KEY_MINUS],
                String::from("play-song.sh album"),
            ),
        ];

        let real_result = parse_contents(contents.to_string());

        assert!(real_result.is_ok());

        let real_result = real_result.unwrap();

        assert_eq!(real_result.len(), expected_result.len());

        for i in 0..real_result.len() {
            assert_eq!(real_result[i].keysyms, expected_result[i].keysyms);
            assert_eq!(real_result[i].command, expected_result[i].command);
        }

        Ok(())
    }

    #[ignore]
    fn test_multiline_command() -> std::io::Result<()> {
        let contents = "
k
    mpc ls | dmenu | \\
    sed -i 's/foo/bar/g'
                    ";

        let expected_keybind = Hotkey::new(
            vec![evdev::Key::KEY_K],
            String::from("mpc ls | dmenu | sed -i 's/foo/bar/g'"),
        );

        let real_keybind = parse_contents(contents.to_string());

        assert!(real_keybind.is_ok());

        let real_keybind = real_keybind.unwrap();

        assert_eq!(real_keybind.len(), 1);

        assert_eq!(real_keybind[0].keysyms, expected_keybind.keysyms);
        assert_eq!(real_keybind[0].command, expected_keybind.command);

        Ok(())
    }

    #[test]
    fn test_commented_out_keybind() -> std::io::Result<()> {
        let contents = "
#w
    gimp
                    ";

        assert!(parse_contents(contents.to_string()).is_err());

        Ok(())
    }

    // TODO: Write these tests as needed.

    #[ignore]
    fn test_homerow_special_keys() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_numrow_special_keys() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_all_modifier_keys() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_mod_keys_after_normal_keys() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_plus_at_start_and_end_of_keybind() -> std::io::Result<()> {
        Ok(())
    }

    // Bracket expansion example:
    // `super + ctrl + {h,j,k,l}`
    // `    bspc node -p {westh,south,north,west}`
    #[ignore]
    fn test_bracket_expansion() -> std::io::Result<()> {
        Ok(())
    }

    // `super + {1-9}`
    // `    bspc desktop -f '^{1-9}'`
    #[ignore]
    fn test_bracket_expansion_numbers() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_unclosed_bracket_in_binding() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_bracket_in_binding_but_not_in_command() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_bracket_non_matching_counts() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_multiple_brackets() -> std::io::Result<()> {
        Ok(())
    }

    #[ignore]
    fn test_multiple_brackets_only_one_in_command() -> std::io::Result<()> {
        Ok(())
    }
}
