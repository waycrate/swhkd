use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::{fmt, path};

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            Error::ConfigNotFound => "Config file not found.".fmt(f),

            Error::Io(io_err) => format!("I/O Error while parsing config file: {}", io_err).fmt(f),

            Error::InvalidConfig(parse_err) => match parse_err {
                ParseError::UnknownSymbol(line_nr) => {
                    format!("Unknown symbol at line {}.", line_nr).fmt(f)
                }
                ParseError::InvalidKeysym(line_nr) => {
                    format!("Invalid keysym at line {}.", line_nr).fmt(f)
                }
                ParseError::InvalidModifier(line_nr) => {
                    format!("Invalid modifier at line {}.", line_nr).fmt(f)
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hotkey {
    pub keysym: evdev::Key,
    pub modifiers: Vec<Modifier>,
    pub command: String,
}

#[derive(Debug, PartialEq, Copy, Clone)]
// TODO: make the commented-out modifiers available
pub enum Modifier {
    Super,
    // Hyper,
    // Meta,
    Alt,
    Control,
    Shift,
    // ModeSwitch,
    // Lock,
    // Mod1,
    // Mod2,
    // Mod3,
    // Mod4,
    // Mod5,
}

impl Hotkey {
    pub fn new(keysym: evdev::Key, modifiers: Vec<Modifier>, command: String) -> Self {
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
        ("escape", evdev::Key::KEY_ESC),
        ("delete", evdev::Key::KEY_DELETE),
        ("backspace", evdev::Key::KEY_BACKSPACE),
        ("return", evdev::Key::KEY_ENTER),
        ("enter", evdev::Key::KEY_ENTER),
        ("tab", evdev::Key::KEY_TAB),
        ("space", evdev::Key::KEY_SPACE),
        ("minus", evdev::Key::KEY_MINUS),
        ("-", evdev::Key::KEY_MINUS),
        ("equal", evdev::Key::KEY_EQUAL),
        ("=", evdev::Key::KEY_EQUAL),
        ("grave", evdev::Key::KEY_GRAVE),
        ("`", evdev::Key::KEY_GRAVE),
        ("print", evdev::Key::KEY_SYSRQ),
        ("volumeup", evdev::Key::KEY_VOLUMEUP),
        ("xf86audioraisevolume", evdev::Key::KEY_VOLUMEUP),
        ("volumedown", evdev::Key::KEY_VOLUMEDOWN),
        ("xf86audiolowervolume", evdev::Key::KEY_VOLUMEDOWN),
        ("mute", evdev::Key::KEY_MUTE),
        ("xf86audiomute", evdev::Key::KEY_MUTE),
        ("brightnessup", evdev::Key::KEY_BRIGHTNESSUP),
        ("brightnessdown", evdev::Key::KEY_BRIGHTNESSDOWN),
        (",", evdev::Key::KEY_COMMA),
        ("comma", evdev::Key::KEY_COMMA),
        (".", evdev::Key::KEY_DOT),
        ("dot", evdev::Key::KEY_DOT),
        ("/", evdev::Key::KEY_SLASH),
        ("slash", evdev::Key::KEY_SLASH),
        ("backslash", evdev::Key::KEY_BACKSLASH),
        ("leftbrace", evdev::Key::KEY_LEFTBRACE),
        ("[", evdev::Key::KEY_LEFTBRACE),
        ("rightbrace", evdev::Key::KEY_RIGHTBRACE),
        ("]", evdev::Key::KEY_RIGHTBRACE),
        (";", evdev::Key::KEY_SEMICOLON),
        ("semicolon", evdev::Key::KEY_SEMICOLON),
        ("'", evdev::Key::KEY_APOSTROPHE),
        ("apostrophe", evdev::Key::KEY_APOSTROPHE),
        ("left", evdev::Key::KEY_LEFT),
        ("right", evdev::Key::KEY_RIGHT),
        ("up", evdev::Key::KEY_UP),
        ("down", evdev::Key::KEY_DOWN),
        ("f1", evdev::Key::KEY_F1),
        ("f2", evdev::Key::KEY_F2),
        ("f3", evdev::Key::KEY_F3),
        ("f4", evdev::Key::KEY_F4),
        ("f5", evdev::Key::KEY_F5),
        ("f6", evdev::Key::KEY_F6),
        ("f7", evdev::Key::KEY_F7),
        ("f8", evdev::Key::KEY_F8),
        ("f9", evdev::Key::KEY_F9),
        ("f10", evdev::Key::KEY_F10),
        ("f11", evdev::Key::KEY_F11),
        ("f12", evdev::Key::KEY_F12),
        ("f13", evdev::Key::KEY_F13),
        ("f14", evdev::Key::KEY_F14),
        ("f15", evdev::Key::KEY_F15),
        ("f16", evdev::Key::KEY_F16),
        ("f17", evdev::Key::KEY_F17),
        ("f18", evdev::Key::KEY_F18),
        ("f19", evdev::Key::KEY_F19),
        ("f20", evdev::Key::KEY_F20),
        ("f21", evdev::Key::KEY_F21),
        ("f22", evdev::Key::KEY_F22),
        ("f23", evdev::Key::KEY_F23),
        ("f24", evdev::Key::KEY_F24),
    ]);

    let mod_to_mod_enum: HashMap<&str, Modifier> = HashMap::from([
        ("ctrl", Modifier::Control),
        ("control", Modifier::Control),
        ("super", Modifier::Super),
        ("mod4", Modifier::Super),
        ("alt", Modifier::Alt),
        ("mod1", Modifier::Alt),
        ("shift", Modifier::Shift),
    ]);

    let lines: Vec<&str> = contents.split('\n').collect();

    // Go through each line, ignore comments and empty lines, mark lines starting with whitespace
    // as commands, and mark the other lines as keysyms. Mark means storing a line's type and the
    // line number in a vector.
    let mut lines_with_types: Vec<(&str, u32)> = Vec::new();
    for (line_number, line) in lines.iter().enumerate() {
        if line.trim().starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            lines_with_types.push(("command", line_number as u32));
        } else {
            lines_with_types.push(("keysym", line_number as u32));
        }
    }

    // Edge case: return a blank vector if no lines detected
    if lines_with_types.is_empty() {
        return Ok(vec![]);
    }

    // Go through lines_with_types, and add the next line over and over until the current line no
    // longer ends with backslash. (Only if the lines have the same type)
    let mut actual_lines: Vec<(&str, u32, String)> = Vec::new();
    let mut current_line_type = lines_with_types[0].0;
    let mut current_line_number = lines_with_types[0].1;
    let mut current_line_string = String::new();
    for (line_type, line_number) in lines_with_types {
        if line_type != current_line_type {
            current_line_type = line_type;
            current_line_number = line_number;
            current_line_string = String::new();
        }
        current_line_string.push_str(lines[line_number as usize].trim());
        if !current_line_string.ends_with('\\') {
            actual_lines.push((
                current_line_type,
                current_line_number,
                current_line_string.replace("\\", ""),
            ));
            current_line_type = line_type;
            current_line_number = line_number;
            current_line_string = String::new();
        }
    }

    let mut hotkeys: Vec<Hotkey> = Vec::new();

    for (i, item) in actual_lines.iter().enumerate() {
        let line_type = item.0;
        let line_number = item.1;
        let line = &item.2;

        if line_type != "keysym" {
            continue;
        }

        let next_line = actual_lines.get(i + 1);
        if next_line.is_none() {
            break;
        }
        let next_line = next_line.unwrap();

        if next_line.0 != "command" {
            continue; // this should ignore keysyms that are not followed by a command
        }

        let extracted_keys = extract_curly_brace(line);
        let extracted_commands = extract_curly_brace(&next_line.2);

        'hotkey_parse: for (key, command) in extracted_keys.iter().zip(extracted_commands.iter()) {
            println!("{} {}", key, command);
            let (keysym, modifiers) =
                parse_keybind(key, line_number + 1, &key_to_evdev_key, &mod_to_mod_enum)?;
            let hotkey = Hotkey { keysym, modifiers, command: command.to_string() };

            // Ignore duplicate hotkeys
            for i in hotkeys.iter() {
                if i.keysym == hotkey.keysym && i.modifiers == hotkey.modifiers {
                    continue 'hotkey_parse;
                }
            }

            hotkeys.push(hotkey);
        }
    }
    Ok(hotkeys)
}

// We need to get the reference to key_to_evdev_key
// and mod_to_mod enum instead of recreating them
// after each function call because it's too expensive
fn parse_keybind(
    line: &str,
    line_nr: u32,
    key_to_evdev_key: &HashMap<&str, evdev::Key>,
    mod_to_mod_enum: &HashMap<&str, Modifier>,
) -> Result<(evdev::Key, Vec<Modifier>), Error> {
    let line = line.split('#').next().unwrap();
    let tokens: Vec<String> =
        line.split('+').map(|s| s.trim().to_lowercase()).filter(|s| s != "_").collect();
    let last_token = tokens.last().unwrap().trim();

    // Check if each token is valid
    for token in &tokens {
        if key_to_evdev_key.contains_key(token.as_str()) {
            // Can't have a key that's like a modifier
            if token != last_token {
                return Err(Error::InvalidConfig(ParseError::InvalidModifier(line_nr)));
            }
        } else if mod_to_mod_enum.contains_key(token.as_str()) {
            // Can't have a modifier that's like a modifier
            if token == last_token {
                return Err(Error::InvalidConfig(ParseError::InvalidKeysym(line_nr)));
            }
        } else {
            return Err(Error::InvalidConfig(ParseError::UnknownSymbol(line_nr)));
        }
    }

    // Translate keypress into evdev key
    let keysym = key_to_evdev_key.get(last_token).unwrap();

    let modifiers: Vec<Modifier> = tokens[0..(tokens.len() - 1)]
        .iter()
        .map(|token| *mod_to_mod_enum.get(token.as_str()).unwrap())
        .collect();

    Ok((*keysym, modifiers))
}

fn extract_curly_brace(line: &str) -> Vec<String> {
    if !line.is_ascii() {
        return vec![line.to_string()];
    }
    let mut output: Vec<String> = Vec::new();

    let index_open_brace = line.chars().position(|c| c == '{');
    let index_close_brace = line.chars().position(|c| c == '}');

    if index_open_brace.is_none() || index_close_brace.is_none() {
        return vec![line.to_string()];
    }

    let start_index = index_open_brace.unwrap();
    let end_index = index_close_brace.unwrap();

    // There are no expansions to build if } is earlier than {
    if start_index >= end_index {
        return vec![line.to_string()];
    }

    let str_before_braces = line[..start_index].to_string();
    let str_after_braces = line[end_index + 1..].to_string();

    let comma_separated_items = line[start_index + 1..end_index].split(',');

    for item in comma_separated_items {
        let mut push_one_item = || {
            output.push(format!("{}{}{}", str_before_braces, item.trim(), str_after_braces));
        };

        if !item.contains('-') {
            push_one_item();
            continue;
        }

        // Parse dash ranges like {1-5} and {a-f}

        let mut range = item.split('-').map(|s| s.trim());
        let begin_char: &str;
        let end_char: &str;

        if let Some(b) = range.next() {
            begin_char = b;
        } else {
            push_one_item();
            continue;
        }

        if let Some(e) = range.next() {
            end_char = e;
        } else {
            push_one_item();
            continue;
        }

        // Do not accept range values that are longer than one char
        // Example invalid: {ef-p} {3-56}
        // Beginning of the range cannot be greater than end
        // Example invalid: {9-4} {3-2}
        if begin_char.len() != 1 || end_char.len() != 1 || begin_char > end_char {
            push_one_item();
            continue;
        }

        // In swhkd we will parse the full range using ASCII values.

        let begin_ascii_val = begin_char.parse::<char>().unwrap() as u8;
        let end_ascii_val = end_char.parse::<char>().unwrap() as u8;

        for ascii_number in begin_ascii_val..=end_ascii_val {
            output
                .push(format!("{}{}{}", str_before_braces, ascii_number as char, str_after_braces));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

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
            if self.path.exists() {
                fs::remove_file(self.path()).unwrap();
            }
        }
    }

    // Wrapper for config tests
    fn eval_config_test(contents: &str, expected_hotkeys: Vec<Hotkey>) -> std::io::Result<()> {
        let result = parse_contents(contents.to_string());

        let mut expected_hotkeys_mut = expected_hotkeys;

        if result.is_err() {
            panic!("Expected Ok config, found Err {:?}", result.unwrap_err());
        }

        let actual_hotkeys = result.unwrap();

        assert_eq!(actual_hotkeys.len(), expected_hotkeys_mut.len());

        // Go through each actual hotkey, and pop a corresponding
        // hotkey from the expected hotkeys
        // to make sure that order does not matter
        for hotkey in actual_hotkeys {
            if let Some(index) = expected_hotkeys_mut.iter().position(|key| {
                key.keysym == hotkey.keysym
                    && key.command == hotkey.command
                    && key.modifiers == hotkey.modifiers
            }) {
                expected_hotkeys_mut.remove(index);
            } else {
                panic!(
                    "unexpected hotkey {:#?} found in result\nExpected result:\n{:#?}",
                    hotkey, expected_hotkeys_mut
                );
            }
        }

        if !expected_hotkeys_mut.is_empty() {
            panic!(
                "Some hotkeys were not returned by the actual result:\n{:#?}",
                expected_hotkeys_mut
            );
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
            Error::ConfigNotFound => {}
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
            vec![Hotkey::new(evdev::Key::KEY_R, vec![], String::from("alacritty"))],
        )
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

        eval_config_test(contents, vec![hotkey_1, hotkey_2, hotkey_3])
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

        eval_config_test(contents, expected_keybinds)
    }

    #[test]
    fn test_multiple_keypress() -> std::io::Result<()> {
        let contents = "
super + 5
    alacritty
        ";

        let expected_keybinds =
            vec![Hotkey::new(evdev::Key::KEY_5, vec![Modifier::Super], String::from("alacritty"))];

        eval_config_test(contents, expected_keybinds)
    }

    #[test]
    fn test_keysym_instead_of_modifier() -> std::io::Result<()> {
        let contents = "
shift + k + m
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents, ParseError::InvalidModifier(2))
    }

    #[test]
    fn test_modifier_instead_of_keysym() -> std::io::Result<()> {
        let contents = "
shift + k + alt
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents, ParseError::InvalidModifier(2))
    }

    #[test]
    fn test_unfinished_plus_sign() -> std::io::Result<()> {
        let contents = "


shift + alt +
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents, ParseError::UnknownSymbol(4))
    }

    #[test]
    fn test_plus_sign_at_start() -> std::io::Result<()> {
        let contents = "
+ shift + k
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents, ParseError::UnknownSymbol(2))
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
            Hotkey::new(
                evdev::Key::KEY_K,
                vec![Modifier::Shift],
                "notify-send 'Hello world!'".to_string(),
            ),
            Hotkey::new(
                evdev::Key::KEY_5,
                vec![Modifier::Control],
                "notify-send 'Hello world!'".to_string(),
            ),
            Hotkey::new(
                evdev::Key::KEY_2,
                vec![Modifier::Alt],
                "notify-send 'Hello world!'".to_string(),
            ),
            Hotkey::new(
                evdev::Key::KEY_Z,
                vec![Modifier::Super],
                "notify-send 'Hello world!'".to_string(),
            ),
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

        eval_config_test(contents, expected_keybinds)
    }

    #[test]
    fn test_invalid_keybinding() -> std::io::Result<()> {
        let contents = "
p
    xbacklight -inc 10 -fps 30 -time 200

pesto
    xterm
                    ";

        eval_invalid_config_test(contents, ParseError::UnknownSymbol(5))
    }

    #[test]
    // keysyms not followed by command should be ignored
    fn test_no_command() -> std::io::Result<()> {
        let contents = "
k
    xbacklight -inc 10 -fps 30 -time 200

w

                    ";

        eval_config_test(
            contents,
            vec![Hotkey::new(
                evdev::Key::KEY_K,
                vec![],
                "xbacklight -inc 10 -fps 30 -time 200".to_string(),
            )],
        )
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
            Hotkey::new(evdev::Key::KEY_0, vec![Modifier::Control], String::from("play-song.sh")),
            Hotkey::new(
                evdev::Key::KEY_MINUS,
                vec![Modifier::Super],
                String::from("play-song.sh album"),
            ),
        ];

        eval_config_test(contents, expected_result)
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

        eval_config_test(contents, vec![expected_keybind])
    }

    #[test]
    fn test_commented_out_keybind() -> std::io::Result<()> {
        let contents = "
#w
    gimp
                    ";

        eval_config_test(contents, vec![])
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

        let expected_result: Vec<Hotkey> =
            keysyms.iter().map(|keysym| Hotkey::new(*keysym, vec![], "st".to_string())).collect();

        eval_config_test(contents, expected_result)
    }

    #[test]
    fn test_homerow_special_keys_top() -> std::io::Result<()> {
        let symbols: [&str; 7] =
            ["Escape", "BackSpace", "Return", "Tab", "minus", "equal", "grave"];

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

        let expected_result: Vec<Hotkey> =
            keysyms.iter().map(|keysym| Hotkey::new(*keysym, vec![], "st".to_string())).collect();

        eval_config_test(contents, expected_result)
    }

    #[test]
    fn test_case_insensitive() -> std::io::Result<()> {
        let contents = "
Super + SHIFT + alt + a
    st
ReTurn
    ts
            ";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_A,
                    vec![Modifier::Super, Modifier::Shift, Modifier::Alt],
                    "st".to_string(),
                ),
                Hotkey::new(evdev::Key::KEY_ENTER, vec![], "ts".to_string()),
            ],
        )
    }

    #[test]
    fn test_duplicate_hotkeys() -> std::io::Result<()> {
        let contents = "
super + a
    st
suPer +   A
    ts
b    
    st
B
    ts
";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(evdev::Key::KEY_A, vec![Modifier::Super], "st".to_string()),
                Hotkey::new(evdev::Key::KEY_B, vec![], "st".to_string()),
            ],
        )
    }

    #[test]
    fn test_inline_comment() -> std::io::Result<()> {
        let contents = "
super + a #comment and comment super
    st
super + shift + b
    ts #this comment should be handled by shell
"
        .to_string();
        eval_config_test(
            &contents,
            vec![
                Hotkey::new(evdev::Key::KEY_A, vec![Modifier::Super], "st".to_string()),
                Hotkey::new(
                    evdev::Key::KEY_B,
                    vec![Modifier::Super, Modifier::Shift],
                    "ts #this comment should be handled by shell".to_string(),
                ),
            ],
        )
    }

    #[test]
    fn test_blank_config() -> std::io::Result<()> {
        let contents = "";

        eval_config_test(contents, vec![])
    }

    #[test]
    fn test_blank_config_with_whitespace() -> std::io::Result<()> {
        let contents = "


            ";

        eval_config_test(contents, vec![])
    }

    #[test]
    fn test_extract_curly_brace() -> std::io::Result<()> {
        let keybind_with_curly_brace = "super + {a,b,c}";
        assert_eq!(
            extract_curly_brace(keybind_with_curly_brace),
            vec!["super + a", "super + b", "super + c",]
        );
        let command_with_curly_brace = "bspc node -p {west,south,north,west}";
        assert_eq!(
            extract_curly_brace(command_with_curly_brace),
            vec![
                "bspc node -p west",
                "bspc node -p south",
                "bspc node -p north",
                "bspc node -p west",
            ]
        );
        let wrong_format = "super + }a, b, c{";
        assert_eq!(extract_curly_brace(wrong_format), vec![wrong_format]);
        let single_sym = "super + {a}";
        assert_eq!(extract_curly_brace(single_sym), vec!["super + a"]);
        Ok(())
    }

    #[test]
    fn test_curly_brace() -> std::io::Result<()> {
        let contents = "
super + {a,b,c}
    {firefox, brave, chrome}";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(evdev::Key::KEY_A, vec![Modifier::Super], "firefox".to_string()),
                Hotkey::new(evdev::Key::KEY_B, vec![Modifier::Super], "brave".to_string()),
                Hotkey::new(evdev::Key::KEY_C, vec![Modifier::Super], "chrome".to_string()),
            ],
        )
    }

    #[test]
    fn test_curly_brace_less_commands() -> std::io::Result<()> {
        let contents = "
super + {a,b,c}
    {firefox, brave}";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(evdev::Key::KEY_A, vec![Modifier::Super], "firefox".to_string()),
                Hotkey::new(evdev::Key::KEY_B, vec![Modifier::Super], "brave".to_string()),
            ],
        )
    }

    #[test]
    fn test_curly_brace_less_keysyms() -> std::io::Result<()> {
        let contents = "
super + {a, b}
    {firefox, brave, chrome}";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(evdev::Key::KEY_A, vec![Modifier::Super], "firefox".to_string()),
                Hotkey::new(evdev::Key::KEY_B, vec![Modifier::Super], "brave".to_string()),
            ],
        )
    }

    #[test]
    fn test_range_syntax() -> std::io::Result<()> {
        let contents = "
super + {1-9,0}
    bspc desktop -f '{1-9,0}'";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_1,
                    vec![Modifier::Super],
                    "bspc desktop -f '1'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_2,
                    vec![Modifier::Super],
                    "bspc desktop -f '2'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_3,
                    vec![Modifier::Super],
                    "bspc desktop -f '3'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_4,
                    vec![Modifier::Super],
                    "bspc desktop -f '4'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_5,
                    vec![Modifier::Super],
                    "bspc desktop -f '5'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_6,
                    vec![Modifier::Super],
                    "bspc desktop -f '6'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_7,
                    vec![Modifier::Super],
                    "bspc desktop -f '7'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_8,
                    vec![Modifier::Super],
                    "bspc desktop -f '8'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_9,
                    vec![Modifier::Super],
                    "bspc desktop -f '9'".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_0,
                    vec![Modifier::Super],
                    "bspc desktop -f '0'".to_string(),
                ),
            ],
        )
    }

    #[test]
    fn test_range_syntax_ascii_character() -> std::io::Result<()> {
        let contents = "
super + {a-c}
    {firefox, brave, chrome}";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(evdev::Key::KEY_A, vec![Modifier::Super], "firefox".to_string()),
                Hotkey::new(evdev::Key::KEY_B, vec![Modifier::Super], "brave".to_string()),
                Hotkey::new(evdev::Key::KEY_C, vec![Modifier::Super], "chrome".to_string()),
            ],
        )
    }

    #[test]
    fn test_range_syntax_not_ascii() -> std::io::Result<()> {
        let contents = "
super + {a-是}
    {firefox, brave}
    ";
        eval_invalid_config_test(contents, ParseError::UnknownSymbol(2))
    }

    #[test]
    fn test_range_syntax_invalid_range() -> std::io::Result<()> {
        let contents = "
super + {bc-ad}
    {firefox, brave}
    ";
        eval_invalid_config_test(contents, ParseError::UnknownSymbol(2))
    }

    #[test]
    fn test_ranger_syntax_not_full_range() -> std::io::Result<()> {
        let contents = "
super + {a-}
    {firefox, brave}";
        eval_invalid_config_test(contents, ParseError::UnknownSymbol(2))
    }

    #[test]
    fn test_none() -> std::io::Result<()> {
        let contents = "
super + {_, shift} + b
    {firefox, brave}";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(evdev::Key::KEY_B, vec![Modifier::Super], "firefox".to_string()),
                Hotkey::new(
                    evdev::Key::KEY_B,
                    vec![Modifier::Super, Modifier::Shift],
                    "brave".to_string(),
                ),
            ],
        )
    }

    #[test]
    fn test_multiple_ranges() -> std::io::Result<()> {
        let contents = "
super + {shift,alt} + {c,d}
    {librewolf, firefox} {--sync, --help}
            ";

        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_C,
                    vec![Modifier::Super, Modifier::Shift],
                    "librewolf --sync".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_D,
                    vec![Modifier::Super, Modifier::Shift],
                    "librewolf --help".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_C,
                    vec![Modifier::Super, Modifier::Alt],
                    "firefox --sync".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_D,
                    vec![Modifier::Super, Modifier::Alt],
                    "firefox --help".to_string(),
                ),
            ],
        )
    }

    #[test]
    fn test_multiple_ranges_numbers() -> std::io::Result<()> {
        let contents = "
{control,super} + {1-3}
    {notify-send, echo} {hello,how,are}
            ";

        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_1,
                    vec![Modifier::Control],
                    "notify-send hello".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_2,
                    vec![Modifier::Control],
                    "notify-send how".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_3,
                    vec![Modifier::Control],
                    "notify-send are".to_string(),
                ),
                Hotkey::new(evdev::Key::KEY_1, vec![Modifier::Super], "echo hello".to_string()),
                Hotkey::new(evdev::Key::KEY_2, vec![Modifier::Super], "echo how".to_string()),
                Hotkey::new(evdev::Key::KEY_3, vec![Modifier::Super], "echo are".to_string()),
            ],
        )
    }
}

#[cfg(test)]
mod display_test {
    use super::*;
    use std::io;

    #[test]
    fn test_display_config_not_found_error() {
        let error = Error::ConfigNotFound;

        assert_eq!(format!("{}", error), "Config file not found.");
    }

    #[test]
    fn test_display_io_error() {
        let error = Error::Io(io::Error::from(io::ErrorKind::UnexpectedEof));

        if !format!("{}", error).contains("unexpected end of file") {
            panic!("Error message was '{}", error);
        }
    }

    #[test]
    fn test_display_unknown_symbol_error() {
        let error = Error::InvalidConfig(ParseError::UnknownSymbol(10));

        assert_eq!(format!("{}", error), "Unknown symbol at line 10.");
    }

    #[test]
    fn test_display_invalid_modifier_error() {
        let error = Error::InvalidConfig(ParseError::InvalidModifier(25));

        assert_eq!(format!("{}", error), "Invalid modifier at line 25.");
    }

    #[test]
    fn test_invalid_keysm_error() {
        let error = Error::InvalidConfig(ParseError::InvalidKeysym(7));

        assert_eq!(format!("{}", error), "Invalid keysym at line 7.");
    }
}
