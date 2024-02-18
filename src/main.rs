// TODO : fix position bug where when resising, the timer moves
// TODO : add save when exiting

use std::{
    io::{self, stdout, Write},
    process::exit,
    time::{Duration, SystemTime}
};

use crossterm::{
    cursor, 
    event::{poll, read, Event, KeyCode}, 
    terminal::{enable_raw_mode, disable_raw_mode},
    QueueableCommand
};

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
                if !paused {
                    s_work = &elapsed.as_secs() - s_pause;
                    let time_units: TimeUnits = get_time_units(s_work);
                    stdout.queue(cursor::RestorePosition)?;
                    
                    // get width of terminal window in columns
                    if let Some((w, h)) = term_size::dimensions() {
                        // for loop in ordered to wipe clean every line
                        for _ in 1..(h - cursor::position().unwrap().1 as usize) {
                            print!("{}\n", " ".repeat(w)); // wipe clean the line
                        }
                    }
    
                    stdout.queue(cursor::RestorePosition)?;
                    
                    // add a zero in front if below ten
                    let h_0: char = if time_units.h < 10 {'0'} else {0 as char};
                    let s_0: char = if time_units.s < 10 {'0'} else {0 as char};
                    let m_0: char = if time_units.m < 10 {'0'} else {0 as char};
    
                    print!("\r{}{}:{}{}:{}{}", 
                            h_0, time_units.h, m_0, time_units.m, s_0, time_units.s);
                    stdout.flush().unwrap();
                } else {
                    s_pause = &elapsed.as_secs() - s_work;
                }
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
                // TODO
                    // toggle over to other time and start adding seconds there
            }

            if event == Event::Key(KeyCode::Char('q').into()) {
                disable_raw_mode()?;
                exit(0);
            }
        }
        disable_raw_mode()?;
    }
}