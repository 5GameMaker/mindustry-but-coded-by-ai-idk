const CAPACITY_RATIO_LOW: f32 = 1.5;
const CAPACITY_RATIO_HIGH: f32 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct PathfindQueue {
    pub queue: Vec<i32>,
    pub weights: Vec<f32>,
    pub size: usize,
}

impl PathfindQueue {
    pub fn new() -> Self {
        Self::with_capacity(12)
    }

    pub fn with_capacity(initial_capacity: usize) -> Self {
        Self {
            queue: vec![0; initial_capacity],
            weights: vec![0.0; initial_capacity],
            size: 0,
        }
    }

    pub fn empty(&self) -> bool {
        self.size == 0
    }

    pub fn add(&mut self, value: i32, weight: f32) -> bool {
        let index = self.size;
        if index >= self.queue.len() {
            self.grow_to_size(index + 1);
        }
        self.size = index + 1;
        if index == 0 {
            self.queue[0] = value;
            self.weights[0] = weight;
        } else {
            self.sift_up(index, value, weight);
        }
        true
    }

    pub fn peek(&self) -> i32 {
        if self.size == 0 {
            0
        } else {
            self.queue[0]
        }
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }

    pub fn poll(&mut self) -> i32 {
        if self.size == 0 {
            return 0;
        }
        self.size -= 1;
        let result = self.queue[0];
        let x = self.queue[self.size];
        let weight = self.weights[self.size];
        if self.size != 0 {
            self.sift_down(0, x, weight);
        }
        result
    }

    pub fn capacity(&self) -> usize {
        self.queue.len()
    }

    fn sift_up(&mut self, mut k: usize, value: i32, weight: f32) {
        while k > 0 {
            let parent = (k - 1) >> 1;
            let existing = self.queue[parent];
            if weight >= self.weights[parent] {
                break;
            }
            self.queue[k] = existing;
            self.weights[k] = self.weights[parent];
            k = parent;
        }
        self.queue[k] = value;
        self.weights[k] = weight;
    }

    fn sift_down(&mut self, mut k: usize, value: i32, weight: f32) {
        let half = self.size >> 1;
        while k < half {
            let mut child = (k << 1) + 1;
            let mut candidate = self.queue[child];
            let right = child + 1;
            if right < self.size && self.weights[child] > self.weights[right] {
                child = right;
                candidate = self.queue[child];
            }
            if weight <= self.weights[child] {
                break;
            }
            self.queue[k] = candidate;
            self.weights[k] = self.weights[child];
            k = child;
        }
        self.queue[k] = value;
        self.weights[k] = weight;
    }

    fn grow_to_size(&mut self, min_capacity: usize) {
        let old = self.queue.len();
        let ratio = if old < 64 {
            CAPACITY_RATIO_HIGH
        } else {
            CAPACITY_RATIO_LOW
        };
        let mut new_capacity = if old < 64 {
            ((old + 1) as f32 * ratio) as usize
        } else {
            (old as f32 * ratio) as usize
        };
        if new_capacity < min_capacity {
            new_capacity = min_capacity;
        }
        self.queue.resize(new_capacity, 0);
        self.weights.resize(new_capacity, 0.0);
    }
}

impl Default for PathfindQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_add_peek_and_poll_return_lowest_weight_first() {
        let mut queue = PathfindQueue::new();

        assert!(queue.empty());
        queue.add(10, 5.0);
        queue.add(20, 3.0);
        queue.add(30, 7.0);
        queue.add(40, 1.0);

        assert_eq!(queue.peek(), 40);
        assert_eq!(queue.poll(), 40);
        assert_eq!(queue.poll(), 20);
        assert_eq!(queue.poll(), 10);
        assert_eq!(queue.poll(), 30);
        assert_eq!(queue.poll(), 0);
        assert!(queue.empty());
    }

    #[test]
    fn queue_clear_keeps_capacity_and_resets_size() {
        let mut queue = PathfindQueue::with_capacity(2);
        queue.add(1, 2.0);
        queue.add(2, 1.0);
        let capacity = queue.capacity();

        queue.clear();

        assert_eq!(queue.size, 0);
        assert_eq!(queue.peek(), 0);
        assert_eq!(queue.capacity(), capacity);
    }

    #[test]
    fn queue_grows_like_java_capacity_formula() {
        let mut queue = PathfindQueue::with_capacity(1);

        queue.add(1, 1.0);
        queue.add(2, 2.0);

        assert_eq!(queue.capacity(), 4);
        assert_eq!(queue.size, 2);
    }
}
