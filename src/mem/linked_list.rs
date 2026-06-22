use core::fmt;
use core::marker::PhantomData;

#[derive(Copy, Clone)]
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

    pub fn iter(&self) -> LinkedListIter<'_> {
        LinkedListIter {
            current: self.next,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> LinkedListIterMut<'_> {
        LinkedListIterMut {
            current: self.next,
            _marker: PhantomData,
        }
    }
}

impl fmt::Debug for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter().map(|p| p as usize)).finish()
    }
}

impl PartialEq for LinkedList {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next
    }
}

pub struct LinkedListIter<'a> {
    current: *mut usize,
    _marker: PhantomData<&'a mut LinkedList>,
}

impl<'a> Iterator for LinkedListIter<'a> {
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

pub struct LinkedListIterMut<'a> {
    current: *mut usize,
    _marker: PhantomData<&'a mut LinkedList>,
}

impl<'a> Iterator for LinkedListIterMut<'a> {
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

#[derive(Debug)]
pub struct RawLinkedListNode<T> {
    pub data: T,
    next: *mut RawLinkedListNode<T>,
}

impl<T> RawLinkedListNode<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            next: core::ptr::null_mut(),
        }
    }
}

pub struct RawLinkedList<T> {
    head: *mut RawLinkedListNode<T>,
}

impl<T> RawLinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: core::ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn push(&mut self, node: *mut RawLinkedListNode<T>) {
        unsafe {
            (*node).next = self.head;
        }
        self.head = node;
    }

    pub fn pop(&mut self) -> Option<*mut RawLinkedListNode<T>> {
        if self.is_empty() {
            None
        } else {
            let node = self.head;
            self.head = unsafe { (*node).next };
            Some(node)
        }
    }

    pub fn remove(&mut self, item: &T) -> bool
    where
        T: PartialEq,
    {
        if self.head.is_null() {
            return false;
        }

        if unsafe { &(*self.head).data } == item {
            self.head = unsafe { (*self.head).next };
            return true;
        }

        let mut current = self.head;
        while !current.is_null() {
            let next = unsafe { (*current).next };
            if !next.is_null() && unsafe { &(*next).data } == item {
                unsafe {
                    (*current).next = (*next).next;
                }
                return true;
            }
            current = unsafe { (*current).next };
        }

        false
    }

    pub fn iter(&self) -> RawLinkedListIter<'_, T> {
        RawLinkedListIter {
            current: self.head,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> RawLinkedListIterMut<'_, T> {
        RawLinkedListIterMut {
            current: self.head,
            _marker: PhantomData,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for RawLinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

pub struct RawLinkedListIter<'a, T> {
    current: *mut RawLinkedListNode<T>,
    _marker: PhantomData<&'a RawLinkedList<T>>,
}

impl<'a, T> Iterator for RawLinkedListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let node = unsafe { &*self.current };
            self.current = node.next;
            Some(&node.data)
        }
    }
}

pub struct RawLinkedListIterMut<'a, T> {
    current: *mut RawLinkedListNode<T>,
    _marker: PhantomData<&'a mut RawLinkedList<T>>,
}

impl<'a, T> Iterator for RawLinkedListIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let node = unsafe { &mut *self.current };
            self.current = node.next;
            Some(&mut node.data)
        }
    }
}
