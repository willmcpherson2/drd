use crate::{eval, parse, serialise, Env, Exp};

use std::{
    fs,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

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
            match parse(&text) {
                Ok(parsed) => {
                    let env = read_env(&parsed, dir).unwrap();
                    match eval(&parsed, &env) {
                        Ok((result, env)) => {
                            write_env(&parsed, dir, &env);
                            let response = serialise(result);
                            stream.write_all(response.as_bytes()).unwrap();
                        }
                        Err(e) => println!("Error evaluating expression: {}", e),
                    }
                }
                Err(e) => println!("Error parsing expression: {}", e),
            }
        }
        Err(e) => println!("Error reading from connection: {}", e),
    }
}

fn read_env(exp: &Exp, dir: &str) -> io::Result<Env> {
    let reads = analyse_reads(exp, &vec![]);

    let env = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|file| {
            let path = file.path();
            let var = path.file_name().unwrap().to_string_lossy().to_string();
            if !reads.contains(&var) {
                return None;
            }

            let mut file = fs::File::open(&path).ok()?;
            let mut text = String::new();
            file.read_to_string(&mut text).ok()?;
            parse(&text).ok().map(|program| (var, program))
        })
        .collect();

    Ok(env)
}

fn write_env(exp: &Exp, dir: &str, env: &Env) {
    let writes = analyse_writes(exp);

    for (filename, program) in env {
        if !writes.contains(filename) {
            continue;
        }

        let path = format!("{}/{}", dir, filename);
        let mut file = fs::File::create(&path).unwrap();
        let text = serialise(program.clone());
        file.write_all(text.as_bytes()).unwrap();
    }
}

fn analyse_reads(exp: &Exp, defined: &Vec<String>) -> Vec<String> {
    match exp {
        Exp::Let(var, exp, body) => [
            analyse_reads(exp, defined),
            analyse_reads(body, &[&[var.clone()], defined.as_slice()].concat()),
        ]
        .concat(),
        Exp::Select(_, r) => analyse_reads(r, defined),
        Exp::Where(l, r) => [analyse_reads(l, defined), analyse_reads(r, defined)].concat(),
        Exp::Union(l, r) => [analyse_reads(l, defined), analyse_reads(r, defined)].concat(),
        Exp::Difference(l, r) => [analyse_reads(l, defined), analyse_reads(r, defined)].concat(),
        Exp::Product(l, r) => [analyse_reads(l, defined), analyse_reads(r, defined)].concat(),
        Exp::Table(_, r) => r
            .iter()
            .flat_map(|exp| analyse_reads(exp, defined))
            .collect(),
        Exp::Or(l, r) => [analyse_reads(l, defined), analyse_reads(r, defined)].concat(),
        Exp::Equals(l, r) => [analyse_reads(l, defined), analyse_reads(r, defined)].concat(),
        Exp::And(l, r) => [analyse_reads(l, defined), analyse_reads(r, defined)].concat(),
        Exp::Not(exp) => analyse_reads(exp, defined),
        Exp::Var(var) if !defined.contains(var) => vec![var.clone()],
        _ => vec![],
    }
}

fn analyse_writes(exp: &Exp) -> Vec<String> {
    match exp {
        Exp::Let(var, _, body) => [vec![var.clone()], analyse_writes(body)].concat(),
        _ => vec![],
    }
}
