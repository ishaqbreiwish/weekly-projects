
use std::net::{TcpListener, TcpStream};   // networking
use std::thread;                          // spawn threads
use std::sync::{Arc, Mutex};              // share data safely across threads
use std::io::{BufRead, BufReader, Write};


fn handle_client(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) -> std::io::Result<()> {
    // we are going to use buf reader to read the tcp stream
    let addr = stream.peer_addr()?; // idenifier for the client
    let reader = BufReader::new(stream);
    let mut removals = Vec::new(); // in case we need to mark for removal later

    for line in reader.lines() {
        let message = match line {
            Ok(s) => s,
            Err(_) => break,
        };

         // assuming that we want a groupchat style thing so each client gets each emssage
        let mut temp_clients = clients.lock().unwrap();
        removals.clear();

        for (i, client) in temp_clients.iter_mut().enumerate() { // iter_mut lets you call write methods on each stream
             // attempts to send the message
             // if it doesnt work  then that client is marked for removal
            if client.write_all(format!("{}\n", message).as_bytes()).is_err() { 
                removals.push(i);
            } else { // if it doesnt fail then we call flush() to force the message out of rusts buffer immediatley
                let _ = client.flush();
            }
        }

         // remove all the dead connections we market
        // needs to be in reverse order so we dont shift indices down
        for idx in removals.iter().rev() {
            temp_clients.remove(*idx);
        }
    } 


    // once the loop ends thats a sign that the client disconnected because our buffer is now empty
    let mut temp_clients = clients.lock().unwrap();
    // we use ok to conver the result into an option 
    // position to internally scan the vec and inf the position
    if let Some(i) = temp_clients.iter().position(|c| c.peer_addr().ok() == Some(addr)) {
        temp_clients.remove(i);
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    // create a tcp  listener bound to an address
    let listener = TcpListener::bind("0.0.0.0:3000")?;

    // we want to make shared_clients a vector because we know that we want it to grow and shrink dynamically

    // arc is for reference-counted shared ownership; its a smart way to count how many threads wants to access a piece of data
    // rust keeps a hidden coutner of how many references (owners) there are
    // every time you arc::clone the count goes up
    //every time one onwer goes away the count does down
    // when the count hits zero the data is dropped

    // mutex stands for mutual exclusion and its way to lock down a piece of data so you dont have a data race
    let shared_clients = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
            match stream {
                Ok(stream) =>  {
                    // when you call .lock() you get a MutexGuard<Vec<TcpSTream>>
                    // this mutexguard temporarily owns access to the vec
                    // by saving the mutexguard in clients, clients becomes the mutexguard
                    // so now clients is kind of like &mut Vec<TcpStream>
                    // when clients goes out of scope the guard is dropped , which releases the lock so other threads can a
                    // access vec
                    // the vec inside shared_clients now contains the new stream
                    let mut clients = shared_clients.lock().unwrap(); // we unwrap because mutex might fail if another thread panics
                    clients.push(stream.try_clone()?);
                    
                    let clients_ref = Arc::clone(&shared_clients);
                    thread::spawn(move || { // we use move so ownership of the variables in the closure goes to the thread
                        let _ = handle_client(stream, clients_ref);
                    });
                }
                Err(e) => {
                    println!("Connection failed: {}", e);
                }
        }
    }

    Ok(())
}