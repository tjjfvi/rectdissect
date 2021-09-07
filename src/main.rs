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
  collections::{
    hash_map::{DefaultHasher, Entry},
    HashMap, HashSet, VecDeque,
  },
  fmt::Debug,
  hash::{Hash, Hasher},
  time::Instant,
};

fn main() {
  let start = Instant::now();
  let mut divs = HashMap::new();
  let mut new_divs = HashMap::new();
  divs.insert(hash_division(&Division::default()), Division::default());
  for i in 2..=8 {
    for (_, div) in divs.drain() {
      for new_div in divide(&div) {
        let hash = hash_division(&new_div);
        let entry = new_divs.entry(hash);
        if let Entry::Vacant(entry) = entry {
          if label_edges(&new_div).is_some() {
            entry.insert(new_div);
          }
        }
      }
    }
    std::mem::swap(&mut divs, &mut new_divs);
    eprintln!("{}: {}", i, divs.len());
  }
  eprintln!("{:?}", start.elapsed());
  println!("{}", generate_svg(&divs));
}
