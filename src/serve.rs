use crate::{eval, parse, serialise, Cli, Env, Exp};

use std::{
    fs,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

pub fn serve(cli: Cli) -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    let listener = TcpListener::bind(addr)?;

    fs::create_dir_all(&cli.directory)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => match handle_connection(stream, &cli) {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to handle connection: {}", e),
            },
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, cli: &Cli) -> Result<(), String> {
    if cli.timeout > 0 {
        stream
            .set_read_timeout(Some(Duration::from_millis(cli.timeout)))
            .map_err(|e| e.to_string())?;
    }

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&buffer);
    let parsed = parse(&text)?;

    let env = read_env(&parsed, &cli.directory).map_err(|e| e.to_string())?;
    let (result, env) = eval(&parsed, &env)?;
    write_env(&parsed, &cli.directory, &env).map_err(|e| e.to_string())?;

    let response = serialise(result);
    stream
        .write_all(response.as_bytes())
        .map_err(|e| e.to_string())?;

    if cli.verbose {
        println!();
        println!("Input: {}", serialise(parsed));
        println!("Result: {}", response);
    }

    Ok(())
}

fn read_env(exp: &Exp, dir: &str) -> io::Result<Env> {
    let reads = analyse_reads(exp, &vec![]);

    let env = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|file| {
            let path = file.path();
            let var = path.file_name()?.to_string_lossy().to_string();
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

fn write_env(exp: &Exp, dir: &str, env: &Env) -> io::Result<()> {
    let writes = analyse_writes(exp);

    for (filename, program) in env {
        if !writes.contains(filename) {
            continue;
        }

        let path = format!("{}/{}", dir, filename);
        let mut file = fs::File::create(&path)?;
        let text = serialise(program.clone());
        file.write_all(text.as_bytes())?;
    }

    Ok(())
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
