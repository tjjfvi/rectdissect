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
use helper_fn::{helper_fn, use_helper_fn};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{fmt::Debug, time::Instant};

fn main() {
  let oeis_mode = false;

  let start = Instant::now();

  let mut divs = CHashMap::new();
  let oeis_count = CHashMap::new();

  use_helper_fn! {
    print_state(&divs, oeis_mode, &oeis_count),
    add_div(&divs, oeis_mode, &oeis_count),
  }

  add_div!(Division::default());
  print_state!(1, start, start);

  for i in 2..=5 {
    oeis_count.clear();
    let round_start = Instant::now();
    std::mem::replace(&mut divs, CHashMap::new())
      .into_iter()
      .flat_map(|(_, div)| iter_with_owned(div, divide))
      .par_bridge()
      .for_each(|div| add_div!(div));
    print_state!(i, start, round_start);
  }

  println!("{}", generate_svg(divs, oeis_mode, oeis_count));

  #[helper_fn(
    &divs: CHashMap<u64, Division>,
    oeis_mode: bool,
    &oeis_count: CHashMap<u64, ()>,
  )]
  fn print_state(i: u8, start: Instant, round_start: Instant) {
    let now = Instant::now();
    eprintln!(
      "{:>2}: {:<10}{} {:>10} {:>10}",
      i,
      divs.len(),
      if oeis_mode {
        format!(" {:<10}", oeis_count.len())
      } else {
        "".to_string()
      },
      format!("{:.1?}", now - round_start),
      format!("{:.1?}", now - start),
    );
  }

  #[helper_fn(
    &divs: CHashMap<u64, Division>,
    oeis_mode: bool,
    &oeis_count: CHashMap<u64, ()>,
  )]
  fn add_div(div: Division) {
    let hash = hash_division(&div, None);
    if !divs.contains_key(&hash) {
      let mut any = false;
      for edge_labels in label_edges(&div) {
        any = true;
        if oeis_mode {
          oeis_count.insert(hash_division(&div, Some(&edge_labels)), ());
        } else {
          break;
        }
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
