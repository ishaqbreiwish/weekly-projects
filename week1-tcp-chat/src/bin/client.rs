use std::net::{TcpStream};                 // networking
use std::io::{self, BufRead, BufReader, BufWriter, Write};              // read/write to sockets
use std::thread;                          // spawn threads

fn main() -> io::Result<()>{
    let stream = TcpStream::connect("0.0.0.0:3000").expect("Couldn't connect to the server...");

    // clone the stream
    let mut writer = BufWriter::new(stream.try_clone()?);
    let reader_stream = stream.try_clone()?; // for the thread

    // thread reading from the server
    thread::spawn(move || {
        let mut reader = BufReader::new(reader_stream);
    
        for line in reader.lines() {
            match line {
                Ok(msg) => println!("{}", msg),
                Err(_) => break,
            }
        }
    });

    loop {
        // main thread writing to the server
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        writer.write_all(user_input.as_bytes())?;
        writer.flush()?
    }
}
