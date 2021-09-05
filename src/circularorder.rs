use std::{fmt::Debug, iter::FusedIterator};

#[derive(Clone)]
pub struct CircularOrder<T>(Vec<T>);

impl<T: Eq> CircularOrder<T> {
  pub fn new(vec: Vec<T>) -> CircularOrder<T> {
    debug_assert!(vec.len() != 0);
    CircularOrder(vec)
  }
  fn index(&self, item: &T) -> usize {
    self.0.iter().position(|x| x == item).unwrap()
  }
  pub fn get_item_after(&self, item: &T) -> &T {
    &self.0[(self.index(item) + 1) % self.len()]
  }
  pub fn get_item_before(&self, item: &T) -> &T {
    &self.0[(self.index(item) + self.len() - 1) % self.len()]
  }
  pub fn get_items_around(&self, item: &T) -> (&T, &T) {
    let i = self.index(item);
    (
      &self.0[(i + 1) % self.len()],
      &self.0[(i + self.len() - 1) % self.len()],
    )
  }
  pub fn insert_items_after(&mut self, item: &T, iter: impl IntoIterator<Item = T>) {
    let i = self.index(item) + 1;
    self.0.splice(i..i, iter);
  }
  pub fn delete_items_between(&mut self, start: &T, end: &T) {
    let start = self.index(start);
    let end = self.index(end);
    if start + 1 > end {
      self.0.splice(start + 1.., []);
      self.0.splice(..end, []);
    } else {
      self.0.splice(start + 1..end, []);
    }
  }
  pub fn len(&self) -> usize {
    self.0.len()
  }
  pub fn iter(&self) -> Iter<'_, T> {
    Iter(self, Some((0, 0)))
  }
  pub fn iter_starting_at<'a>(&'a self, start: &'a T) -> Iter<'a, T> {
    let i = self.index(start);
    Iter(self, Some((i, i)))
  }
}

impl<T: Eq + Debug> Debug for CircularOrder<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

pub struct Iter<'a, T>(&'a CircularOrder<T>, Option<(usize, usize)>);

impl<'a, T: Eq> FusedIterator for Iter<'a, T> {}

impl<'a, T: Eq> Iterator for Iter<'a, T> {
  type Item = &'a T;
  fn next(&mut self) -> Option<Self::Item> {
    if let Some((cur, end)) = self.1 {
      let next = (cur + 1) % self.0.len();
      self.1 = if next == end { None } else { Some((next, end)) };
      Some(&self.0 .0[cur])
    } else {
      None
    }
  }
}

impl<'a, T: Eq> DoubleEndedIterator for Iter<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if let Some((start, cur)) = self.1 {
      let next = (cur + self.0.len() - 1) % self.0.len();
      self.1 = if next == start {
        None
      } else {
        Some((start, next))
      };
      Some(&self.0 .0[cur])
    } else {
      None
    }
  }
}
