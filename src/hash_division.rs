use crate::*;
use either::Either;
use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

pub fn hash_division(div: &Division, edge_labels: Option<&EdgeLabels>) -> u64 {
  struct State<'a> {
    div: &'a Division,
    edge_labels: Option<&'a EdgeLabels>,
    start: u8,
    dir: bool,
    hasher: DefaultHasher,
    node_ids: Vec<Node>,
  }
  let mut hash = u64::MAX;
  for start in 0..4 {
    for dir in [true, false] {
      let mut state = State {
        div,
        edge_labels,
        start,
        dir,
        hasher: DefaultHasher::new(),
        node_ids: Vec::with_capacity((div.regions() + 4) as usize),
      };
      visit_node(
        &mut state,
        Node::border(start),
        Node::border(start + if dir { 3 } else { 1 }),
      );
      let new_hash = state.hasher.finish();
      if new_hash < hash {
        hash = new_hash
      }
    }
  }
  return hash;
  fn visit_node(state: &mut State<'_>, node: Node, last: Node) {
    let mut fresh = false;
    let id = state
      .node_ids
      .iter()
      .position(|x| x == &node)
      .unwrap_or_else(|| {
        let id = state.node_ids.len();
        fresh = true;
        state.node_ids.push(node);
        id
      });
    state.hasher.write_usize(id);
    if let Some(x) = state.edge_labels {
      x.get(&UnorderedPair(node, last))
        .map(|&x| x == (state.start % 2 == 0))
        .hash(&mut state.hasher);
    }
    if fresh {
      let connected_nodes = &state.div[node];
      state.hasher.write_u8(connected_nodes.len());
      for next in maybe_reverse(connected_nodes.iter_starting_at(last).skip(1), state.dir) {
        visit_node(state, next, node);
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
}
