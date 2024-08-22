use std::{
    fs,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use crate::{eval::Env, parse::parse, read_eval, serialise::serialise};

pub fn serve(dir: String, port: u16, timeout: u64) -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr)?;

    fs::create_dir_all(&dir)?;

    for stream in listener.incoming().flatten() {
        handle_connection(stream, &dir, timeout);
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, dir: &str, timeout: u64) {
    if timeout > 0 {
        stream
            .set_read_timeout(Some(Duration::from_millis(timeout)))
            .unwrap();
    }

    let mut buffer = Vec::new();
    match stream.read_to_end(&mut buffer) {
        Ok(_) => {
            let text = String::from_utf8_lossy(&buffer);

            let env = read_env(dir).unwrap();

            let response = match read_eval(&text, &env) {
                Ok((program, env)) => {
                    write_env(dir, &env);
                    serialise(program)
                }
                Err(e) => format!("Error evaluating program: {}", e),
            };

            stream.write_all(response.as_bytes()).unwrap();
        }
        Err(e) => println!("Error reading from connection: {}", e),
    }
}

fn read_env(dir: &str) -> io::Result<Env> {
    let env = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|file| {
            let path = file.path();
            let mut file = fs::File::open(&path).ok()?;
            let mut text = String::new();
            file.read_to_string(&mut text).ok()?;
            parse(&text).ok().map(|program| {
                let var = path.file_name().unwrap().to_string_lossy().to_string();
                (var, program)
            })
        })
        .collect();

    Ok(env)
}

fn write_env(dir: &str, env: &Env) {
    for (filename, program) in env {
        let path = format!("{}/{}", dir, filename);
        let mut file = fs::File::create(&path).unwrap();
        let text = serialise(program.clone());
        file.write_all(text.as_bytes()).unwrap();
    }
}
