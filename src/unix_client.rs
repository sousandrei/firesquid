use hyper::body::Body;
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;

pub async fn get_client<B>(
    socket: &str,
) -> Result<hyper::client::conn::http1::SendRequest<B>, Box<dyn std::error::Error>>
where
    B: Body + Send + Sync + 'static,
    B::Data: Send,
    B::Error: std::error::Error + Send + Sync,
{
    let stream = UnixStream::connect(socket).await?;
    let io = TokioIo::new(stream);
    let (sender, connection) = hyper::client::conn::http1::handshake(io).await?;

    tokio::spawn(async move {
        if let Err(error) = connection.await {
            tracing::error!(%error, "Unix socket connection failed");
        }
    });

    Ok(sender)
}
