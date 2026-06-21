use core::marker::PhantomData;
use crate::print;

#[derive(Copy, Clone, Debug)]
pub struct LinkedList {
    next: *mut usize,
}

impl LinkedList {
    pub const fn new() -> Self {
        Self {
            next: core::ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.next.is_null()
    }

    pub fn push(&mut self, item: *mut usize) {
        unsafe { *item = self.next as usize };
        self.next = item;
    }

    pub fn pop(&mut self) -> Option<*mut usize> {
        match self.is_empty() {
            true => None,
            false => {
                let item = self.next;
                self.next = unsafe { *item as *mut usize };
                Some(item)
            }
        }
    }

    pub fn remove(&mut self, item: usize) -> bool {
        if self.next as usize == item {
            self.next = unsafe { *self.next as *mut usize };
            return true;
        }

        let mut current = self.next;
        while !current.is_null() {
            let next = unsafe { *current };
            if next == item {
                unsafe { *current = *(next as *mut usize) }
                return true;
            }
            current = unsafe { *current as *mut usize };
        }

        false
    }

    pub fn iter(&self) -> LinkedListIterator<'_> {
        LinkedListIterator {
            current: self.next,
            _marker: PhantomData,
        }
    }

    pub fn debug(&self) {
        for item in self.iter() {
            print!("{:#x} -> ", item as usize);
        }
    }
}

impl PartialEq for LinkedList {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next
    }
}

pub struct LinkedListIterator<'a> {
    current: *mut usize,
    _marker: PhantomData<&'a mut LinkedList>,
}

impl<'a> Iterator for LinkedListIterator<'a> {
    type Item = *mut usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let item = self.current;
            self.current = unsafe { *item as *mut usize };
            Some(item)
        }
    }
}