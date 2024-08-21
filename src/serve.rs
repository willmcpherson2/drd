use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

pub fn serve(port: u16, timeout: u64) -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming().flatten() {
        handle_connection(stream, timeout);
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, timeout: u64) {
    stream
        .set_read_timeout(Some(Duration::from_millis(timeout)))
        .unwrap();

    let mut buffer = Vec::new();
    match stream.read_to_end(&mut buffer) {
        Ok(_) => {
            let received = String::from_utf8_lossy(&buffer);
            println!("Received: {}", received);

            let response = format!("Server received: {}", received);
            stream.write_all(response.as_bytes()).unwrap();
        }
        Err(e) => println!("Error reading from connection: {}", e),
    }
}
