use hyper::body::Body;
use tokio::net::UnixStream;

use crate::io;

pub async fn get_client<B>(
    socket: &str,
) -> Result<hyper::client::conn::http1::SendRequest<B>, Box<dyn std::error::Error>>
where
    B: Body + Send + Sync + 'static,
    <B as Body>::Data: Send,
    <B as Body>::Error: std::error::Error + Send + Sync,
{
    let stream = UnixStream::connect(socket).await?;
    let io = io::TokioIo::new(stream);

    let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    Ok(sender)
}
