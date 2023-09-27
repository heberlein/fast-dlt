use std::{error::Error, path::PathBuf};

use fast_dlt::{error::DltError, file::DltFile};

fn main() -> Result<(), Box<dyn Error>> {
    let Some(path) = std::env::args().nth(1).map(PathBuf::from) else {
      return Err("This example expects a path to a DLT file!".into());  
    };

    let data = std::fs::read(path)?;

    let file = DltFile::new(&data);

    file.for_each(|message| match message {
        Ok(message) => println!("{message}"),
        Err(DltError::Recoverable {
            message_len: _,
            index,
            cause,
        }) => println!("recoverable error at byte {index}: {cause}"),
        Err(DltError::Fatal { index, cause }) => println!("Fatal error at byte {index}: {cause}"),
    });

    Ok(())
}
