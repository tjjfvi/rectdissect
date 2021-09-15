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
  divs.insert(
    hash_division(&Division::default(), None),
    Division::default(),
  );
  for i in 2..=5 {
    edge_labelings.clear();
    let start2 = Instant::now();
    std::mem::replace(&mut divs, CHashMap::new())
      .into_iter()
      .flat_map(|(_, div)| {
        let div_box = Box::new(div);
        let div = unsafe { ignore_lifetime(&*div_box) };
        divide(div).chain(
          std::iter::once_with(move || {
            drop(div_box);
            None
          })
          .flatten(),
        )
      })
      .par_bridge()
      .for_each(|new_div| {
        let hash = hash_division(&new_div, None);
        if !divs.contains_key(&hash) {
          let mut any = false;
          for edge_labels in label_edges(&new_div) {
            edge_labelings.insert(hash_division(&new_div, Some(&edge_labels)), ());
            any = true;
          }
          if any {
            divs.insert(hash, new_div);
          }
        }
      });
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
}

unsafe fn ignore_lifetime<T>(ptr: &'_ T) -> &'static T {
  std::mem::transmute(ptr)
}
