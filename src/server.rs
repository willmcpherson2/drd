use crate::{eval, parse, serialise, Env, Exp, Server};

use std::{collections::HashSet, fs, io, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
pub async fn server(conf: Server) -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], conf.port));
    let listener = TcpListener::bind(addr).await?;

    fs::create_dir_all(&conf.directory)?;

    let conf = Arc::new(conf);

    loop {
        let (stream, _) = listener.accept().await?;
        let conf = Arc::clone(&conf);

        tokio::spawn(async move {
            handle_connection(stream, conf).await.unwrap_or_else(|e| {
                eprintln!("Error handling connection: {}", e);
            });
        });
    }
}

async fn handle_connection(mut stream: TcpStream, conf: Arc<Server>) -> Result<(), String> {
    let mut text = String::new();
    stream
        .read_to_string(&mut text)
        .await
        .map_err(|e| e.to_string())?;
    let parsed = parse(&text)?;

    let filenames = filenames(&conf.directory).map_err(|e| e.to_string())?;

    let reads = filenames
        .intersection(&analyse_reads(&parsed, &empty()))
        .cloned()
        .collect();
    let env = read_env(&conf.directory, &reads).await?;

    let (result, env) = eval(&parsed, &env)?;

    let writes = filenames
        .intersection(&analyse_writes(&parsed))
        .cloned()
        .collect::<HashSet<_>>();
    let env = env
        .into_iter()
        .filter(|(k, _)| writes.contains(k))
        .collect();
    write_env(&conf.directory, &env)
        .await
        .map_err(|e| e.to_string())?;

    let response = serialise(result);
    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    if conf.verbose {
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

async fn read_env(dir: &str, reads: &HashSet<String>) -> Result<Env, String> {
    let mut env = Env::new();
    for filename in reads {
        let path = format!("{}/{}", dir, filename);
        let text = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| e.to_string())?;
        let exp = parse(&text)?;
        env.insert(filename.clone(), exp);
    }
    Ok(env)
}

async fn write_env(dir: &str, env: &Env) -> io::Result<()> {
    for (filename, exp) in env {
        let path = format!("{}/{}", dir, filename);
        let serialised = serialise(exp.clone());
        tokio::fs::write(path, serialised).await?;
    }
    Ok(())
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
