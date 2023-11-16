use std::net::TcpStream;
use std::io::{self, Write, Read};
use std::thread;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("0.0.0.0:8080")?;
    let mut cloned_stream = stream.try_clone()?;
    println!("Chat open:");

    thread::spawn(move ||
        loop {

            let mut read_buffer = [0; 1024];
    
            let bytes_read = cloned_stream.read(&mut read_buffer).unwrap_or_else(|e| {
                eprintln!("Error while reading stream: {}", e);
                return 0;
            });
    
            let message_len = match read_buffer.iter().position(|&x| x == b'\0') {
                Some(index) => index,
                None => bytes_read
            };
            
            let message = String::from_utf8_lossy(&read_buffer[..message_len]).to_string();
            println!("{}", message);    
        }
    );

    loop {
        let mut msg_buffer = String::new(); 
        io::stdin().read_line(&mut msg_buffer).unwrap();
        let msg = msg_buffer.trim().to_string();
        if msg == "quit" {break}
        stream.write(msg.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    println!("Closing chat...");
    Ok(())
}