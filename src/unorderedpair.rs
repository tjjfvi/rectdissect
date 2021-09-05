use std::{
  fmt::Debug,
  hash::{Hash, Hasher},
};

#[derive(Default, Clone, Copy)]
pub struct UnorderedPair<T>(pub T, pub T);

impl<T: Ord> From<UnorderedPair<T>> for (T, T) {
  fn from(pair: UnorderedPair<T>) -> Self {
    if pair.0 < pair.1 {
      (pair.0, pair.1)
    } else {
      (pair.1, pair.0)
    }
  }
}

impl<'a, T: Ord> From<&'a UnorderedPair<T>> for (&'a T, &'a T) {
  fn from(pair: &'a UnorderedPair<T>) -> Self {
    if pair.0 < pair.1 {
      (&pair.0, &pair.1)
    } else {
      (&pair.1, &pair.0)
    }
  }
}

impl<T: Ord + Hash> Hash for UnorderedPair<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    <_ as Into<(_, _)>>::into(self).hash(state);
  }
}

impl<T: Ord + PartialEq> PartialEq for UnorderedPair<T> {
  fn eq(&self, other: &Self) -> bool {
    <_ as Into<(_, _)>>::into(self) == other.into()
  }
}

impl<T: Ord + Eq> Eq for UnorderedPair<T> {}

impl<T: Debug> Debug for UnorderedPair<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}-{:?}", self.0, self.1)
  }
}
