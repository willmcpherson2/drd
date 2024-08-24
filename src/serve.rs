use crate::{eval, parse, serialise, Cli, Env, Exp};

use std::{
    collections::HashSet,
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

    let mut text = String::new();
    stream
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let parsed = parse(&text)?;

    let filenames = filenames(&cli.directory).map_err(|e| e.to_string())?;

    let reads = filenames
        .intersection(&analyse_reads(&parsed, &empty()))
        .cloned()
        .collect();
    let env = read_env(&cli.directory, &reads)?;

    let (result, env) = eval(&parsed, &env)?;

    let writes = filenames
        .intersection(&analyse_writes(&parsed))
        .cloned()
        .collect::<HashSet<_>>();
    let env = env
        .into_iter()
        .filter(|(k, _)| writes.contains(k))
        .collect();
    write_env(&cli.directory, &env).map_err(|e| e.to_string())?;

    let response = serialise(result);
    stream
        .write_all(response.as_bytes())
        .map_err(|e| e.to_string())?;

    if cli.verbose {
        println!();
        println!("Input: {}", serialise(parsed));
        println!("Result: {}", response);
        println!(
            "Reads: {}",
            reads.into_iter().collect::<Vec<_>>().join(", ")
        );
        println!(
            "Writes: {}",
            writes.into_iter().collect::<Vec<_>>().join(", ")
        );
    }

    Ok(())
}

fn read_env(dir: &str, reads: &HashSet<String>) -> Result<Env, String> {
    let env = reads
        .iter()
        .map(|filename| {
            let path = format!("{}/{}", dir, filename);
            let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let exp = parse(&text)?;
            Ok((filename.clone(), exp))
        })
        .collect::<Result<Env, String>>()?;

    Ok(env)
}

fn write_env(dir: &str, env: &Env) -> io::Result<()> {
    env.iter().try_for_each(|(filename, exp)| {
        let path = format!("{}/{}", dir, filename);
        let serialised = serialise(exp.clone());
        fs::write(path, serialised)
    })
}

fn filenames(dir: &str) -> io::Result<HashSet<String>> {
    let files = fs::read_dir(dir)?
        .filter_map(|file| Some(file.ok()?.path().file_name()?.to_str()?.to_string()))
        .collect();

    Ok(files)
}

fn analyse_reads(exp: &Exp, defined: &HashSet<String>) -> HashSet<String> {
    match exp {
        Exp::Let(var, exp, body) => union(
            analyse_reads(exp, defined),
            analyse_reads(body, &union(single(var), defined.clone())),
        ),
        Exp::Select(_, r) => analyse_reads(r, defined),
        Exp::Where(l, r) => union(analyse_reads(l, defined), analyse_reads(r, defined)),
        Exp::Union(l, r) => union(analyse_reads(l, defined), analyse_reads(r, defined)),
        Exp::Difference(l, r) => union(analyse_reads(l, defined), analyse_reads(r, defined)),
        Exp::Product(l, r) => union(analyse_reads(l, defined), analyse_reads(r, defined)),
        Exp::Table(_, r) => r
            .iter()
            .flat_map(|exp| analyse_reads(exp, defined))
            .collect(),
        Exp::Or(l, r) => union(analyse_reads(l, defined), analyse_reads(r, defined)),
        Exp::Equals(l, r) => union(analyse_reads(l, defined), analyse_reads(r, defined)),
        Exp::And(l, r) => union(analyse_reads(l, defined), analyse_reads(r, defined)),
        Exp::Not(exp) => analyse_reads(exp, defined),
        Exp::Var(var) if !defined.contains(var) => single(var),
        _ => empty(),
    }
}

fn analyse_writes(exp: &Exp) -> HashSet<String> {
    match exp {
        Exp::Let(var, _, body) => union(single(var), analyse_writes(body)),
        _ => empty(),
    }
}

fn empty() -> HashSet<String> {
    HashSet::new()
}

fn single(s: &str) -> HashSet<String> {
    HashSet::from([s.to_string()])
}

fn union(a: HashSet<String>, b: HashSet<String>) -> HashSet<String> {
    a.union(&b).cloned().collect()
}
