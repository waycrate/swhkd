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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    // u32 is the line number where an error occured
    UnknownSymbol(u32),
    MissingCommand(u32),
    CommandWithoutWhitespace(u32),
    InvalidModifier(u32),
    InvalidKeysym(u32),
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
    keysym: evdev::Key,
    modifiers: Vec<Modifier>,
    command: String,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Modifier {
    Super,
    Hyper,
    Meta,
    Alt,
    Control,
    Shift,
    ModeSwitch,
    Lock,
    Mod1,
    Mod2,
    Mod3,
    Mod4,
    Mod5
}

impl Hotkey {
    fn new(keysym: evdev::Key, modifiers: Vec<Modifier>, command: String) -> Self {
        Hotkey { keysym, modifiers, command }
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

// We need to get the reference to key_to_evdev_key
// and mod_to_mod enum instead of recreating them
// after each function call because it's too expensive
fn parse_keybind(line: &str, line_nr: u32,
                 key_to_evdev_key: &HashMap<&str, evdev::Key>,
                 mod_to_mod_enum: &HashMap<&str, Modifier>)
    -> Result<(evdev::Key, Vec<Modifier>), Error> {

    let tokens: Vec<&str> = line.split('+').map(|token| token.trim()).collect();
    let last_token = tokens.last().unwrap().trim();

    // Check if each token is valid
    for token in &tokens {
        if key_to_evdev_key.contains_key(token) {
            // Can't have a key that's like a modifier
            if token != &last_token {
                return Err(Error::InvalidConfig(ParseError::InvalidModifier(line_nr)));
            }
        } else if mod_to_mod_enum.contains_key(token) {
            // Can't have a modifier that's like a modifier
            if token == &last_token {
                return Err(Error::InvalidConfig(ParseError::InvalidKeysym(line_nr)));
            }
        } else {
            return Err(Error::InvalidConfig(ParseError::UnknownSymbol(line_nr)));
        }
    }

    // Translate keypress into evdev key
    let keysym = key_to_evdev_key.get(last_token).unwrap();

    let mut modifiers: Vec<Modifier> = Vec::new();

    for i in 0..(tokens.len() - 1) {
        modifiers.push(*mod_to_mod_enum.get(tokens[i]).unwrap());
    }

    let modifiers: Vec<Modifier> = tokens[0..(tokens.len() - 1)]
        .iter()
        .map(|token| *mod_to_mod_enum.get(token).unwrap())
        .collect();

    Ok((*keysym, modifiers))
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

    let mod_to_mod_enum: HashMap<&str, Modifier> = HashMap::from([
        ("ctrl", Modifier::Control),
        ("control", Modifier::Control),
        ("super", Modifier::Super),
        ("alt", Modifier::Alt),
        ("shift", Modifier::Shift),
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

        let (keysym, modifiers) = parse_keybind(
            lines[i],
            real_line_no,
            &key_to_evdev_key,
            &mod_to_mod_enum)?;

        // Error if keybind line is at the very last line
        // ( It's impossible for there to be a command )
        if i >= lines.len() - 1 {
            return Err(Error::InvalidConfig(ParseError::MissingCommand(real_line_no)));
        }

        // Error if empty command
        if lines[i + 1].trim().is_empty() {
            return Err(Error::InvalidConfig(ParseError::MissingCommand(real_line_no + 1)));
        }

        // Error if the command doesn't start with whitespace
        if !lines[i + 1].starts_with(' ') && !lines[i + 1].starts_with('\t') {
            return Err(Error::InvalidConfig(ParseError::CommandWithoutWhitespace(
                real_line_no + 1,
            )));
        }

        // Parse the command, also handling multiline commands
        let mut command = String::new();
        let mut j = i + 1;
        loop {
            if !command.is_empty() {
                command.push(' ');
            }

            command.push_str(lines[j].trim_end_matches('\\')
                                     .trim());

            if lines[j].ends_with('\\') {
                j += 1;
                lines_to_skip += 1;
                continue;
            }
            break;
        }

        // Push a new hotkey to the hotkeys vector
        hotkeys.push(Hotkey::new(keysym, modifiers, String::from(command.trim())));

        // Skip trying to parse the next line (command)
        // because we already dealt with it
        lines_to_skip += 1;
    }

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

    // Wrapper for config tests
    fn eval_config_test(
        contents: &str,
        expected_hotkeys: Vec<Hotkey>
        ) -> std::io::Result<()> {

        let result = parse_contents(contents.to_string());

        if result.is_err() {
            panic!("Expected Ok config, found Err {:?}", result.unwrap_err());
        }

        let actual_hotkeys = result.unwrap();

        assert_eq!(actual_hotkeys.len(), expected_hotkeys.len());

        for i in 0..actual_hotkeys.len() {
            assert_eq!(actual_hotkeys[i].keysym,
                       expected_hotkeys[i].keysym);

            assert_eq!(actual_hotkeys[i].modifiers.len(),
                       expected_hotkeys[i].modifiers.len());
            for j in 0..expected_hotkeys[i].modifiers.len() {
                assert!(actual_hotkeys[i].modifiers
                        .contains(&expected_hotkeys[i].modifiers[j]));
            }

            assert_eq!(actual_hotkeys[i].command,
                       expected_hotkeys[i].command);
        }

        Ok(())
    }

    // Wrapper for the many error tests
    fn eval_invalid_config_test(
        contents: &str,
        parse_error_type: ParseError,
    ) -> std::io::Result<()> {
        let result = parse_contents(contents.to_string());

        assert!(result.is_err());
        let result = result.unwrap_err();

        // Check if the Error type is InvalidConfig
        let result = match result {
            Error::InvalidConfig(parse_err) => parse_err,
            _ => panic!(),
        };

        // Check the ParseError enum type
        if result != parse_error_type {
            panic!("ParseError: Expected `{:?}`, found `{:?}`", parse_error_type, result);
        }

        Ok(())
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

        eval_config_test(
            contents,
            vec![Hotkey::new(evdev::Key::KEY_R, vec![], String::from("alacritty"))]
        )?;
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

        let hotkey_1 = Hotkey::new(evdev::Key::KEY_R, vec![], String::from("alacritty"));
        let hotkey_2 = Hotkey::new(evdev::Key::KEY_W, vec![], String::from("kitty"));
        let hotkey_3 = Hotkey::new(evdev::Key::KEY_T, vec![], String::from("/bin/firefox"));

        eval_config_test(contents,
                         vec![hotkey_1, hotkey_2, hotkey_3])?;
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
            Hotkey::new(evdev::Key::KEY_R, vec![], String::from("alacritty")),
            Hotkey::new(evdev::Key::KEY_W, vec![], String::from("kitty")),
        ];

        eval_config_test(contents, expected_keybinds)?;
        Ok(())
    }

    #[test]
    fn test_multiple_keypress() -> std::io::Result<()> {
        let contents = "
super + 5
    alacritty
        ";

        let expected_keybinds = vec![
            Hotkey::new(evdev::Key::KEY_5,
                        vec![Modifier::Super],
                        String::from("alacritty")),
        ];

        eval_config_test(contents, expected_keybinds)?;
        Ok(())
    }

    #[test]
    fn test_keysym_instead_of_modifier() -> std::io::Result<()> {
        let contents = "
shift + k + m
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents,
                                 ParseError::InvalidModifier(2))
    }

    #[test]
    fn test_modifier_instead_of_keysym() -> std::io::Result<()> {
        let contents = "
shift + k + alt
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents,
                                 ParseError::InvalidModifier(2))
    }

    #[test]
    fn test_unfinished_plus_sign() -> std::io::Result<()> {
        let contents = "


shift + alt +
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents,
                                 ParseError::UnknownSymbol(4))
    }

    #[test]
    fn test_plus_sign_at_start() -> std::io::Result<()> {
        let contents = "
+ shift + k
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents,
                                 ParseError::UnknownSymbol(2))
    }

    #[test]
    fn test_common_modifiers() -> std::io::Result<()> {
        let contents = "
shift + k
    notify-send 'Hello world!'

control + 5
    notify-send 'Hello world!'

alt + 2
    notify-send 'Hello world!'

super + z
    notify-send 'Hello world!'
            ";

        let expected_hotkeys = vec![
            Hotkey::new(evdev::Key::KEY_K,
                        vec![Modifier::Shift],
                        "notify-send 'Hello world!'".to_string()),
            Hotkey::new(evdev::Key::KEY_5,
                        vec![Modifier::Control],
                        "notify-send 'Hello world!'".to_string()),
            Hotkey::new(evdev::Key::KEY_2,
                        vec![Modifier::Alt],
                        "notify-send 'Hello world!'".to_string()),
            Hotkey::new(evdev::Key::KEY_Z,
                        vec![Modifier::Super],
                        "notify-send 'Hello world!'".to_string()),
        ];

        eval_config_test(contents, expected_hotkeys)
    }

    #[test]
    fn test_command_with_many_spaces() -> std::io::Result<()> {
        let contents = "
p
    xbacklight -inc 10 -fps 30 -time 200
        ";

        let expected_keybinds = vec![Hotkey::new(
            evdev::Key::KEY_P,
            vec![],
            String::from("xbacklight -inc 10 -fps 30 -time 200"),
        )];

        eval_config_test(contents, expected_keybinds)?;
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

        eval_invalid_config_test(contents, ParseError::UnknownSymbol(5))?;
        Ok(())
    }

    #[test]
    fn test_command_without_whitespace() -> std::io::Result<()> {
        let contents = "0
    firefox

1
brave
            ";

        eval_invalid_config_test(contents, ParseError::CommandWithoutWhitespace(5))?;
        Ok(())
    }

    #[test]
    fn test_eofed_keybinding() -> std::io::Result<()> {
        let contents = "
k
    xbacklight -inc 10 -fps 30 -time 200

c ";

        eval_invalid_config_test(contents, ParseError::MissingCommand(5))?;
        Ok(())
    }

    #[test]
    fn test_no_command() -> std::io::Result<()> {
        let contents = "
k
    xbacklight -inc 10 -fps 30 -time 200

w

                    ";

        eval_invalid_config_test(contents, ParseError::MissingCommand(6))?;
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

    #[test]
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
                evdev::Key::KEY_ESC,
                vec![Modifier::Super],
                String::from("pkill -USR1 -x sxhkd ; sxhkd &"),
            ),
            Hotkey::new(
                evdev::Key::KEY_ENTER,
                vec![Modifier::Super],
                String::from(
                    "alacritty -t \"Terminal\" -e \"$HOME/.config/sxhkd/new_tmux_terminal.sh\"",
                ),
            ),
            Hotkey::new(
                evdev::Key::KEY_ENTER,
                vec![Modifier::Super, Modifier::Shift],
                String::from("alacritty -t \"Terminal\""),
            ),
            Hotkey::new(
                evdev::Key::KEY_ENTER,
                vec![Modifier::Alt],
                String::from("alacritty -t \"Terminal\" -e \"tmux\""),
            ),
            Hotkey::new(
                evdev::Key::KEY_0,
                vec![Modifier::Control],
                String::from("play-song.sh"),
            ),
            Hotkey::new(
                evdev::Key::KEY_MINUS,
                vec![Modifier::Super],
                String::from("play-song.sh album"),
            ),
        ];

        eval_config_test(contents, expected_result)?;
        Ok(())
    }

    #[test]
    fn test_multiline_command() -> std::io::Result<()> {
        let contents = "
k
    mpc ls | dmenu | \\
    sed -i 's/foo/bar/g'
                    ";

        let expected_keybind = Hotkey::new(
            evdev::Key::KEY_K,
            vec![],
            String::from("mpc ls | dmenu | sed -i 's/foo/bar/g'"),
        );

        eval_config_test(contents, vec![expected_keybind])?;
        Ok(())
    }

    #[test]
    fn test_commented_out_keybind() -> std::io::Result<()> {
        let contents = "
#w
    gimp
                    ";

        eval_invalid_config_test(contents, ParseError::UnknownSymbol(3))?;
        Ok(())
    }

    // TODO: Write these tests as needed.

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
        let contents = &contents;

        let expected_result: Vec<Hotkey> = keysyms
                        .iter()
                        .map(|keysym|
                             Hotkey::new(*keysym,
                                         vec![],
                                         "st".to_string()))
                        .collect();

        eval_config_test(contents, expected_result)
    }

    #[test]
    #[ignore]
    fn test_homerow_special_keys() -> std::io::Result<()> {
        // Quite difficult to find the evdev equivalnets for these.
        let symbols: [&str; 31] = [
            "bracketleft",
            "braceleft",
            "bracketright",
            "braceright",
            "semicolon",
            "colon",
            "apostrophe",
            "quotedbl",
            "comma",
            "less",
            "period",
            "greater",
            "slash",
            "question",
            "backslash",
            "bar",
            "grave",
            "asciitilde",
            "at",
            "numbersign",
            "dollar",
            "percent",
            "asciicircum",
            "ampersand",
            "asterisk",
            "parenleft",
            "parenright",
            "minus",
            "underscore",
            "equal",
            "plus",
        ];

        // TODO: Find the appropiate key for each keysym
        let keysyms: [evdev::Key; 18] = [
            evdev::Key::KEY_A,
            evdev::Key::KEY_B,
            evdev::Key::KEY_C,
            evdev::Key::KEY_B,
            evdev::Key::KEY_C,
            evdev::Key::KEY_D,
            evdev::Key::KEY_E,
            evdev::Key::KEY_F,
            evdev::Key::KEY_G,
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
        let contents = &contents;

        let expected_result: Vec<Hotkey> = keysyms
                        .iter()
                        .map(|keysym|
                             Hotkey::new(*keysym,
                                         vec![],
                                         "st".to_string()))
                        .collect();

        eval_config_test(contents, expected_result)
    }

    #[test]
    fn test_homerow_special_keys_top() -> std::io::Result<()> {
        let symbols: [&str; 7] = [
            "Escape",
            "BackSpace",
            "Return",
            "Tab",
            "minus",
            "equal",
            "grave",
        ];

        let keysyms: [evdev::Key; 7] = [
            evdev::Key::KEY_ESC,
            evdev::Key::KEY_BACKSPACE,
            evdev::Key::KEY_ENTER,
            evdev::Key::KEY_TAB,
            evdev::Key::KEY_MINUS,
            evdev::Key::KEY_EQUAL,
            evdev::Key::KEY_GRAVE,
        ];

        let mut contents = String::new();
        for symbol in &symbols {
            contents.push_str(&format!("{}\n    st\n", symbol));
        }
        let contents = &contents;

        let expected_result: Vec<Hotkey> = keysyms
                        .iter()
                        .map(|keysym|
                             Hotkey::new(*keysym,
                                         vec![],
                                         "st".to_string()))
                        .collect();

        eval_config_test(contents, expected_result)
    }

    #[test]
    #[ignore]
    fn test_numrow_special_keys() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_all_modifier_keys() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_mod_keys_after_normal_keys() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_plus_at_start_and_end_of_keybind() -> std::io::Result<()> {
        Ok(())
    }

    // Bracket expansion example:
    // `super + ctrl + {h,j,k,l}`
    // `    bspc node -p {westh,south,north,west}`
    #[test]
    #[ignore]
    fn test_bracket_expansion() -> std::io::Result<()> {
        Ok(())
    }

    // `super + {1-9}`
    // `    bspc desktop -f '^{1-9}'`
    #[test]
    #[ignore]
    fn test_bracket_expansion_numbers() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_unclosed_bracket_in_binding() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_bracket_in_binding_but_not_in_command() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_bracket_non_matching_counts() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_multiple_brackets() -> std::io::Result<()> {
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_multiple_brackets_only_one_in_command() -> std::io::Result<()> {
        Ok(())
    }
}
