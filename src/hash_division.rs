use crate::*;
use either::Either;

pub fn hash_division(div: &Division) -> u64 {
  struct State<'a> {
    div: &'a Division,
    dir: bool,
    hasher: DefaultHasher,
    node_id_map: HashMap<Node, u32>,
    next_node_id: u32,
  }
  let mut hash = u64::MAX;
  for i in 0..4 {
    for dir in [true, false] {
      let mut state = State {
        div,
        dir,
        hasher: DefaultHasher::new(),
        node_id_map: HashMap::new(),
        next_node_id: 0,
      };
      visit_node(
        &mut state,
        Border(i),
        &Border((i + if dir { 1 } else { 3 }) % 4),
      );
      let new_hash = state.hasher.finish();
      if new_hash < hash {
        hash = new_hash
      }
    }
  }
  return hash;
  fn visit_node(state: &mut State<'_>, node: Node, last: &Node) {
    let mut fresh = false;
    let next_node_id = &mut state.next_node_id;
    state
      .hasher
      .write_u32(*state.node_id_map.entry(node).or_insert_with(|| {
        fresh = true;
        std::mem::replace(next_node_id, *next_node_id + 1)
      }));
    if fresh {
      let connected_nodes = &state.div.connections[&node];
      for &next in maybe_reverse(connected_nodes.iter_starting_at(last), state.dir) {
        if &next != last {
          visit_node(state, next, &node);
        }
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
