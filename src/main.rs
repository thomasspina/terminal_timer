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
use chrono::{Utc, NaiveDate, DateTime, Local};

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
        .subcommand(Command::new("today")
                .short_flag('t')
                .long_flag("td")
                .about("Gets today's performance data."))
        .subcommand(Command::new("start")
                .short_flag('s')
                .long_flag("start")
                .about("Starts the timer. Press 'p' to pause and 'q' to quit."))
        .subcommand(Command::new("lastxdays")
                .short_flag('x')
                .long_flag("lx")
                .about("Gets last x days' performance data.")
                .arg(Arg::new("days").num_args(1))
                .arg(Arg::new("sum")
                        .short('s')
                        .long("sum")
                        .num_args(0)))
}

fn unix_to_local_date(unix_time: i64) -> String {
    let utc_datetime: DateTime<Utc> = DateTime::<Utc>::from_timestamp(unix_time, 0).unwrap();
    let local_datetime: DateTime<Local> = utc_datetime.with_timezone(&Local);
    local_datetime.format("%Y-%m-%d").to_string()
}

fn is_last_x_days(x: usize, date_str: String) -> i64 {
    let today = Local::now().naive_local().date();
    let date_x_days_before_today = today - chrono::Duration::days(x.try_into().unwrap());

    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();

    if date <= today && date >= date_x_days_before_today {
        return today.signed_duration_since(date).num_days();
    }

    return -1;
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
        Some(("today", _)) => {
            match support_file_path {
                Some(val) => {
                    let file: fs::File = OpenOptions::new()
                                        .read(true)
                                        .open(&val)
                                        .unwrap();
                    
                    let today: NaiveDate = Local::now().naive_local().date();
                    let today_str: String = today.format("%Y-%m-%d").to_string();
                    
                    let mut rdr = csv::Reader::from_reader(file);
                    let mut total: (u64, u64) = (0, 0);
                    for result in rdr.records() {
                        let record = result.unwrap();
                        let end_date: String = unix_to_local_date(record[2].parse().unwrap());

                        if end_date != today_str {
                            continue;
                        }

                        total.0 += &record[0].parse().unwrap();
                        total.1 += &record[1].parse().unwrap();
                    }

                    println!("{}: {}  {}: {}", 
                            "Work".bold().bright_white(),
                            get_timer_string(total.0), 
                            "Play".bold().bright_white(),
                            get_timer_string(total.1));
                }
                None => { eprintln!("No support file found."); }
            }
        }
        Some(("lastxdays", args)) => {
            match args.get_one::<String>("days") {
                Some(x) => {
                    match x.parse::<usize>() {
                        Ok(num) => { 
                            match support_file_path {
                                Some(val) => {
                                    let file: fs::File = OpenOptions::new()
                                        .read(true)
                                        .open(&val)
                                        .unwrap();

                                    let mut arr: Vec<(u64, u64)> = vec![(0, 0); num];
                                    let mut rdr = csv::Reader::from_reader(file);
                                    for record in rdr.records() {
                                        let result = record.unwrap();
                                        let date = unix_to_local_date(result[2].parse().unwrap());
                                        
                                        let i: i64 = is_last_x_days(num, date) - 1;
                                        if i > -1 { // if day in last_x_days
                                            arr[i as usize].0 += result[0].parse::<u64>().unwrap(); // work
                                            arr[i as usize].1 += result[1].parse::<u64>().unwrap(); // play
                                        }
                                    }
                                    
                                    if *args.get_one::<bool>("sum").unwrap() {
                                        let mut sum: (u64, u64) = (0, 0);
                                        for tup in arr.iter() {
                                            sum.0 += tup.0;
                                            sum.1 += tup.1;
                                        }
                                        println!("{}: {}  {}: {}",
                                                    "Work".bold(),
                                                    get_timer_string(sum.0),
                                                    "Play".bold(),
                                                    get_timer_string(sum.1));
                                    } else {
                                        for (i, tup) in arr.iter().enumerate() {
                                            println!("{} {}: {}  {}: {}",
                                                    i,
                                                    "Work".bold(),
                                                    get_timer_string(tup.0),
                                                    "Play".bold(),
                                                    get_timer_string(tup.1));
                                        }
                                    }
                                }
                                None => { eprintln!("No support file found."); }
                            }
                        }
                        Err(e) => { println!("Provide valid number: {}", e) }
                    }

                }
                None => { println!("No days provided"); }
            }
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