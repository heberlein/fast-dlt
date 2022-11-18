use fallible_iterator::FallibleIterator;
use fast_dlt::DltFile;
use fast_dlt::Result;

fn main() -> Result<()> {
    let start = std::time::Instant::now();
    let data = std::fs::read("test-files/helloworld.dlt").unwrap();
    let elapsed = start.elapsed();
    println!("Opened file in {:.2} seconds", elapsed.as_secs_f64());
    let start = std::time::Instant::now();
    let file = DltFile::new(&data);
    let messages = file.collect::<Vec<_>>()?;
    let elapsed = start.elapsed();

    print!(
        "Parsed {} messages in {:.2}s ({:.2} per seconds, {:.2} MiB/s)",
        messages.len(),
        elapsed.as_secs_f64(),
        messages.len() as f64 / elapsed.as_secs_f64(),
        (data.len() / (1024 * 1024)) as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}
