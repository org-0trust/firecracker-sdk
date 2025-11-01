use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::vm::firercracker_process::FirecrackerProcess;

pub mod firercracker_process;
pub mod vm_socket;

pub struct VM {
    stream: UnixStream,
    firercracker_proc: FirecrackerProcess,
}

impl VM {
    pub fn new(stream: UnixStream, process: FirecrackerProcess) -> Self {
        Self {
            stream,
            firercracker_proc: process,
        }
    }

    async fn send_raw(&mut self, raw: &[u8]) -> Result<()> {
        self.stream.write_all(raw).await?;
        Ok(())
    }

    async fn read_raw(&mut self, raw: &mut [u8]) -> Result<usize> {
        Ok(self.stream.read(raw).await?)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::tempdir;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UnixListener,
    };

    use crate::vm::vm_socket::VMSocket;

    #[tokio::test]
    async fn unix_socket_connect_test() -> Result<()> {
        let dir = tempdir()?;
        let socket = dir.path().join("echo.socket");
        let lis = UnixListener::bind(&socket)?;

        let server = tokio::spawn(async move {
            let (mut socket, _) = lis.accept().await?;
            let mut buf = [0u8; 64];
            let n = socket.read(&mut buf).await?;
            assert_eq!(&buf[..n], b"ping");
            socket.write_all(b"pong").await?;
            Ok::<_, anyhow::Error>(())
        });

        let mut client = VMSocket::new()?.connect(socket).await?;
        client.send_raw(b"ping").await?;
        let mut buf = [0u8; 64];
        let n = client.read_raw(&mut buf).await?;
        assert_eq!(&buf[..n], b"pong");

        server.await??;

        dir.close()?;
        Ok(())
    }
}
