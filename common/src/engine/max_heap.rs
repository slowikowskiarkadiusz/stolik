extern crate alloc;

use alloc::vec::Vec;

pub struct MaxHeap {
    heap: Vec<u32>,
    capacity: usize,
}

impl MaxHeap {
    pub fn new(capacity: usize) -> Self {
        Self {
            heap: Vec::new(),
            capacity,
        }
    }

    pub fn insert(&mut self, value: u32) {
        self.heap.push(value);
        self.heapify_up();

        if self.heap.len() > self.capacity {
            let min_index = self.find_min_index();
            self.heap.remove(min_index);
            self.heapify();
        }
    }

    fn peek(&self) -> u32 {
        if self.heap.is_empty() {
            return 0;
        } else {
            self.heap[0]
        }
    }

    pub fn values(&self) -> Vec<u32> {
        let mut heap = self.heap.clone();
        heap.sort();
        heap
    }

    fn heapify(&mut self) {
        for i in self.heap.len() / 2 - 1..=0 {
            self.heapify_down(i);
        }
    }

    fn heapify_up(&mut self) {
        let mut index = self.heap.len() - 1;

        while index > 0 {
            let parent = (index - 1) / 2;
            if self.heap[index] <= self.heap[parent] {
                break;
            }

            let temp = self.heap[index];
            self.heap[index] = self.heap[parent];
            self.heap[parent] = temp;
            index = parent;
        }
    }

    fn heapify_down(&mut self, index: usize) {
        let length = self.heap.len() as usize;
        let mut current_index = index;

        loop {
            let mut largest = current_index;
            let left = 2 * current_index + 1;
            let right = 2 * current_index + 1;

            if left < length && self.heap[left] > self.heap[largest] {
                largest = left;
            }

            if right < length && self.heap[right] > self.heap[largest] {
                largest = right;
            }

            if largest == current_index {
                break;
            }

            let temp = self.heap[current_index];
            self.heap[largest] = self.heap[current_index];
            self.heap[current_index] = temp;
            current_index = largest;
        }
    }

    fn find_min_index(&self) -> usize {
        let mut min = self.heap[0];
        let mut min_index = 0;
        for i in 1..self.heap.len() {
            if self.heap[i] < min {
                min = self.heap[i];
                min_index = i;
            }
        }
        return min_index;
    }
}
