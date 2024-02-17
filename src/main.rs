use std::{
    io::{stdout, Write},
    thread::sleep,
    time::{Duration, SystemTime},
};
use crossterm::{QueueableCommand, cursor};

struct TimeUnits {
    h: u8,
    m: u8,
    s: u8,
}

fn get_time_units(s: &u64)->TimeUnits {
    TimeUnits {
        h: (s / 3600) as u8,
        m: ((s - ((s / 3600) * 3600)) / 60) as u8,
        s: (s - (((s - ((s / 3600) * 3600)) / 60) * 60)) as u8,
    }
}

fn main() {
    
    let now = SystemTime::now();
    let mut stdout = stdout();

    _ = stdout.queue(cursor::SavePosition);

    loop {
        match now.elapsed() {
            Ok(elapsed) => {
                let time_units: TimeUnits = get_time_units(&elapsed.as_secs());
                _ = stdout.queue(cursor::RestorePosition);
                
                // get width of terminal window in columns
                if let Some((w, _)) = term_size::dimensions() {
                    print!("\r{}", " ".repeat(w)); // wipe clean the current line
                }

                let h_0: char = if time_units.h < 10 {'0'} else {0 as char};
                let s_0: char = if time_units.s < 10 {'0'} else {0 as char};
                let m_0: char = if time_units.m < 10 {'0'} else {0 as char};

                print!("\r{}{}:{}{}:{}{}", 
                        h_0, time_units.h, m_0, time_units.m, s_0, time_units.s);
                stdout.flush().unwrap();
            }
            Err(e) => {
                println!("Error: {e:?}");
            }
        }
        sleep(Duration::new(1, 0));
    }
}