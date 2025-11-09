use anyhow::Result;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::domain::http::Http;

/// A structure that allows you to work safely with VMs
pub(crate) struct Stream {
    stream: UnixStream,
}

#[allow(unused)]
impl Stream {
    pub(crate) fn new(stream: UnixStream) -> Self {
        Self { stream }
    }

    pub async fn send_user_request(&mut self, req: Http) -> Result<()> {
        self.send_raw(&req.build()).await?;
        Ok(())
    }

    pub async fn read_req(&mut self) -> Result<Http> {
        let mut buf = vec![];
        self.read_raw(&mut buf).await?;
        Ok(Http::from(buf))
    }

    pub(crate) async fn send_raw(&mut self, raw: &[u8]) -> Result<()> {
        self.stream.write_all(raw).await?;
        Ok(())
    }

    pub(crate) async fn read_raw(&mut self, raw: &mut Vec<u8>) -> Result<usize> {
        Ok(self.stream.read_buf(raw).await?)
    }

    /// Safely closes the unix stream
    pub async fn close(mut self) -> Result<()> {
        self.stream.shutdown().await?;
        fs::remove_file(self.stream.peer_addr()?.as_pathname().unwrap()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use tempfile::tempdir;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        join,
        net::UnixListener,
    };

    use crate::infrastructure::connection::socket::Socket;

    #[tokio::test]
    async fn unix_socket_connect_test() -> Result<()> {
        let dir = tempdir()?;
        let socket = dir.path().join("echo.socket");
        let lis = UnixListener::bind(&socket)?;

        assert!(socket.exists());

        let server = tokio::spawn(async move {
            let (mut socket, _) = lis.accept().await?;
            let mut buf = [0u8; 64];
            let n = socket.read(&mut buf).await?;
            assert_eq!(&buf[..n], b"ping");
            socket.write_all(b"pong").await?;
            Ok::<_, anyhow::Error>(())
        });

        let mut client = Socket::new()?.connect(socket).await?;
        client.send_raw(b"ping").await?;
        let mut buf = vec![];
        let n = client.read_raw(&mut buf).await?;
        assert_eq!(&buf[..n], b"pong");

        join!(server).0??;

        dir.close()?;
        Ok(())
    }
}
