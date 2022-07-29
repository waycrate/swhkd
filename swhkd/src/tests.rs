mod test_config {
    use crate::config::{
        extract_curly_brace, load, load_file_contents, parse_contents, Error, Hotkey, Modifier,
        ParseError, Prefix,
    };
    use std::fs;
    use std::io::Write;
    use std::{fs::File, path::PathBuf};

    // Implement a struct for a path used in tests
    // so that the test file will be automatically removed
    // no matter how the test goes
    struct TestPath {
        path: PathBuf,
    }

    impl TestPath {
        fn new(path: &str) -> Self {
            TestPath { path: PathBuf::from(path) }
        }

        // Create a path method for a more succinct way
        // to deal with borrowing the path value
        fn path(&self) -> PathBuf {
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
        let result = parse_contents(PathBuf::new(), contents.to_string());

        let mut expected_hotkeys_mut = expected_hotkeys;

        if result.is_err() {
            panic!("Expected Ok config, found Err {:?}", result.unwrap_err());
        }

        let result = &result.unwrap()[0];
        let actual_hotkeys = &result.hotkeys;

        assert_eq!(actual_hotkeys.len(), expected_hotkeys_mut.len());

        // Go through each actual hotkey, and pop a corresponding
        // hotkey from the expected hotkeys
        // to make sure that order does not matter
        for hotkey in actual_hotkeys {
            if let Some(index) = expected_hotkeys_mut.iter().position(|key| {
                key.keybinding == hotkey.keybinding && key.command == hotkey.command
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
        let result = parse_contents(PathBuf::new(), contents.to_string());

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
        let path = PathBuf::from(r"This File Doesn't Exist");

        let result = load_file_contents(&path);

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

        let result = load_file_contents(&setup.path());
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_load_multiple_config() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file2");
        let mut f = File::create(setup.path())?;
        f.write_all(
            b"
include /tmp/swhkd-test-file3
super + b
   firefox",
        )?;

        let setup2 = TestPath::new("/tmp/swhkd-test-file3");
        let mut f2 = File::create(setup2.path())?;
        f2.write_all(
            b"
super + c
    hello",
        )?;

        let hotkeys = &load(&setup.path()).unwrap()[0].hotkeys;
        assert_eq!(
            *hotkeys,
            vec!(
                Hotkey::new(evdev::Key::KEY_C, vec![Modifier::Super], String::from("hello")),
                Hotkey::new(evdev::Key::KEY_B, vec![Modifier::Super], String::from("firefox"))
            )
        );
        Ok(())
    }

    #[test]
    fn test_relative_import() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-relative-file1");
        let mut f = File::create(setup.path())?;
        f.write_all(
            b"
include swhkd-relative-file2
super + b
   firefox",
        )?;

        let setup2 = TestPath::new("swhkd-relative-file2");
        let mut f2 = File::create(setup2.path())?;
        f2.write_all(
            b"
super + c
    hello",
        )?;

        let hotkeys = &load(&setup.path()).unwrap()[0].hotkeys;
        assert_eq!(
            *hotkeys,
            vec!(
                Hotkey::new(evdev::Key::KEY_C, vec![Modifier::Super], String::from("hello")),
                Hotkey::new(evdev::Key::KEY_B, vec![Modifier::Super], String::from("firefox"))
            )
        );
        Ok(())
    }

    #[test]
    fn test_more_multiple_configs() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file4");
        let mut f = File::create(setup.path())?;
        f.write_all(
            b"
a
    a",
        )?;

        let setup2 = TestPath::new("/tmp/swhkd-test-file5");
        let mut f2 = File::create(setup2.path())?;
        f2.write_all(
            b"
include /tmp/swhkd-test-file4
b
    b",
        )?;
        let setup3 = TestPath::new("/tmp/swhkd-test-file6");
        let mut f3 = File::create(setup3.path())?;
        f3.write_all(
            b"
include /tmp/swhkd-test-file4
include /tmp/swhkd-test-file5
include /tmp/swhkd-test-file6
include /tmp/swhkd-test-file7
c
    c",
        )?;
        let setup4 = TestPath::new("/tmp/swhkd-test-file7");
        let mut f4 = File::create(setup4.path())?;
        f4.write_all(
            b"
include /tmp/swhkd-test-file6
d
    d",
        )?;

        let hotkeys = &load(&setup4.path()).unwrap()[0].hotkeys;
        assert_eq!(
            *hotkeys,
            vec!(
                Hotkey::new(evdev::Key::KEY_C, vec![], String::from("c")),
                Hotkey::new(evdev::Key::KEY_A, vec![], String::from("a")),
                Hotkey::new(evdev::Key::KEY_B, vec![], String::from("b")),
                Hotkey::new(evdev::Key::KEY_D, vec![], String::from("d")),
            )
        );
        Ok(())
    }
    #[test]
    fn test_include_and_unbind() -> std::io::Result<()> {
        let setup = TestPath::new("/tmp/swhkd-test-file8");
        let mut f = File::create(setup.path())?;
        f.write_all(
            b"
include /tmp/swhkd-test-file9
super + b
   firefox
ignore super + d",
        )?;

        let setup2 = TestPath::new("/tmp/swhkd-test-file9");
        let mut f2 = File::create(setup2.path())?;
        f2.write_all(
            b"
super + c
    hello
super + d
    world",
        )?;

        let hotkeys = &load(&setup.path()).unwrap()[0].hotkeys;
        assert_eq!(
            *hotkeys,
            vec!(
                Hotkey::new(evdev::Key::KEY_C, vec![Modifier::Super], String::from("hello")),
                Hotkey::new(evdev::Key::KEY_B, vec![Modifier::Super], String::from("firefox"))
            )
        );
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

        eval_invalid_config_test(contents, ParseError::InvalidModifier(PathBuf::new(), 2))
    }

    #[test]
    fn test_modifier_instead_of_keysym() -> std::io::Result<()> {
        let contents = "
shift + k + alt
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents, ParseError::InvalidModifier(PathBuf::new(), 2))
    }

    #[test]
    fn test_unfinished_plus_sign() -> std::io::Result<()> {
        let contents = "


shift + alt +
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents, ParseError::UnknownSymbol(PathBuf::new(), 4))
    }

    #[test]
    fn test_plus_sign_at_start() -> std::io::Result<()> {
        let contents = "
+ shift + k
    notify-send 'Hello world!'
            ";

        eval_invalid_config_test(contents, ParseError::UnknownSymbol(PathBuf::new(), 2))
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

        eval_invalid_config_test(contents, ParseError::UnknownSymbol(PathBuf::new(), 5))
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
super + shift + a
    st
shift + suPer +   A
    ts
b
    st
B
    ts
";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_A,
                    vec![Modifier::Super, Modifier::Shift],
                    "ts".to_string(),
                ),
                Hotkey::new(evdev::Key::KEY_B, vec![], "ts".to_string()),
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
        eval_invalid_config_test(contents, ParseError::UnknownSymbol(PathBuf::new(), 2))
    }

    #[test]
    fn test_range_syntax_invalid_range() -> std::io::Result<()> {
        let contents = "
super + {bc-ad}
    {firefox, brave}
    ";
        eval_invalid_config_test(contents, ParseError::UnknownSymbol(PathBuf::new(), 2))
    }

    #[test]
    fn test_ranger_syntax_not_full_range() -> std::io::Result<()> {
        let contents = "
super + {a-}
    {firefox, brave}";
        eval_invalid_config_test(contents, ParseError::UnknownSymbol(PathBuf::new(), 2))
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

    #[test]
    fn test_bspwm_multiple_curly_brace() -> std::io::Result<()> {
        let contents = "
super + {_,shift + }{h,j,k,l}
	bspc node -{f,s} {west,south,north,east}";

        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_H,
                    vec![Modifier::Super],
                    "bspc node -f west".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_J,
                    vec![Modifier::Super],
                    "bspc node -f south".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_K,
                    vec![Modifier::Super],
                    "bspc node -f north".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_L,
                    vec![Modifier::Super],
                    "bspc node -f east".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_H,
                    vec![Modifier::Super, Modifier::Shift],
                    "bspc node -s west".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_J,
                    vec![Modifier::Super, Modifier::Shift],
                    "bspc node -s south".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_K,
                    vec![Modifier::Super, Modifier::Shift],
                    "bspc node -s north".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_L,
                    vec![Modifier::Super, Modifier::Shift],
                    "bspc node -s east".to_string(),
                ),
            ],
        )
    }

    #[test]
    fn test_longer_multiple_curly_brace() -> std::io::Result<()> {
        let contents = "
super + {_, ctrl +} {_, shift +} {1-2}
    riverctl {set, toggle}-{focused, view}-tags {1-2}";
        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_1,
                    vec![Modifier::Super],
                    "riverctl set-focused-tags 1".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_2,
                    vec![Modifier::Super],
                    "riverctl set-focused-tags 2".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_1,
                    vec![Modifier::Super, Modifier::Control],
                    "riverctl toggle-focused-tags 1".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_2,
                    vec![Modifier::Super, Modifier::Control],
                    "riverctl toggle-focused-tags 2".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_1,
                    vec![Modifier::Super, Modifier::Shift],
                    "riverctl set-view-tags 1".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_2,
                    vec![Modifier::Super, Modifier::Shift],
                    "riverctl set-view-tags 2".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_1,
                    vec![Modifier::Super, Modifier::Control, Modifier::Shift],
                    "riverctl toggle-view-tags 1".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_2,
                    vec![Modifier::Super, Modifier::Control, Modifier::Shift],
                    "riverctl toggle-view-tags 2".to_string(),
                ),
            ],
        )
    }

    #[test]
    fn test_period_binding() -> std::io::Result<()> {
        let contents = "
super + {comma, period}
	riverctl focus-output {previous, next}";

        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_COMMA,
                    vec![Modifier::Super],
                    "riverctl focus-output previous".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_DOT,
                    vec![Modifier::Super],
                    "riverctl focus-output next".to_string(),
                ),
            ],
        )
    }

    #[test]
    fn test_period_escape_binding() -> std::io::Result<()> {
        let contents = "
super + {\\,, .}
	riverctl focus-output {previous, next}";

        eval_config_test(
            contents,
            vec![
                Hotkey::new(
                    evdev::Key::KEY_COMMA,
                    vec![Modifier::Super],
                    "riverctl focus-output previous".to_string(),
                ),
                Hotkey::new(
                    evdev::Key::KEY_DOT,
                    vec![Modifier::Super],
                    "riverctl focus-output next".to_string(),
                ),
            ],
        )
    }

    #[test]
    fn test_prefix() -> std::io::Result<()> {
        let contents = "
super + @1
    1
super + ~2
    2
super + ~@3
    3
super + @~4
    4";

        eval_config_test(
            contents,
            vec![
                Hotkey::new(evdev::Key::KEY_1, vec![Modifier::Super], "1".to_string()).on_release(),
                Hotkey::new(evdev::Key::KEY_2, vec![Modifier::Super], "2".to_string()).send(),
                Hotkey::new(evdev::Key::KEY_3, vec![Modifier::Super], "3".to_string())
                    .on_release()
                    .send(),
                Hotkey::new(evdev::Key::KEY_4, vec![Modifier::Super], "4".to_string())
                    .on_release()
                    .send(),
            ],
        )
    }

    #[test]
    fn test_override() -> std::io::Result<()> {
        let contents = "
super + a
    1
super + a
    2";
        eval_config_test(
            contents,
            vec![Hotkey::new(evdev::Key::KEY_A, vec![Modifier::Super], "2".to_string())],
        )
    }

    #[test]
    fn test_any_modifier() -> std::io::Result<()> {
        let contents = "
any + a
    1";
        eval_config_test(
            contents,
            vec![Hotkey::new(evdev::Key::KEY_A, vec![Modifier::Any], "1".to_string())],
        )
    }
}

mod test_config_display {
    use crate::config::{Error, ParseError};
    use std::io;
    use std::path::PathBuf;

    #[test]
    fn test_display_io_error() {
        let error = Error::Io(io::Error::from(io::ErrorKind::UnexpectedEof));

        if !format!("{}", error).contains("unexpected end of file") {
            panic!("Error message was '{}", error);
        }
    }

    #[test]
    fn test_display_unknown_symbol_error() {
        let error = Error::InvalidConfig(ParseError::UnknownSymbol(PathBuf::new(), 10));

        assert_eq!(
            format!("{}", error),
            "Error parsing config file \"\". Unknown symbol at line 10."
        );
    }

    #[test]
    fn test_display_invalid_modifier_error() {
        let error = Error::InvalidConfig(ParseError::InvalidModifier(PathBuf::new(), 25));

        assert_eq!(
            format!("{}", error),
            "Error parsing config file \"\". Invalid modifier at line 25."
        );
    }

    #[test]
    fn test_invalid_keysm_error() {
        let error = Error::InvalidConfig(ParseError::InvalidKeysym(PathBuf::new(), 7));

        assert_eq!(
            format!("{}", error),
            "Error parsing config file \"\". Invalid keysym at line 7."
        );
    }
}
