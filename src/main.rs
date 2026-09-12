use std::fs::{File, exists, self, OpenOptions};
use std::io::{Read, Write, self, stdin};
use std::path::PathBuf;
use std::env;
use std::process;
use std::borrow::Borrow;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use users::{get_user_by_uid, get_current_uid};

mod parse_st;
use parse_st::StData;

fn save(data: &StData, fname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(fname)?;
    file.write_all(data.to_string().as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;

    let args: Vec<String> = env::args().collect();
    let username = String::from_utf8(get_user_by_uid(get_current_uid()).expect("Couldn't get UID").name().as_encoded_bytes().to_vec())?;
    let configfile = "/home/".to_owned() + &username + "/.config/writeST/recent_files";
    if !exists("/home/".to_owned() + &username + "/.config/writeST").expect("Failed to check if the config directory exists") {
        println!("Creating config directory");
        fs::create_dir("/home/".to_owned() + &username + "/.config/writeST")?;
    }
    let mut config = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&configfile)?;
    let mut configinfo = String::new();
    config.read_to_string(&mut configinfo)?;
    let mut x: u16 = 0;
    let mut y: u16 = 0;
    let mut openfile: String;
    print!("\x1b[?1049h\x1b[H");
    let mut fnames: Vec<&str> = configinfo.split('\n').collect();
    fnames.remove(0);
    fnames.reverse();
    if fnames.len() < 1 {
        println!("No file history has been saved. Please specify a file as an argument.");
    }
    if args.len() < 2 {
        for f in &fnames {
            let path = PathBuf::from(f);
            println!("[ ] {} \x1b[33m{}\x1b[39m", path.file_name().unwrap().to_str().unwrap().to_string(), f);
        }
        println!("\nUp and down arrows to move selection\nPress enter to open a file\nPress q to quit");
        print!("\x1b[1;2H");
        io::stdout().flush().expect("Failed to flush stdout");
        enable_raw_mode()?;
        'gallery: loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char('q') => {
                                print!("\x1b[s");
                                print!("\x1b[{};0H", fnames.len() + 1);
                                print!("Press q again to quit, or press escape to cancel.");
                                io::stdout().flush().expect("Failed to flush stdout");
                                loop {
                                    if event::poll(std::time::Duration::from_millis(100))? {
                                        if let Event::Key(key_event) = event::read()? {
                                            if key_event.kind == KeyEventKind::Press {
                                                match key_event.code {
                                                    KeyCode::Char('q') => {
                                                        print!("\x1b[?1049l");
                                                        process::exit(0);
                                                    },
                                                    KeyCode::Esc => {
                                                        break;
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                print!("\x1b[{};0H", fnames.len() + 1);
                                for _ in 0.."Press q again to quit, or press escape to cancel.".len() {
                                    print!(" ");
                                }
                                print!("\x1b[u");
                                io::stdout().flush().expect("Failed to flush stdout");
                            },
                            KeyCode::Up => {
                                if y > 0 {
                                    print!("\x1b[A");
                                    io::stdout().flush().expect("Failed to flush stdout");
                                    y -= 1;
                                }
                            },
                            KeyCode::Down => {
                                if y < (fnames.len() - 1) as u16 {
                                    print!("\x1b[B");
                                    io::stdout().flush().expect("Failed to flush stdout");
                                    y += 1;
                                }
                            },
                            KeyCode::Enter => {
                                print!("\x1b[s");
                                print!("\x1b[{};0H", fnames.len() + 1);
                                print!("You are opening {}. Press enter again to confirm, or escape to cancel.", fnames[y as usize]);
                                io::stdout().flush().expect("Failed to flush stdout");
                                loop {
                                    if event::poll(std::time::Duration::from_millis(100))? {
                                        if let Event::Key(key_event) = event::read()? {
                                            if key_event.kind == KeyEventKind::Press {
                                                match key_event.code {
                                                    KeyCode::Enter => {
                                                        openfile = fnames[y as usize].to_string();
                                                        break 'gallery;
                                                    },
                                                    KeyCode::Esc => {
                                                        break;
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                print!("\x1b[{};0H", fnames.len() + 1);
                                io::stdout().flush().expect("Failed to flush stdout");
                                for _ in 0..format!("You are opening {}. Press enter again to confirm, or escape to cancel.", fnames[y as usize]).len() {
                                    print!(" ");
                                }
                                print!("\x1b[u");
                                io::stdout().flush().expect("Failed to flush stdout");
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        disable_raw_mode()?;
    } else {
        openfile = fs::canonicalize((&args[1]).to_owned())?.display().to_string();
    }
    print!("\x1b[2J");
    print!("\x1b[H");
    if !exists(&openfile).expect("Failed to check if the file exists") {
        println!("Error: The file you specified does not exist");
        process::exit(1);
    }
    let openfilepath = PathBuf::from(&openfile);
    let mut miles = false;
    for f in &fnames {
        if *f == openfile {
            miles = true;
            break;
        }
    }
    if miles {
        config.set_len(0)?;
        fnames.reverse();
        for f in &fnames {
            if *f != openfile {
                config.write(format!("\n{}", fs::canonicalize(*f)?.display().to_string()).as_bytes())?;
            }
        }
        fnames.reverse();
        config.write(format!("\n{}", openfile).as_bytes())?;
    } else {
        config.write(format!("\n{}", fs::canonicalize(&openfilepath)?.display().to_string()).as_bytes())?;
    }
    let mut file = File::open(&openfile)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let mut data = StData::from_string(contents).expect("unable to parse st data");
    let w = data.width() as u16;
    let h = data.height() as u16;
    let mut index = 0;
        for col in data.data() {
        print!("\x1b[48;2;{};{};{}m  \x1b[0m", col[0], col[1], col[2]);
        index += 1;
        if index >= data.width() {
            index = 0;
            println!();
        }
    }
    let messagestr = "\nUse arrow keys to move\nPress q to quit\nPress s to save\nPress enter to select colour\nPress space to toggle drawing mode\nPress r to reset display";
    println!("{messagestr}");
    print!("\x1b[H");
    print!("\x1b[1 q");
    io::stdout().flush().expect("Failed to flush stdout");
    let mut col: String;
    let mut red: u8 = 0;
    let mut green: u8 = 0;
    let mut blue: u8 = 0;
    let mut paint = false;
    enable_raw_mode()?;
    'editor: loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            print!("\x1b[s");
                            print!("\x1b[{};0H", h + 1);
                            print!("Warning: All unsaved changes will be lost. Press q again to quit, or press escape to cancel.");
                            io::stdout().flush().expect("Failed to flush stdout");
                            loop {
                                if event::poll(std::time::Duration::from_millis(100))? {
                                    if let Event::Key(key_event) = event::read()? {
                                        if key_event.kind == KeyEventKind::Press {
                                            match key_event.code {
                                                KeyCode::Char('q') => {
                                                    break 'editor;
                                                },
                                                KeyCode::Esc => {
                                                    break;
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            print!("\x1b[{};0H", h + 1);
                            for _ in 0.."Warning: All unsaved changes will be lost. Press q again to quit, or press escape to cancel.".len() {
                                print!(" ");
                            }
                            print!("\x1b[u");
                        },
                        KeyCode::Up => {
                            if y > 0 {
                                print!("\x1b[A");
                                y -= 1;
                            }
                        },
                        KeyCode::Down => {
                            if y < h - 1 {
                                print!("\x1b[B");
                                y += 1;
                            }
                        },
                        KeyCode::Right => {
                            if x < w - 1 {
                                print!("\x1b[2C");
                                x += 1;
                            }
                        },
                        KeyCode::Left => {
                            if x > 0 {
                                print!("\x1b[2D");
                                x -= 1;
                            }
                        },
                        KeyCode::Char(' ') => {
                            paint = !paint;
                        },
                        KeyCode::Char('r') => {
                            print!("\x1b[2J");
                            print!("\x1b[H");
                            x = 0;
                            y = 0;
                            let mut xx = 0;
                            disable_raw_mode()?;
                            for col in data.data() {
                                print!("\x1b[48;2;{};{};{}m  \x1b[0m", col[0], col[1], col[2]);
                                xx += 1;
                                if xx > w - 1 {
                                    println!();
                                    xx = 0;
                                }
                            }
                            println!("{messagestr}");
                            enable_raw_mode()?;
                            print!("\x1b[H");
                        },
                        KeyCode::Char('s') => {
                            print!("\x1b[s");
                            print!("\x1b[{};0H", h + 1);
                            print!("Your changes will be written to {openfile}. Press s again to save, press n to save with a certain filename, or press escape to cancel.");
                            std::io::stdout().flush()?;
                            loop {
                                if event::poll(std::time::Duration::from_millis(100))? {
                                    if let Event::Key(key_event) = event::read()? {
                                        if key_event.kind == KeyEventKind::Press {
                                            match key_event.code {
                                                KeyCode::Char('s') => {
                                                    save(&data, &openfile)?;
                                                    break;
                                                },
                                                KeyCode::Char('n') => {
                                                    print!("\x1b[s");
                                                    print!("\x1b]12;#00FF00");
                                                    print!("\x1b[{};0H", h + 1);
                                                    for _ in 0..("Your changes will be written to ".to_owned() + &openfile + ". Press s again to save, press n to save as a certain filename, press x to export a png, or press escape to cancel.").len() {
                                                        print!(" ");
                                                    }
                                                    print!("Enter file name: ");
                                                    io::stdout().flush().expect("Failed to flush stdout");
                                                    let mut fname = String::new();
                                                    disable_raw_mode()?;
                                                    stdin().read_line(&mut fname).expect("Did not enter a correct string");
                                                    enable_raw_mode()?;
                                                    print!("\x1b[1A");
                                                    for _ in 0..("Your changes will be written to ".to_owned() + &openfile + ". Press s again to save, press n to save as a certain filename, press x to export a png, or press escape to cancel.").len() {
                                                        print!(" ");
                                                    }
                                                    save(&data, &fname)?;
                                                    openfile = fname;
                                                    print!("\x1b[1 q");
                                                    print!("\x1b[u");
                                                    break;
                                                },
                                                KeyCode::Esc => {
                                                    break;
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            print!("\x1b[{};0H", h + 1);
                            for _ in 0..("Your changes will be written to ".to_owned() + &openfile + ". Press s again to save, press n to save as a certain filename, or press escape tocancel.").len() {
                                print!(" ");
                            }
                            print!("\x1b[u");
                        },
                        KeyCode::Enter => {
                            col = String::new();
                            print!("\x1b[s");
                            print!("\x1b[3 q");
                            print!("\x1b]12;#00FF00");
                            print!("\x1b[{};0H", h + 1);
                            print!("Enter RGB Colour Code (rrrgggbbb) or one-character ST shorthand (rgbpcykw): ");
                            io::stdout().flush().expect("Failed to flush stdout");
                            disable_raw_mode()?;
                            stdin().read_line(&mut col).expect("Did not enter a correct string");
                            enable_raw_mode()?;
                            let redstr: &str;
                            let greenstr: &str;
                            let bluestr: &str;
                            let nonred: &str;
                            if col.len() - 1 == 1 {
                                redstr = match col.borrow() {
                                    "r\n" | "p\n" | "y\n" | "w\n" => "255",
                                    _ => "000",
                                };
                                bluestr = match col.borrow() {
                                    "b\n" | "p\n" | "c\n" | "w\n" => "255",
                                    _ => "000",
                                };
                                greenstr = match col.borrow() {
                                    "g\n" | "y\n" | "c\n" | "w\n" => "255",
                                    _ => "000",
                                };
                            } else if col.len() - 1 == 9 {
                                (redstr, nonred) = col.split_at(3);
                                (greenstr, bluestr) = nonred.split_at(3);
                            } else {
                                print!("\x1b[{};0H", h + 1);
                                for _ in 0.."Enter RGB Colour Code (rrrgggbbb) or one-character ST shorthand (rgbpcykw): ".len() + col.len() {
                                    print!(" ");
                                }
                                print!("\x1b[{};0H", h + 1);
                                print!("Incorrect colour length (must be exactly nine characters: rrrgggbbb). Press enter to continue");
                                io::stdout().flush().expect("Failed to flush stdout");
                                loop {
                                    if event::poll(std::time::Duration::from_millis(100))? {
                                        if let Event::Key(key_event) = event::read()? {
                                            if key_event.kind == KeyEventKind::Press {
                                                match key_event.code {
                                                    KeyCode::Enter => {
                                                        break;
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                print!("\x1b[{};0H", h + 1);
                                for _ in 0.."Incorrect colour length (must be exactly nine characters: rrrgggbbb). Press enter to continue".len() {
                                    print!(" ");
                                }
                                print!("\x1b[1 q");
                                print!("\x1b[u");
                                continue;
                            }
                            red = redstr.parse().expect("Failed to parse redstr");
                            green = greenstr.parse().expect("Failed to parse greenstr");
                            blue = bluestr.replace('\n', "").parse().expect("Failed to parse bluestr");
                            print!("\x1b[1A");
                            for _ in 0.."Enter RGB Colour Code (rrrgggbbb) or one-character colour shorthand (rgbpcykw): 255255255".len() {
                                print!(" ");
                            }
                            print!("\x1b[1 q");
                            print!("\x1b[u");
                        },
                        _ => {}
                    }
                }
            }
        }
        if paint == true {
            print!("\x1b[48;2;{red};{green};{blue}m  \x1b[0m");
            print!("\x1b[2D");
            data.set_pos(x as usize, y as usize, [red, green, blue]);
        }
        let redinverse = data.get_pos(x as usize, y as usize)[0] as i32 * -1 + 255;
        let greeninverse = data.get_pos(x as usize, y as usize)[1] as i32 * -1 + 255;
        let blueinverse = data.get_pos(x as usize, y as usize)[2] as i32 * -1 + 255;
        let mut redhex = String::new();
        let mut greenhex = String::new();
        let mut bluehex = String::new();
        let _ = write!(redhex, "{:02x}", redinverse);
        let _ = write!(greenhex, "{:02x}", greeninverse);
        let _ = write!(bluehex, "{:02x}", blueinverse);
        print!("\x1b]12;#{redhex}{greenhex}{bluehex}");
        io::stdout().flush().expect("Failed to flush stdout");
    }
    disable_raw_mode()?;
    print!("\x1b[2 q");
    print!("\x1b]12;#00FF00");
    print!("\x1b[?1049l");
    Ok(())
}
