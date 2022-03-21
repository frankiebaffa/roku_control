use {
    clap::Parser,
    crossterm::{
        cursor::{
            MoveToColumn,
            MoveToNextLine,
            MoveTo,
            Hide,
            Show,
        },
        event::{
            Event,
            KeyCode,
            poll,
            read,
        },
        execute,
        Result as CrosstermResult,
        terminal::{
            Clear,
            ClearType,
            disable_raw_mode,
            enable_raw_mode,
            EnterAlternateScreen,
            LeaveAlternateScreen,
        },
    },
    reqwest::blocking::Client,
    std::{
        io::{
            stdout as get_stdout,
            Write,
        },
        sync::{
            Arc,
            Mutex,
        },
        thread::spawn as thread_spawn,
        time::Duration,
    },
};
fn print(msg: impl AsRef<str>) -> CrosstermResult<()> {
    let message = msg.as_ref();
    execute!(get_stdout(), MoveToColumn(0))?;
    get_stdout().lock().write(message.as_bytes()).unwrap();
    execute!(get_stdout(), MoveToNextLine(1))?;
    Ok(())
}
fn send(ip_opt: &Option<impl AsRef<str>>, key_ref: impl AsRef<str>) {
    let key = key_ref.as_ref();
    let keypath = format!("/keypress/{}", key);
    match ip_opt {
        Some(ip_ref) => {
            let ip = ip_ref.as_ref();
            let url = format!("http://{}:8060{}", ip, keypath);
            print(format!("Sending {}", url)).unwrap();
            match Client::new().get(&url).send() {
                Ok(_) => {
                    print(format!("Complete: {}", url)).unwrap();
                },
                Err(e) => {
                    print(format!("Error:    {} | {}", url, e)).unwrap();
                },
            }
        },
        None => {
            print(keypath).unwrap();
        },
    }
}
struct Command {
    key: String,
    character: KeyCode,
}
impl Command {
    fn new(key: String, character: KeyCode) -> Self {
        Self { key, character, }
    }
}
struct Commands {
    commands: Vec<Command>,
}
impl Commands {
    fn new() -> Self {
        Self { commands: Vec::new(), }
    }
    fn add(&mut self, key_ref: impl AsRef<str>, character: KeyCode) {
        let key = key_ref.as_ref().to_string();
        self.commands.push(Command::new(key, character));
    }
    fn init(&mut self) {
        self.add("Home", KeyCode::Char('g'));
        self.add("Left", KeyCode::Char('h'));
        self.add("Rev", KeyCode::Char('H'));
        self.add("KeyboardMode", KeyCode::Char('i'));
        self.add("InstantReplay", KeyCode::Char('I'));
        self.add("Down", KeyCode::Char('j'));
        self.add("ChannelDown", KeyCode::Char('J'));
        self.add("Up", KeyCode::Char('k'));
        self.add("ChannelUp", KeyCode::Char('K'));
        self.add("Fwd", KeyCode::Char('L'));
        self.add("Right", KeyCode::Char('l'));
        self.add("Mute", KeyCode::Char('m'));
        self.add("PowerOn", KeyCode::Char('p'));
        self.add("PowerOff", KeyCode::Char('P'));
        self.add("Search", KeyCode::Char('s'));
        self.add("Quit", KeyCode::Char('q'));
        self.add("HardQuit", KeyCode::Char('Q'));
        self.add("VolumeDown", KeyCode::Char('v'));
        self.add("VolumeUp", KeyCode::Char('V'));
        self.add("Select", KeyCode::Enter);
        self.add("Play", KeyCode::Char(' '));
        self.add("Pause", KeyCode::Char('|'));
        self.add("Help", KeyCode::Char('?'));
        self.add("Info", KeyCode::Char('*'));
        self.add("Search", KeyCode::Char('s'));
    }
    fn show_help(&self) {
        self.commands.iter().for_each(|cmd| {
            print(format!("{} | \"{:?}\"", cmd.key, cmd.character)).unwrap();
        });
    }
}
#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    ip: Option<String>,
}
fn main() -> CrosstermResult<()> {
    const TICK: u64 = 200;
    let args = Args::parse();
    let mut cmds = Commands::new();
    cmds.init();
    execute!(get_stdout(), EnterAlternateScreen)?;
    execute!(get_stdout(), Hide)?;
    enable_raw_mode()?;
    let keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    { // keypusher
        let mut keyboard_mode = false;
        let keypusher_keys = keys.clone();
        let mut initial = true;
        thread_spawn(move || {
            cmds.show_help();
            loop {
                if poll(Duration::from_millis(TICK)).unwrap() {
                    match read().unwrap() {
                        Event::Key(event) => {
                            if initial {
                                initial = false;
                            }
                            execute!(
                                get_stdout(), MoveTo(0, 0)
                            ).unwrap();
                            execute!(
                                get_stdout(), Clear(ClearType::All)
                            ).unwrap();
                            let key;
                            if !keyboard_mode {
                                let matching = cmds.commands.iter().filter(|cmd| {
                                    cmd.character.eq(&event.code)
                                }).collect::<Vec<&Command>>();
                                if matching.len() == 0 {
                                    print(format!("Invalid key: {:?}", event.code))
                                        .unwrap();
                                    continue;
                                } else {
                                    key = matching.get(0).unwrap().key.clone();
                                }
                            } else {
                                match event.code {
                                    KeyCode::Backspace => {
                                        key = String::from("Backspace");
                                    },
                                    KeyCode::Char(' ') => {
                                        key = format!("Lit_+");
                                    },
                                    KeyCode::Char(c) => {
                                        key = format!("Lit_{}", c);
                                    },
                                    KeyCode::Enter => {
                                        key = String::from("Enter");
                                    },
                                    KeyCode::Esc => {
                                        print("Exiting keyboard mode")
                                            .unwrap();
                                        keyboard_mode = false;
                                        continue;
                                    },
                                    _ => {
                                        continue;
                                    },
                                }
                            }
                            if key.eq("KeyboardMode") {
                                print("Entering keyboard mode")
                                    .unwrap();
                                keyboard_mode = true;
                                continue;
                            }
                            print(String::from("KeyPusher: locking"))
                                .unwrap();
                            let mut lock = keypusher_keys.lock().unwrap();
                            if key.eq("Quit") {
                                (*lock).push(key);
                                break;
                            } else if key.eq("Help") {
                                cmds.show_help();
                                continue;
                            } else {
                                print(format!("KeyPusher: sending {}", key))
                                    .unwrap();
                                (*lock).push(key);
                                continue;
                            }
                        },
                        _ => {},
                    }
                }
            }
        });
    }
    { // keyreceiver
        let keyreceiver_keys = keys.clone();
        loop {
            let key;
            {
                let mut lock = keyreceiver_keys.lock().unwrap();
                if (*lock).len() == 0 {
                    continue;
                }
                let hard_quit = lock.iter().filter(|cmd| {
                    cmd.eq(&"HardQuit")
                });
                if hard_quit.count() > 0 {
                    break;
                }
                key = (*lock).remove(0);
            }
            if key.eq("Quit") {
                break;
            }
            print(format!("KeyReceiver: sending {}", key)).unwrap();
            send(&args.ip, key);
        }
    }
    print(String::from("Main: exiting")).unwrap();
    disable_raw_mode()?;
    execute!(get_stdout(), Show)?;
    execute!(get_stdout(), LeaveAlternateScreen)?;
    Ok(())
}
