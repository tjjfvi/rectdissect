use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct PairHashMap<K, V>(HashMap<K, HashMap<K, V>>, usize);

impl<K: Hash + Eq + Clone, V: Clone> PairHashMap<K, V> {
  pub fn new() -> PairHashMap<K, V> {
    PairHashMap(HashMap::new(), 0)
  }
  pub fn add(&mut self, key_a: K, key_b: K, value: V) -> Option<V> {
    self.1 += self
      .0
      .entry(key_a.clone())
      .or_insert_with(<_>::default)
      .insert(key_b.clone(), value.clone())
      .is_none() as usize;
    self
      .0
      .entry(key_b)
      .or_insert_with(<_>::default)
      .insert(key_a, value)
  }
  pub fn remove(&mut self, key_a: &K, key_b: &K) -> Option<V> {
    self.1 -= self
      .0
      .get_mut(key_a)
      .and_then(|x| x.remove(key_b))
      .is_some() as usize;
    self.0.get_mut(key_b).and_then(|x| x.remove(key_a))
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
  pub fn len(&self) -> usize {
    self.1
  }
  pub fn iter(&self) -> std::collections::hash_map::Iter<K, HashMap<K, V>> {
    self.0.iter()
  }
}

impl<K: Hash + Eq + Clone, V: Clone> Default for PairHashMap<K, V> {
  fn default() -> Self {
    Self::new()
  }
}
