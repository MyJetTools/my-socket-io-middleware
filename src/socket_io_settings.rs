use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

pub struct SocketIoSettings {
    ping_interval: AtomicU64,
    ping_timeout: AtomicU64,
}

impl SocketIoSettings {
    pub fn default() -> Self {
        Self {
            ping_interval: AtomicU64::new(6000),
            ping_timeout: AtomicU64::new(2000),
        }
    }

    pub fn get_ping_interval(&self) -> Duration {
        Duration::from_millis(self.ping_interval.load(Ordering::Relaxed))
    }

    pub fn get_ping_timeout(&self) -> Duration {
        Duration::from_millis(self.ping_timeout.load(Ordering::Relaxed))
    }

    pub fn set_ping_interval(&self) -> Duration {
        Duration::from_millis(self.ping_interval.load(Ordering::SeqCst))
    }

    pub fn set_ping_timeout(&self) -> Duration {
        Duration::from_millis(self.ping_timeout.load(Ordering::SeqCst))
    }
}
