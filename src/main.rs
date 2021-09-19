mod connected_nodes;
mod divide;
mod division;
mod generate_layout;
mod hash_division;
mod label_edges;
mod node;
mod svg;
mod unorderedpair;

pub(crate) use connected_nodes::*;
pub(crate) use divide::*;
pub(crate) use division::*;
pub(crate) use generate_layout::*;
pub(crate) use hash_division::*;
pub(crate) use label_edges::*;
pub(crate) use node::*;
pub(crate) use svg::*;
pub(crate) use unorderedpair::*;

use chashmap::CHashMap;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{fmt::Debug, time::Instant};

fn main() {
  let start = Instant::now();

  let mut divs = CHashMap::new();
  let edge_labelings = CHashMap::new();

  add_div(Division::default(), &divs, &edge_labelings);

  for i in 2..=9 {
    edge_labelings.clear();
    let start2 = Instant::now();
    std::mem::replace(&mut divs, CHashMap::new())
      .into_iter()
      .flat_map(|(_, div)| iter_with_owned(div, divide))
      .par_bridge()
      .for_each(|div| add_div(div, &divs, &edge_labelings));
    eprintln!(
      "{}: {}/{} (took {:.1?}, total {:.1?})",
      i,
      divs.len(),
      edge_labelings.len(),
      start2.elapsed(),
      start.elapsed()
    );
  }

  println!("{}", generate_svg(divs, edge_labelings));

  fn add_div(div: Division, divs: &CHashMap<u64, Division>, edge_labelings: &CHashMap<u64, ()>) {
    let hash = hash_division(&div, None);
    if !divs.contains_key(&hash) {
      let mut any = false;
      for edge_labels in label_edges(&div) {
        edge_labelings.insert(hash_division(&div, Some(&edge_labels)), ());
        any = true;
      }
      if any {
        divs.insert(hash, div);
      }
    }
  }
}

fn iter_with_owned<'a, T: 'a, I: Iterator>(
  value: T,
  cb: impl Fn(&'a T) -> I,
) -> WithOwnedIter<T, I> {
  let boxed = Box::new(value);
  WithOwnedIter(Some((cb(unsafe { ignore_lifetime(&*boxed) }), boxed)))
}

struct WithOwnedIter<T, I>(Option<(I, Box<T>)>);

impl<T, I> Iterator for WithOwnedIter<T, I>
where
  I: Iterator,
{
  type Item = <I as Iterator>::Item;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some((iter, _)) = &mut self.0 {
      iter.next().or_else(|| {
        self.0.take();
        None
      })
    } else {
      None
    }
  }
}

unsafe fn ignore_lifetime<'a, T: 'a>(ptr: &'_ T) -> &'a T {
  std::mem::transmute(ptr)
}
