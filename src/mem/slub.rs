use crate::mem::linked_list::{LinkedList, RawLinkedList};

pub struct SlubPage {
    free_list: LinkedList,
    inuse: usize,
    objects: usize,
    page_start: *mut u8,
}

pub struct Cache {
    page_size: usize,
    objects_size: usize,
    free_slubs: RawLinkedList<SlubPage>,
    partal_slubs: RawLinkedList<SlubPage>,
    full_slubs: RawLinkedList<SlubPage>,
}
