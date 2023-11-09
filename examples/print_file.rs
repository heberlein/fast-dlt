use std::{error::Error, path::PathBuf};

use fast_dlt::file::DltFile;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(path) = std::env::args().nth(1).map(PathBuf::from) else {
        return Err("This example expects a path to a DLT file!".into());
    };

    let data = std::fs::read(path)?;

    let file = DltFile::new(&data);

    file.flat_map(Result::ok)
        .for_each(|message| println!("{message}"));

    Ok(())
}
