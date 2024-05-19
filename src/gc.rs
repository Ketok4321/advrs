use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
use std::collections::HashSet;

use crate::interpreter::*;

macro_rules! ptr_len {
    ($ptr:expr) => {
        ptr::NonNull::new_unchecked($ptr as *mut [Object]).len()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct GC {
    allocations: HashSet<*mut [Object]>,
    zero_alloc_index: usize,
    stack: *const [Object],
}

impl GC {
    pub fn new(stack: *const [Object], heap_size: usize) -> Self {
        Self {
            allocations: HashSet::with_capacity(heap_size),
            zero_alloc_index: 0,
            stack,
        }
    }

    pub fn alloc(&mut self, size: usize) -> *mut [Object] {
        if size == 0 {
            let result = ptr::slice_from_raw_parts(self.zero_alloc_index as *mut Object, 0) as *mut [Object];
            self.zero_alloc_index += 1;
            result
        } else {
            unsafe {
                let layout = Layout::array::<Object>(size).expect("Invalid layout :<");
                let allocated = ptr::slice_from_raw_parts(alloc(layout), size) as *mut [Object];
                
                if self.allocations.capacity() == self.allocations.len() {
                    self.collect();
                }
                self.allocations.insert(allocated);

                allocated
            }
        }
    }

    pub fn collect(&mut self) {
        unsafe {
            let mut keep_alive = HashSet::with_capacity(self.allocations.capacity());

            fn add(keep_alive: &mut HashSet<*mut [Object]>, obj: &Object) {
                unsafe {
                    if keep_alive.insert(obj.contents) {
                        for i in 0..ptr_len!(obj.contents) {
                            add(keep_alive, &(*obj.contents)[i]);
                        }
                    }
                }
            }

            for i in 0..ptr_len!(self.stack) {
                let fella = (*self.stack)[i];
                if fella == Object::TRUE_NULL {
                    break;
                }
                add(&mut keep_alive, &fella);
            }

            for garbage in &self.allocations - &keep_alive {
                dealloc(garbage as *mut u8, Layout::array::<Object>(ptr_len!(garbage)).expect("Invalid layout :<"));
            }

            self.allocations = keep_alive;
        }
    }
}
