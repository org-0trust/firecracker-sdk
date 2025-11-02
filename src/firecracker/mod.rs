use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

pub mod firecracker_socket;
pub mod firercracker_process;

/// A structure that allows you to work safely with VMs
///
/// Exemple:
/// ```no_run
/// let vm_process = FirecrackerStartup::new()
///     .api_sock("/tmp/some.socket")
///     .start().unwrap();
///
/// let firecracker_socket = FirecrackerSocket::new().unwrap();
/// let firecracker_stream = firecracker_socket.connect("/tmp/some.socket");
/// ```
#[allow(unused)]
pub struct FirecrackerStream {
    stream: UnixStream,
}

#[allow(unused)]
impl FirecrackerStream {
    pub(crate) fn new(stream: UnixStream) -> Self {
        Self { stream }
    }

    async fn send_raw(&mut self, raw: &[u8]) -> Result<()> {
        self.stream.write_all(raw).await?;
        Ok(())
    }

    async fn read_raw(&mut self, raw: &mut [u8]) -> Result<usize> {
        Ok(self.stream.read(raw).await?)
    }

    /// Safely closes the unix stream
    pub async fn close(mut self) -> Result<()> {
        self.stream.shutdown().await?;
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

    use crate::firecracker::firecracker_socket::FirecrackerSocket;

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

        let mut client = FirecrackerSocket::new()?.connect(socket).await?;
        client.send_raw(b"ping").await?;
        let mut buf = [0u8; 64];
        let n = client.read_raw(&mut buf).await?;
        assert_eq!(&buf[..n], b"pong");

        join!(server).0??;

        dir.close()?;
        Ok(())
    }
}
