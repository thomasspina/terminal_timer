use std::{
    io::{stdout, Write},
    thread::sleep,
    time::{Duration, SystemTime},
    //env,
};

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
    // let args: Vec<String> = env::args().collect();
    
    let now = SystemTime::now();
    let mut stdout = stdout();
    loop {
        match now.elapsed() {
            Ok(elapsed) => {
                let time_units: TimeUnits = get_time_units(&elapsed.as_secs());
                print!("\r{}h{}m{}s", time_units.h, time_units.m, time_units.s);
                stdout.flush().unwrap();
            }
            Err(e) => {
                println!("Error: {e:?}");
            }
        }
        sleep(Duration::new(1, 0));
    }
}