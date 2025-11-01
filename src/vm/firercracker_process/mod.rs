use tokio::process::Child;

pub mod firecracker_startup;

pub struct FirecrackerProcess {
    process: Child,
}

impl FirecrackerProcess {
    pub(crate) fn new(child: Child) -> Self {
        Self { process: child }
    }
}
