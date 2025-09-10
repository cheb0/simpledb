pub trait Replacer {
    fn evict(&mut self) -> Option<usize>;

    fn record_access(&mut self, idx: usize);

    fn set_evictable(&mut self, idx: usize, set_evictable: bool);
}

pub struct DumbReplacer {
    evictable: Box<[bool]>,
}

impl DumbReplacer {
    pub fn new(buf_count: usize) -> DumbReplacer {
        Self { 
            evictable: vec![true; buf_count].into_boxed_slice()
        }
    }
}

impl Replacer for DumbReplacer {
    fn evict(&mut self) -> Option<usize> {
        for (i, evictable) in self.evictable.iter().enumerate() {
            if *evictable {
                return Some(i);
            }
        }
        None
    }

    fn record_access(&mut self, idx: usize) {

    }

    fn set_evictable(&mut self, idx: usize, evictable: bool) {
        self.evictable[idx] = evictable
    }
}