use std::collections::binary_heap::Iter;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct PairHashMap<K, V>(HashMap<K, HashMap<K, V>>);

impl<K: Hash + Eq + Clone, V: Clone> PairHashMap<K, V> {
  pub fn new() -> PairHashMap<K, V> {
    PairHashMap(HashMap::new())
  }
  pub fn add(&mut self, key_a: K, key_b: K, value: V) {
    self
      .0
      .entry(key_a.clone())
      .or_insert_with(<_>::default)
      .insert(key_b.clone(), value.clone());
    self
      .0
      .entry(key_b)
      .or_insert_with(<_>::default)
      .insert(key_a, value);
  }
  pub fn remove(&mut self, key_a: &K, key_b: &K) {
    self.0.get_mut(key_a).map(|x| x.remove(key_b));
    self.0.get_mut(key_b).map(|x| x.remove(key_a));
  }
  pub fn get_all(&self, key: &K) -> Option<&HashMap<K, V>> {
    self.0.get(key)
  }
  pub fn get(&self, key_a: &K, key_b: &K) -> Option<&V> {
    self.0.get(key_a).and_then(|x| x.get(key_b))
  }
  pub fn keys(&self) -> impl Iterator<Item = &K> {
    self.0.keys()
  }
}

impl<K: Hash + Eq + Clone, V: Clone> Default for PairHashMap<K, V> {
  fn default() -> Self {
    Self::new()
  }
}
