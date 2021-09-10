use crate::*;
use std::{iter::FusedIterator, ptr};

pub struct ConnectedNodes(u8);

impl ConnectedNodes {
  fn get_index(&self, item: Node) -> u8 {
    for i in 0..self.len() {
      if self.index(i) == item {
        return i;
      }
    }
    dbg!(item, self.len(), self);
    panic!("item not in CircularOrder");
  }
  fn index(&self, index: u8) -> Node {
    assert!(index < self.len());
    unsafe { *((self as *const _ as *const Node).offset((index + 1) as isize)) }
  }
  fn index_mut(&mut self, index: u8) -> &mut Node {
    assert!(index < self.len());
    unsafe { &mut *((self as *mut _ as *mut Node).offset((index + 1) as isize)) }
  }
  pub fn get_item_after(&self, item: Node) -> Node {
    self.index((self.get_index(item) + 1) % self.len())
  }
  pub fn get_item_before(&self, item: Node) -> Node {
    self.index((self.get_index(item) + self.len() - 1) % self.len())
  }
  pub fn get_items_around(&self, item: Node) -> (Node, Node) {
    let i = self.get_index(item);
    (
      self.index((i + 1) % self.len()),
      self.index((i + self.len() - 1) % self.len()),
    )
  }
  pub unsafe fn insert_item_after(&mut self, item: Node, new_item: Node) {
    let i = self.get_index(item) + 1;
    let old_len = self.len();
    self.0 += 1;
    if i != old_len {
      ptr::copy(
        self.index_mut(i) as *const _,
        self.index_mut(i + 1) as *mut _,
        (old_len - i) as usize,
      );
    }
    *self.index_mut(i) = new_item;
  }
  pub fn replace_item(&mut self, old_item: Node, new_item: Node) {
    *self.index_mut(self.get_index(old_item)) = new_item;
  }
  pub fn delete_items_between(&mut self, start: Node, end: Node) {
    unsafe {
      let start = self.get_index(start);
      let end = self.get_index(end);
      if start == end {
        return;
      }
      if start > end {
        let (start, end) = (end, start);
        let new_len = end + 1 - start;
        ptr::copy(
          self.index_mut(start) as *const _,
          self.index_mut(0) as *mut _,
          new_len as usize,
        );
        self.0 = new_len;
      } else {
        let remove_count = end - start - 1;
        let tail_count = self.len() - end;
        ptr::copy(
          self.index_mut(end) as *const _,
          self.index_mut(start + 1) as *mut _,
          tail_count as usize,
        );
        self.0 -= remove_count;
      }
    }
  }
  pub fn len(&self) -> u8 {
    self.0
  }
  pub fn iter(&self) -> CircularOrderIter<'_> {
    CircularOrderIter(self, Some((0, 0)))
  }
  pub fn iter_starting_at(&self, start: Node) -> CircularOrderIter<'_> {
    let i = self.get_index(start);
    CircularOrderIter(self, Some((i, i)))
  }
  pub fn contains_item(&self, item: Node) -> bool {
    for i in 0..self.len() {
      if self.index(i) == item {
        return true;
      }
    }
    false
  }
}

impl Debug for ConnectedNodes {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug_list = f.debug_list();
    for i in 0..self.len() {
      debug_list.entry(&self.index(i));
    }
    debug_list.finish()
  }
}

pub struct CircularOrderIter<'a>(&'a ConnectedNodes, Option<(u8, u8)>);

impl<'a> FusedIterator for CircularOrderIter<'a> {}

impl<'a> Iterator for CircularOrderIter<'a> {
  type Item = Node;
  fn next(&mut self) -> Option<Self::Item> {
    if let Some((cur, end)) = self.1 {
      let next = (cur + 1) % self.0.len();
      self.1 = if next == end { None } else { Some((next, end)) };
      Some(self.0.index(cur))
    } else {
      None
    }
  }
}

impl<'a> DoubleEndedIterator for CircularOrderIter<'a> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if let Some((start, cur)) = self.1 {
      let next = (cur + self.0.len() - 1) % self.0.len();
      self.1 = if next == start {
        None
      } else {
        Some((start, next))
      };
      Some(self.0.index(cur))
    } else {
      None
    }
  }
}
