use std::{error::Error, path::PathBuf, time::Instant};

use fast_dlt::file::DltFile;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(path) = std::env::args().nth(1).map(PathBuf::from) else {
        return Err("This example expects a path to a DLT file!".into());
    };

    let data = std::fs::read(path)?;

    let file = DltFile::new(&data);

    let start = Instant::now();
    let count: usize = file
        .map(|message| match message {
            Ok(message) => match message.payload {
                fast_dlt::payload::Payload::NonVerbose(_) => 0,
                fast_dlt::payload::Payload::Verbose(v) => v.arguments().count(),
            },
            Err(_) => 0,
        })
        .sum();
    let elapsed = start.elapsed();

    println!(
        "Parsed {count} arguments in {:.3}s ({:.2} per second, {:.2} MiB/s)",
        elapsed.as_secs_f32(),
        count as f64 / elapsed.as_secs_f64(),
        (data.len() as f64 / f64::from(1024 * 1024) / elapsed.as_secs_f64())
    );
    Ok(())
}
