mod circularorder;
mod divide;
mod division;
mod generate_layout;
mod hash_division;
mod label_edges;
mod unorderedpair;

pub(crate) use circularorder::*;
pub(crate) use divide::*;
pub(crate) use division::*;
pub(crate) use generate_layout::*;
pub(crate) use hash_division::*;
pub(crate) use label_edges::*;
pub(crate) use unorderedpair::*;

use std::{
  collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
  fmt::{Debug, Write},
  hash::{Hash, Hasher},
};

fn main() {
  let mut set = HashSet::new();
  let mut new_set = HashSet::new();
  set.insert(Division::default());
  for _ in 1..4 {
    println!("{}", set.len());
    for div in set.drain() {
      divide(div, |new_div| {
        if label_edges(&new_div).is_some() {
          new_set.insert(new_div);
        }
      })
    }
    std::mem::swap(&mut set, &mut new_set);
  }
  println!("{}", set.len());
  for div in set {
    let layout = generate_layout(&div, &label_edges(&div).unwrap());
    let mut str = r#"<svg viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">"#.to_string();
    for rect in layout {
      write!(
        str,
        r#"<rect x="{}" width="{}" y="{}" height="{}" stroke="black" stroke-width="2" fill="none"/>"#,
        rect[0] * 100. + 10.,
        (rect[2] - rect[0]) * 100.,
        (rect[1]) * 100. + 10.,
        (rect[3] - rect[1]) * 100.
      )
      .unwrap();
    }
    str += "</svg>";
    println!("{}", str);
  }
}
