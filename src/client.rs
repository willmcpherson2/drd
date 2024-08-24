use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
pub async fn client(text: &str, url: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(url).await?;

    stream.write_all(text.as_bytes()).await?;
    stream.shutdown().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    println!("{}", response);

    Ok(())
}
