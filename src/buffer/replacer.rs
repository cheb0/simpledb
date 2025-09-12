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

    fn record_access(&mut self, _idx: usize) {
        // DumbReplacer doesn't track access patterns
    }

    fn set_evictable(&mut self, idx: usize, evictable: bool) {
        self.evictable[idx] = evictable
    }
}

pub struct LruNode {
    prev: Option<usize>,
    next: Option<usize>,
    evictable: bool,
}

impl LruNode {
    pub fn new(prev: Option<usize>, next: Option<usize>) -> Self {
        Self {
            prev,
            next,
            evictable: true
        }
    }
}

pub struct LruReplacer {
    nodes: Box<[LruNode]>,
    head: usize,
    tail: usize,
}

// Structure:
// head -> 0 -> 1 -> 2 -> 3 <- tail (least recently used)
impl LruReplacer {
    pub fn new(size: usize) -> Self {
        if size <= 1 {
            panic!("Unsupportable size for LruReplacer {size}");
        }
        let mut nodes = Vec::with_capacity(size);
        for idx in 0..size {
            let prev = if idx == 0 {
                None
            } else {
                Some(idx - 1)
            };
            let next = if idx == size - 1 {
                None
            } else {
                Some(idx + 1)
            };
            nodes.push(LruNode::new(prev, next));
        }
        Self {
            nodes: nodes.into_boxed_slice(),
            head: 0,
            tail: size - 1,
        }
    }

    fn move_to_head(&mut self, idx: usize) {
        if self.head == idx {
            // already at head
            return;
        }

        let node = &self.nodes[idx];
        let prev = node.prev.unwrap();
        let next = node.next;

        self.nodes[prev].next = next;

        if let Some(next_idx) = next {
            self.nodes[next_idx].prev = Some(prev);
        } else {
            // This was the tail node
            self.tail = prev;
        }

        self.nodes[self.head].prev = Some(idx);
        self.nodes[idx].prev = None;
        self.nodes[idx].next = Some(self.head);
        self.head = idx;
    }
}

impl Replacer for LruReplacer {

    // Returns least recently used
    fn evict(&mut self) -> Option<usize> {
        // Start from the tail (least recently used) and find the first evictable node
        let mut current = Some(self.tail);
        
        while let Some(idx) = current {
            if self.nodes[idx].evictable {
                return Some(idx);
            }
            current = self.nodes[idx].prev;
        }
        None
    }

    fn record_access(&mut self, idx: usize) {
        self.move_to_head(idx);
    }

    fn set_evictable(&mut self, idx: usize, set_evictable: bool) {
        self.nodes[idx].evictable = set_evictable;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_replacer_basic() {
        let mut replacer = LruReplacer::new(3);
        
        replacer.record_access(0);
        replacer.record_access(1);
        replacer.record_access(2);

        assert_eq!(replacer.evict(), Some(0));

        replacer.record_access(0);
        replacer.record_access(1);

        assert_eq!(replacer.evict(), Some(2));

        replacer.record_access(0);

        assert_eq!(replacer.evict(), Some(2));
    }

    #[test]
    fn test_lru_replacer_basic_evictable_set() {
        let mut replacer = LruReplacer::new(3);
        
        assert_eq!(replacer.evict(), Some(2));
        
        replacer.set_evictable(2, false);
        assert_eq!(replacer.evict(), Some(1));
        
        replacer.set_evictable(1, false);
        assert_eq!(replacer.evict(), Some(0));
        
        replacer.set_evictable(0, false);
        assert_eq!(replacer.evict(), None);

        replacer.set_evictable(1, true);
        assert_eq!(replacer.evict(), Some(1));
    }

    #[test]
    fn test_lru_replacer_evict_and_record_access() {
        let mut replacer = LruReplacer::new(3);
        
        replacer.record_access(0);
        replacer.record_access(1);
        replacer.record_access(2);

        assert_eq!(replacer.evict(), Some(0));

        replacer.record_access(0);
        replacer.record_access(1);

        assert_eq!(replacer.evict(), Some(2));

        replacer.set_evictable(2, false);

        assert_eq!(replacer.evict(), Some(0));

        replacer.set_evictable(1, false);

        assert_eq!(replacer.evict(), Some(0));
    }
}