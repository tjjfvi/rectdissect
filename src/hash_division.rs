use crate::*;
use either::Either;
use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

pub fn hash_division(div: &Division, edge_labels: Option<&EdgeLabels>) -> u64 {
  let mut hash = u64::MAX;
  for start in 0..4 {
    for dir in [true, false] {
      let mut hasher = DefaultHasher::new();
      let mut node_ids = Vec::with_capacity(div.nodes().len());
      use_helper_fn!(visit_node(
        div,
        edge_labels,
        start,
        dir,
        &mut hasher,
        &mut node_ids,
      ));
      visit_node!(
        Node::border(start),
        Node::border(start + if dir { 3 } else { 1 }),
      );
      let new_hash = hasher.finish();
      if new_hash < hash {
        hash = new_hash
      }
    }
  }
  return hash;
}

#[helper_fn(
  div: &Division,
  edge_labels: Option<&EdgeLabels>,
  start: u8,
  dir: bool,
  &mut hasher: DefaultHasher,
  &mut node_ids: Vec<Node>,
)]
fn visit_node(node: Node, last: Node) {
  let mut fresh = false;
  let id = node_ids.iter().position(|x| x == &node).unwrap_or_else(|| {
    let id = node_ids.len();
    fresh = true;
    node_ids.push(node);
    id
  });
  hasher.write_usize(id);
  if let Some(x) = edge_labels {
    x.get(&UnorderedPair(node, last))
      .map(|&x| x == (start % 2 == 0))
      .hash(hasher);
  }
  if fresh {
    let connected_nodes = &div[node];
    hasher.write_u8(connected_nodes.len());
    for next in maybe_reverse(connected_nodes.iter_starting_at(last).skip(1), dir) {
      visit_node!(next, node);
    }
  }
  fn maybe_reverse<T, I: DoubleEndedIterator<Item = T>>(
    iter: I,
    reverse: bool,
  ) -> Either<I, std::iter::Rev<I>> {
    if reverse {
      Either::Left(iter)
    } else {
      Either::Right(iter.rev())
    }
  }
}
