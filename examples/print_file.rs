use std::{error::Error, path::PathBuf};

use fallible_iterator::FallibleIterator;
use fast_dlt::file::DltFile;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(path) = std::env::args().nth(1).map(PathBuf::from) else {
      return Err("This example expects a path to a DLT file!".into());  
    };

    let data = std::fs::read(path)?;

    let file = DltFile::new(&data);

    file.for_each(|msg| Ok(println!("{msg}")))?;

    Ok(())
}
