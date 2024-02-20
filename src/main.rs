// TODO : fix position bug where when resising, the timer moves
// TODO : add save when exiting

use std::{
    io::{self, stdout, Write, Stdout},
    process::exit,
    time::{Duration, SystemTime}
};

use crossterm::{
    cursor, 
    event::{poll, read, Event, KeyCode}, 
    terminal::{enable_raw_mode, disable_raw_mode},
    QueueableCommand
};

use colored::Colorize;

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

fn main() -> io::Result<()> {
    let mut paused: bool = false;
    let mut s_work: u64 = 0;
    let mut s_pause: u64 = 0;
    
    let now = SystemTime::now();
    let mut stdout = stdout();

    stdout.queue(cursor::SavePosition)?; 
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
        if poll(Duration::from_millis(1_000))? {
            let event = read()?;

            if event == Event::Key(KeyCode::Char('p').into()) {
                paused = !paused;
            }

            if event == Event::Key(KeyCode::Char('q').into()) {
                disable_raw_mode()?;
                exit(0);
            }
        }
        disable_raw_mode()?;
    }
}