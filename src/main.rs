use std::{
    fs::{self, OpenOptions}, 
    io::{self, stdout, Stdout, Write}, 
    path::PathBuf, 
    process::exit, 
    time::{Duration, SystemTime, UNIX_EPOCH}
};

use crossterm::{
    cursor, 
    event::{poll, read, Event, KeyCode}, 
    terminal::{enable_raw_mode, disable_raw_mode},
    QueueableCommand
};

use clap::{Arg, Command};
use colored::Colorize;
use dirs;
use chrono;

struct TimeUnits {
    h: u8,
    m: u8,
    s: u8,
}

fn get_time_units(s: u64) -> TimeUnits {
    TimeUnits {
        h: (s / 3600) as u8,
        m: ((s - ((s / 3600) * 3600)) / 60) as u8,
        s: (s - (((s - ((s / 3600) * 3600)) / 60) * 60)) as u8,
    }
}

fn get_timer_string(s: u64) -> String {
    let time_units: TimeUnits = get_time_units(s);

    let h_0: char = if time_units.h < 10 {'0'} else {0 as char};
    let s_0: char = if time_units.s < 10 {'0'} else {0 as char};
    let m_0: char = if time_units.m < 10 {'0'} else {0 as char};

    format!("{}{}:{}{}:{}{}", 
        h_0, time_units.h, m_0, time_units.m, s_0, time_units.s)
}

fn wipe_screen(stdout: &mut Stdout) {
    // get width of terminal window in columns
    if let Some((w, h)) = term_size::dimensions() {
        // for loop in ordered to wipe clean every line
        for _ in 1..(h - cursor::position().unwrap().1 as usize) {
            print!("{}\n", " ".repeat(w)); // wipe clean the line
        }
        _ = stdout.queue(cursor::RestorePosition);
    }
}

fn cli() -> Command {
    Command::new("tt")
        .version("0.1.0")
        .author("Thomas Spina <thomas@thomasspina.com>")
        .about("Simple work timer that tracks history")
        .subcommand(Command::new("start")
                .short_flag('s')
                .long_flag("start")
                .about("Starts the timer. Press 'p' to pause and 'q' to quit."))
        .subcommand(Command::new("yesterday")
                .short_flag('y')
                .long_flag("yd")
                .about("Gets yesterday's performance data."))      
        .subcommand(Command::new("lastweek")
                .short_flag('w')
                .long_flag("lw")
                .about("Gets last week's performance data."))
        .subcommand(Command::new("lastmonth")
                .short_flag('m')
                .long_flag("lm")
                .about("Gets last month's performance data."))
        .subcommand(Command::new("lastxdays")
                .short_flag('x')
                .long_flag("lx")
                .about("Gets last x days' performance data.")
                .arg(Arg::new("days")))
        .subcommand(Command::new("range")
                .short_flag('r')
                .long_flag("range")
                .about("Gets performance data for a range of days inclusive. Dates should be in the format YYYY-MM-DD.")
                .arg(Arg::new("start"))
                .arg(Arg::new("end")))
}

fn start_timer(support_file_path: Option<PathBuf>) -> io::Result<()> {
    let mut paused: bool = false;
    let mut s_work: u64 = 0;
    let mut s_pause: u64 = 0;
    let mut stdout: Stdout = stdout();
    let now: SystemTime = SystemTime::now();

    stdout.queue(cursor::SavePosition)?; 

    // Main timer loop
    loop {
        match now.elapsed() {
            Ok(elapsed) => {
                stdout.queue(cursor::RestorePosition)?;
                wipe_screen(&mut stdout);

                if !paused {
                    s_work = &elapsed.as_secs() - s_pause;
                } else {
                    s_pause = &elapsed.as_secs() - s_work;
                }

                print!("{} {}\n{} {}",
                    "Work:".bold().white(),
                    get_timer_string(s_work).truecolor(190, 190, 190),
                    "Play:".bold().truecolor(0, 0, 0).on_truecolor(249, 109, 0), 
                    get_timer_string(s_pause).truecolor(190, 190, 190));
                stdout.flush().unwrap();
            }
            Err(e) => {
                println!("Error: {e:?}");
            }
        }
        
        // check for key presses
        enable_raw_mode()?;
        if poll(Duration::from_millis(1_000))? { // acts as the "sleep"
            let event: Event = read()?;

            if event == Event::Key(KeyCode::Char('p').into()) {
                paused = !paused;
            }

            if event == Event::Key(KeyCode::Char('q').into()) {
                let time_stamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();

                // save session data if file exists
                match support_file_path {
                    Some(val) => {
                        let file: fs::File = OpenOptions::new()
                                            .write(true)
                                            .append(true)
                                            .open(&val)?;

                        let mut wtr: csv::Writer<fs::File> = csv::Writer::from_writer(file);
                        wtr.write_record(&[s_work.to_string(), s_pause.to_string(), time_stamp.to_string()])?;
                        wtr.flush()?;
                    }
                    _ => {}
                }

                disable_raw_mode()?;
                exit(0);
            }
        }
        disable_raw_mode()?;
    }
}

fn unix_epoch_to_date_string(s: u64) -> String {
    let date_time = chrono::NaiveDateTime::from_timestamp_opt(s as i64, 0).unwrap();
    date_time.format("%Y-%m-%d").to_string()
}
fn main() {
    // create support file
    let home_dir_path: Option<PathBuf> = dirs::home_dir();
    let mut support_file_path: Option<PathBuf> = None;

    match home_dir_path {
        Some(mut val) => {
            val.push(".timer_data");

            // create support file if not exist
            if !fs::metadata(&val).is_ok() {
                let file = fs::File::create(&val).unwrap(); // create data file
                let mut wtr = csv::Writer::from_writer(file);

                wtr.write_record(&["Work", "Play", "End"]).unwrap(); // init column names
                wtr.flush().unwrap();
            }

            support_file_path = Some(val);
        },
        None => { eprintln!("HOME directory not found in $PATH. Timer data cannot be saved."); }
    }

    match cli().get_matches().subcommand() {
        Some(("yesterday", _)) => {
            // TODO : UNIX_EPOCH not local time ? Probably wanna save local time

            match support_file_path {
                Some(ref val) => {
                    let file = fs::File::open(&val).unwrap();
                    let mut rdr = csv::Reader::from_reader(file);


                    for result in rdr.records() {
                        match result {
                            Ok(record) => {
                                let end: u64 = record[2].parse().unwrap();
                                let date = unix_epoch_to_date_string(end);
                                println!("{}", date)
                            }
                            Err(e) => { eprintln!("Error: {e:?}"); }
                        }
                    }
                }
                None => { eprintln!("No support file found. Timer data cannot be saved."); }
            }
            
        }
        Some(("lastweek", _)) => {
            // TODO : get last week's data
        }
        Some(("lastmonth", _)) => {
            // TODO : get last month's data
        }
        Some(("lastxdays", _)) => {
            // TODO : get last x days' data
        }
        Some(("range", _)) => {
            // TODO : get range of days' data
        }
        Some(("start", _)) => {
            match start_timer(support_file_path) {
                Ok(_) => {}
                Err(e) => { eprintln!("Error: {e:?}"); }
            }
        }
        _ => {}
    }
}