mod circularorder;
mod divide;
mod division;
mod generate_layout;
mod hash_division;
mod label_edges;
mod svg;
mod unorderedpair;

pub(crate) use circularorder::*;
pub(crate) use divide::*;
pub(crate) use division::*;
pub(crate) use generate_layout::*;
pub(crate) use hash_division::*;
pub(crate) use label_edges::*;
pub(crate) use svg::*;
pub(crate) use unorderedpair::*;

use std::{
  collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
  fmt::{Debug, Write},
  hash::{Hash, Hasher},
  time::Instant,
};

fn main() {
  let start = Instant::now();
  let mut set = HashSet::new();
  let mut new_set = HashSet::new();
  set.insert(Division::default());
  for i in 2..9 {
    println!("{}: {}", i, set.len());
    for div in set.drain() {
      for new_div in divide(div) {
        if label_edges(&new_div).is_some() {
          new_set.insert(new_div);
        }
      }
    }
    std::mem::swap(&mut set, &mut new_set);
  }
  println!("{:?}", start.elapsed());
}
