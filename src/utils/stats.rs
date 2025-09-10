use std::sync::atomic::{AtomicI64, Ordering};

pub struct Counter {
    value: AtomicI64,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            value: AtomicI64::new(0),
        }
    }

    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Stats {
    pub storage_mgr_stats: StorageMgrStats
}

impl Stats {
    pub fn new() -> Self {
        Self {
            storage_mgr_stats: StorageMgrStats::new(),
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StorageMgrStats {
    pub pages_written: Counter,
    pub pages_read: Counter,
}

impl StorageMgrStats {
    pub fn new() -> StorageMgrStats {
        Self {
            pages_written: Counter::new(),
            pages_read: Counter::new(),
        }
    }
}

impl Default for StorageMgrStats {
    fn default() -> Self {
        Self::new()
    }
}