use std::{collections::HashMap, fmt::Debug, hash::Hash, iter::FusedIterator};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MissingItemError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InsertItemsError {
  MissingItem,
  ItemAlreadyExists,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeleteItemsError {
  MissingItem,
  InvalidCircularOrder,
}

#[derive(Clone)]
pub struct CircularOrder<T>(HashMap<T, (T, T)>);

impl<T: Hash + Eq + Clone> CircularOrder<T> {
  pub fn new(iter: impl IntoIterator<Item = T>) -> CircularOrder<T> {
    let mut map = HashMap::new();
    let mut iter = iter.into_iter().peekable();
    let first = iter.next().unwrap();
    let second = iter.peek().unwrap_or(&first).clone();
    let mut last = first.clone();
    while let Some(item) = iter.next() {
      assert!(map
        .insert(item.clone(), (last, iter.peek().unwrap_or(&first).clone()))
        .is_none());
      last = item;
    }
    assert!(map.insert(first, (last, second)).is_none());
    CircularOrder(map)
  }
  pub fn get_item_after(&self, item: &T) -> Result<&T, MissingItemError> {
    Ok(&self.0.get(item).ok_or(MissingItemError)?.1)
  }
  pub fn get_item_before(&self, item: &T) -> Result<&T, MissingItemError> {
    Ok(&self.0.get(item).ok_or(MissingItemError)?.0)
  }
  pub fn get_items_around(&self, item: &T) -> Result<&(T, T), MissingItemError> {
    self.0.get(item).ok_or(MissingItemError)
  }
  pub fn insert_items_after(
    &mut self,
    item: &T,
    iter: impl IntoIterator<Item = T>,
  ) -> Result<(), InsertItemsError> {
    let mut iter = iter.into_iter().peekable();
    let end = std::mem::replace(
      &mut self.0.get_mut(item).ok_or(InsertItemsError::MissingItem)?.1,
      match iter.peek() {
        Some(x) => x.clone(),
        None => return Ok(()),
      },
    );
    let mut last = item.clone();
    while let Some(item) = iter.next() {
      if self
        .0
        .insert(item.clone(), (last, iter.peek().unwrap_or(&end).clone()))
        .is_some()
      {
        return Err(InsertItemsError::ItemAlreadyExists);
      }
      last = item;
    }
    self.0.get_mut(&end).unwrap().0 = last;
    Ok(())
  }
  pub fn delete_items_between(&mut self, start: &T, end: &T) -> Result<(), DeleteItemsError> {
    let delete_start = std::mem::replace(
      &mut self
        .0
        .get_mut(start)
        .ok_or(DeleteItemsError::MissingItem)?
        .1,
      end.clone(),
    );
    self.0.get_mut(end).ok_or(DeleteItemsError::MissingItem)?.0 = start.clone();
    let mut cur_delete = delete_start;
    while cur_delete != *end {
      cur_delete = self
        .0
        .remove(&cur_delete)
        .ok_or(DeleteItemsError::InvalidCircularOrder)?
        .1;
    }
    Ok(())
  }
  pub fn reverse(&mut self) {
    for (a, b) in self.0.values_mut() {
      std::mem::swap(a, b)
    }
  }
  pub fn contains_item(&self, item: &T) -> bool {
    self.0.contains_key(item)
  }
  pub fn len(&self) -> usize {
    self.0.len()
  }
  pub fn iter(&self) -> Iter<'_, T> {
    let start = self.0.keys().next().unwrap();
    Iter(self, Some((start, start)))
  }
  pub fn iter_starting_at<'a>(&'a self, start: &'a T) -> Iter<'a, T> {
    Iter(self, Some((start, start)))
  }
  pub fn iter_between<'a>(&'a self, start: &'a T, end: &'a T) -> Iter<'a, T> {
    Iter(self, Some((start, end)))
  }
}

impl<T: Hash + Eq + Debug> Debug for CircularOrder<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug_list = f.debug_list();
    let start = self.0.keys().next().ok_or(std::fmt::Error)?;
    let mut cur = start;
    loop {
      debug_list.entry(cur);
      cur = &self.0.get(cur).ok_or(std::fmt::Error)?.1;
      if cur == start {
        break;
      }
    }
    debug_list.finish()
  }
}

pub struct Iter<'a, T>(&'a CircularOrder<T>, Option<(&'a T, &'a T)>);

impl<'a, T: Hash + Eq> FusedIterator for Iter<'a, T> {}

impl<'a, T: Hash + Eq> Iterator for Iter<'a, T> {
  type Item = &'a T;
  fn next(&mut self) -> Option<Self::Item> {
    if let Some((cur, end)) = self.1 {
      let next = &self.0 .0.get(cur).unwrap().1;
      self.1 = if next == end { None } else { Some((next, end)) };
      Some(cur)
    } else {
      None
    }
  }
}

impl<'a, T: Hash + Eq> DoubleEndedIterator for Iter<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if let Some((start, cur)) = self.1 {
      let next = &self.0 .0.get(cur).unwrap().0;
      self.1 = if next == start {
        None
      } else {
        Some((start, next))
      };
      Some(cur)
    } else {
      None
    }
  }
}
