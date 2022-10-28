use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt::{self, Display},
    fs::File,
    io::{self, BufReader, Write},
    str,
};

use quick_xml::{
    events::Event,
    Reader
};

mod parser;

const BUF_SIZE: usize = 4096; // 4kb at once

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).ok_or("no filename provided")?;
    let f = File::open(&path).map_err(|e| format!("failed to open {}: {}", path, e))?;
    let xmlfile = BufReader::new(f);
    let mut xmlfile = Reader::from_reader(xmlfile);

    let mut buf = Vec::with_capacity(BUF_SIZE);

    loop {
        match xmlfile.read_event_into(&mut buf)? {
            Event::Eof => break,
            ev => println!("{:?}", ev)
        };
        buf.clear();
    }

    Ok(())
}
