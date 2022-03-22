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
        thread::{
            sleep as thread_sleep,
            spawn as thread_spawn,
        },
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
#[derive(clap::Parser)]
struct TestArgs {
    #[clap(short, long)]
    delay: Option<u64>,
}
#[derive(clap::Parser)]
struct ConnectArgs {
    #[clap(short, long)]
    ip: String,
    #[clap(short, long)]
    timeout: Option<u64>,
}
#[derive(clap::Subcommand)]
enum Mode {
    Test(TestArgs),
    Connect(ConnectArgs),
}
#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    mode: Mode,
}
fn main() -> CrosstermResult<()> {
    const TICK: u64 = 200;
    let args = Args::parse();
    execute!(get_stdout(), EnterAlternateScreen)?;
    execute!(get_stdout(), Hide)?;
    enable_raw_mode()?;
    let keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    { // keypusher
        let mut keyboard_mode = false;
        let keypusher_keys = keys.clone();
        let mut initial = true;
        thread_spawn(move || {
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
                                match event.code {
                                    KeyCode::Char('g') => {
                                        key = String::from("Home");
                                    },
                                    KeyCode::Char('h') => {
                                        key = String::from("Left");
                                    },
                                    KeyCode::Char('H') => {
                                        key = String::from("Rev");
                                    },
                                    KeyCode::Char('i') => {
                                        key = String::from("KeyboardMode");
                                    },
                                    KeyCode::Char('I') => {
                                        key = String::from("InstantReplay");
                                    },
                                    KeyCode::Char('j') => {
                                        key = String::from("Down");
                                    },
                                    KeyCode::Char('J') => {
                                        key = String::from("ChannelDown");
                                    },
                                    KeyCode::Char('k') => {
                                        key = String::from("Up");
                                    },
                                    KeyCode::Char('K') => {
                                        key = String::from("ChannelUp");
                                    },
                                    KeyCode::Char('L') => {
                                        key = String::from("Fwd");
                                    },
                                    KeyCode::Char('l') => {
                                        key = String::from("Right");
                                    },
                                    KeyCode::Char('m') => {
                                        key = String::from("Mute");
                                    },
                                    KeyCode::Char('p') => {
                                        key = String::from("PowerOn");
                                    },
                                    KeyCode::Char('P') => {
                                        key = String::from("PowerOff");
                                    },
                                    KeyCode::Char('s') => {
                                        key = String::from("Search");
                                    },
                                    KeyCode::Char('q') => {
                                        key = String::from("Quit");
                                    },
                                    KeyCode::Char('Q') => {
                                        key = String::from("HardQuit");
                                    },
                                    KeyCode::Char('v') => {
                                        key = String::from("VolumeDown");
                                    },
                                    KeyCode::Char('V') => {
                                        key = String::from("VolumeUp");
                                    },
                                    KeyCode::Char(' ') => {
                                        key = String::from("Play");
                                    },
                                    KeyCode::Char('|') => {
                                        key = String::from("Pause");
                                    },
                                    KeyCode::Char('?') => {
                                        key = String::from("Help");
                                    },
                                    KeyCode::Char('*') => {
                                        key = String::from("Info");
                                    },
                                    KeyCode::Enter => {
                                        key = String::from("Select");
                                    },
                                    KeyCode::Down => {
                                        key = String::from("Down");
                                    },
                                    KeyCode::Up => {
                                        key = String::from("Up");
                                    },
                                    KeyCode::Left => {
                                        key = String::from("Left");
                                    },
                                    KeyCode::Right => {
                                        key = String::from("Right");
                                    },
                                    _ => {
                                        print(
                                            format!(
                                                "KeyPusher: invalid key - {:?}",
                                                event.code
                                            )
                                        ).unwrap();
                                        continue;
                                    },
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
                                        key = String::from("ExitKeyboardMode");
                                    },
                                    _ => {
                                        print(
                                            format!(
                                                "KeyPusher: invalid key - {:?}",
                                                event.code
                                            )
                                        ).unwrap();
                                        continue;
                                    },
                                }
                            }
                            if key.eq("KeyboardMode") {
                                print("Entering keyboard mode")
                                    .unwrap();
                                keyboard_mode = true;
                                continue;
                            } else if key.eq("ExitKeyboardMode") {
                                print("Exiting keyboard mode")
                                    .unwrap();
                                keyboard_mode = false;
                                continue;
                            }
                            print(String::from("KeyPusher: locking"))
                                .unwrap();
                            let mut lock = keypusher_keys.lock().unwrap();
                            if key.eq("Quit") {
                                print("KeyPusher: quitting").unwrap();
                                (*lock).push(key);
                                break;
                            } else if key.eq("HardQuit") {
                                print("KeyPusher: quitting - sending hard quit")
                                    .unwrap();
                                (*lock).push(key);
                                break;
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
            let keypath = format!("/keypress/{}", key);
            print(format!("KeyReceiver: sending {}", keypath)).unwrap();
            match args.mode {
                Mode::Connect(ref args) => {
                    let url = format!("http://{}:8060{}", args.ip, keypath);
                    let timeout = match args.timeout {
                        Some(t) => t.clone(),
                        None => 2000,
                    };
                    match Client::new().get(&url).timeout(
                        Duration::from_millis(timeout)
                    ).send() {
                        Ok(_) => {
                            print(format!("Complete: {}", url)).unwrap();
                        },
                        Err(e) => {
                            print(format!("Error:    {} | {}", url, e)).unwrap();
                        },
                    }
                },
                Mode::Test(ref args) => {
                    match args.delay {
                        Some(delay) => {
                            thread_sleep(Duration::from_millis(delay.to_owned()));
                        },
                        None => {},
                    }
                    print(format!("KeyReceiver: complete {}", keypath)).unwrap();
                },
            }
        }
    }
    print(String::from("Main: exiting")).unwrap();
    disable_raw_mode()?;
    execute!(get_stdout(), Show)?;
    execute!(get_stdout(), LeaveAlternateScreen)?;
    Ok(())
}
