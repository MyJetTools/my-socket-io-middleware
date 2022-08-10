use std::{
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::Duration,
};

pub struct SocketIoSettings {
    ping_interval: AtomicU64,
    ping_timeout: AtomicU64,
    max_payload_size: AtomicUsize,
}

impl SocketIoSettings {
    pub fn default() -> Self {
        Self {
            ping_interval: AtomicU64::new(6000),
            ping_timeout: AtomicU64::new(2000),
            max_payload_size: AtomicUsize::new(10_000_000),
        }
    }

    pub fn get_ping_interval(&self) -> Duration {
        Duration::from_millis(self.ping_interval.load(Ordering::Relaxed))
    }

    pub fn get_ping_timeout(&self) -> Duration {
        Duration::from_millis(self.ping_timeout.load(Ordering::Relaxed))
    }

    pub fn get_max_payload_size(&self) -> usize {
        self.max_payload_size.load(Ordering::Relaxed)
    }

    pub fn set_ping_interval(&self, value: Duration) {
        self.ping_interval
            .store(value.as_millis() as u64, Ordering::SeqCst);
    }

    pub fn set_ping_timeout(&self, value: Duration) {
        self.ping_timeout
            .store(value.as_millis() as u64, Ordering::SeqCst);
    }

    pub fn set_max_payload_size(&self, value: usize) {
        self.max_payload_size.store(value, Ordering::SeqCst);
    }
}
