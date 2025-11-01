use tokio::net::UnixStream;

pub mod vm_socket;

pub struct VM {
    stream: UnixStream,
}

impl VM {
    pub fn new(stream: UnixStream) -> Self {
        Self { stream }
    }
}
