use std::{
  collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
  fmt::{Debug, Write},
  hash::{Hash, Hasher},
};

use circularorder::CircularOrder;
use either::Either;

mod circularorder;
mod pairhashmap;

#[derive(Clone, Debug)]
struct Division {
  regions: u32,
  connections: HashMap<Node, CircularOrder<Node>>,
}

impl Hash for Division {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    state.write_u64(hash_division(self));
  }
}

impl Default for Division {
  fn default() -> Self {
    Division {
      regions: 1,
      connections: {
        let mut connections = HashMap::new();
        for i in 0..4 {
          connections.insert(
            Border(i),
            CircularOrder::new([Border((i + 1) % 4), Region(0), Border((i + 3) % 4)]),
          );
        }
        connections.insert(
          Region(0),
          CircularOrder::new([Border(0), Border(1), Border(2), Border(3)]),
        );
        connections
      },
    }
  }
}

impl PartialEq for Division {
  fn eq(&self, other: &Self) -> bool {
    hash_division(self) == hash_division(other)
  }
}

impl Eq for Division {}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
  Border(u8),
  Region(u32),
}

impl Debug for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Border(x) => write!(f, "b{}", x),
      Region(x) => write!(f, "r{}", x),
    }
  }
}

use Node::*;

fn foo(div: Division, mut cb: impl FnMut(Division)) {
  for region in 0..div.regions {
    let connected_nodes = div.connections.get(&Region(region)).unwrap();
    for (cut_0_ind, cut_0) in connected_nodes.iter().enumerate() {
      for (cut_1_ind, cut_1) in connected_nodes
        .iter()
        .enumerate()
        .take(connected_nodes.len() + cut_0_ind - 1)
        .skip(cut_0_ind + 2)
      {
        let must_share_0 = cut_1_ind - cut_0_ind < 3;
        let must_share_1 = cut_0_ind + connected_nodes.len() - cut_1_ind < 3;
        for share_0 in [true, false] {
          if must_share_0 && !share_0 {
            continue;
          }
          for share_1 in [true, false] {
            if must_share_1 && !share_1 {
              continue;
            }
            let new_region = div.regions;
            let mut new_connections = div.connections.clone();
            {
              let order = new_connections.get_mut(&Region(region)).unwrap();
              order
                .delete_items_between(
                  cut_0,
                  &(if share_1 {
                    *cut_1
                  } else {
                    *order.get_item_after(cut_1).unwrap()
                  }),
                )
                .unwrap();
              order
                .insert_items_after(cut_0, [Region(new_region)])
                .unwrap();
            }
            {
              let mut order = connected_nodes.clone();
              order
                .delete_items_between(
                  cut_1,
                  &(if share_0 {
                    *cut_0
                  } else {
                    *order.get_item_after(cut_0).unwrap()
                  }),
                )
                .unwrap();
              order.insert_items_after(cut_1, [Region(region)]).unwrap();
              new_connections.insert(Region(new_region), order);
            }
            for (i, node) in connected_nodes.iter().enumerate() {
              if i == cut_0_ind && share_0 || i == cut_1_ind && share_1 {
                let order = new_connections.get_mut(node).unwrap();
                order
                  .insert_items_after(
                    &(if i == cut_0_ind {
                      *order.get_item_before(&Region(region)).unwrap()
                    } else {
                      Region(region)
                    }),
                    [Region(new_region)],
                  )
                  .unwrap();
              } else if i > cut_0_ind && i <= cut_1_ind {
                let order = new_connections.get_mut(node).unwrap();
                let before = order.get_item_before(&Region(region)).unwrap().clone();
                order
                  .delete_items_between(
                    &before,
                    &order.get_item_after(&Region(region)).unwrap().clone(),
                  )
                  .unwrap();
                order
                  .insert_items_after(&before, [Region(new_region)])
                  .unwrap();
              }
            }
            let div = Division {
              regions: div.regions + 1,
              connections: new_connections,
            };
            cb(div);
          }
        }
      }
    }
  }
}

fn main() {
  let mut set = HashSet::new();
  let mut new_set = HashSet::new();
  set.insert(Division::default());
  for _ in 1..4 {
    println!("{}", set.len());
    for div in set.drain() {
      foo(div, |new_div| {
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

fn hash_division(div: &Division) -> u64 {
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
      let connected_nodes = state.div.connections.get(&node).unwrap();
      for &next in maybe_reverse(connected_nodes.iter_starting_at(last), state.dir) {
        visit_node(state, next, &node);
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

fn label_edges(div: &Division) -> Option<HashMap<UnorderedPair<Node>, bool>> {
  // dbg!(div);
  #[derive(Clone, Debug)]
  struct State<'a> {
    edge_labels: HashMap<UnorderedPair<Node>, bool>,
    div: &'a Division,
    /// These edges are part of a 0-1-? triangle, and should be guessed at first
    ambiguous_edges: Vec<UnorderedPair<Node>>,
    unlabeled_edges: HashSet<UnorderedPair<Node>>,
  }
  let mut state = State {
    edge_labels: HashMap::new(),
    ambiguous_edges: vec![],
    div,
    unlabeled_edges: div
      .connections
      .iter()
      .flat_map(|(&key_a, x)| x.iter().map(move |&key_b| UnorderedPair(key_a, key_b)))
      .collect(),
  };
  for border_n in 0..4 {
    for node in div.connections.get(&Border(border_n)).unwrap().iter() {
      add_label(
        &mut state,
        Border(border_n),
        *node,
        matches!(node, Border(_)) || border_n % 2 == 0,
      )?;
    }
  }
  return Some(finish_state(state)?.edge_labels);

  fn finish_state(mut state: State) -> Option<State> {
    if state.unlabeled_edges.len() == 0 {
      return Some(state); // All edges have been labeled successfully
    }
    let edge = state
      .ambiguous_edges
      .pop()
      .or_else(|| state.unlabeled_edges.iter().next().map(|&x| x))
      .unwrap();
    for guess in [true, false] {
      let mut state_clone = state.clone();
      if let Some(sucess_state) =
        add_label(&mut state_clone, edge.0, edge.1, guess).and_then(|_| finish_state(state_clone))
      {
        return Some(sucess_state); // One of the guesses worked
      };
    }
    None // Neither of the guesses worked
  }

  fn add_label(state: &mut State, a: Node, b: Node, label: bool) -> Option<()> {
    state.unlabeled_edges.remove(&UnorderedPair(a, b));
    if matches!((a, b), (Border(_), Border(_))) {
      return Some(());
    }
    if let Some(prev_label) = state.edge_labels.insert(UnorderedPair(a, b), label) {
      return if label == prev_label { Some(()) } else { None };
    }
    let a_connected_nodes = state.div.connections.get(&a).unwrap();
    let b_connected_nodes = state.div.connections.get(&b).unwrap();
    let (c0, c1) = a_connected_nodes.get_items_around(&b).unwrap();
    let (d1, d0) = b_connected_nodes.get_items_around(&a).unwrap();
    for (&c, &d) in [(c0, d0), (c1, d1)] {
      if c == d {
        let a_leg = state.edge_labels.get(&UnorderedPair(a, c));
        let b_leg = state.edge_labels.get(&UnorderedPair(b, c));
        match (label, a_leg, b_leg) {
          (_, None, None) => Some(()),
          (true, Some(true), _) => add_label(state, b, c, false),
          (false, Some(false), _) => add_label(state, b, c, true),
          (_, Some(_), None) => Some(state.ambiguous_edges.push(UnorderedPair(b, c))),
          (true, _, Some(true)) => add_label(state, a, c, false),
          (false, _, Some(false)) => add_label(state, a, c, true),
          (_, None, Some(_)) => Some(state.ambiguous_edges.push(UnorderedPair(a, c))),
          (false, Some(true), Some(true)) => Some(()),
          (true, Some(false), Some(false)) => Some(()),
        }?;
      } else {
        add_label(state, a, c, !label)?;
        add_label(state, b, d, !label)?;
        add_label(state, c, d, label)?;
      }
    }
    check_node(state, a)?;
    check_node(state, b)?;
    return Some(());

    fn check_node(state: &mut State, node: Node) -> Option<()> {
      if matches!(node, Border(_)) {
        return Some(());
      }
      let ConnectedNodesClassification {
        all_true,
        all_false,
        all_none,
        true_vecs_count,
        false_vecs_count,
        ..
      } = classify_connected_nodes(node, state.div, &state.edge_labels);
      if all_true.len() + all_none.len() < 2 || all_false.len() + all_none.len() < 2 {
        return None;
      }
      if all_none.len() != 0 {
        if all_true.len() + all_none.len() == 2 {
          for &connected_node in &all_none {
            add_label(state, node, connected_node, true)?;
          }
          return Some(());
        }
        if all_true.len() + all_none.len() == 2 {
          for &connected_node in &all_none {
            add_label(state, node, connected_node, false)?;
          }
          return Some(());
        }
      }
      if all_true.len() == 0 || all_false.len() == 0 {
        return Some(());
      }
      if all_none.len() == 0 && (true_vecs_count != 2 || false_vecs_count != 2) {
        return None;
      }
      Some(())
    }
  }
}

#[derive(Default, Debug, Clone)]
struct ConnectedNodesClassification {
  all_true: Vec<Node>,
  all_false: Vec<Node>,
  all_none: Vec<Node>,
  vecs: Vec<(Vec<Node>, Option<bool>)>,
  true_vecs_count: u32,
  false_vecs_count: u32,
  none_vecs_count: u32,
}
fn classify_connected_nodes(
  node: Node,
  div: &Division,
  edge_labels: &HashMap<UnorderedPair<Node>, bool>,
) -> ConnectedNodesClassification {
  let mut state = ConnectedNodesClassification::default();
  let connected_nodes = div.connections.get(&node).unwrap();
  for &connected_node in connected_nodes.iter() {
    let label = edge_labels
      .get(&UnorderedPair(node, connected_node))
      .cloned();
    match edge_labels.get(&UnorderedPair(node, connected_node)) {
      Some(true) => &mut state.all_true,
      Some(false) => &mut state.all_false,
      None => &mut state.all_none,
    }
    .push(connected_node.clone());
    match state.vecs.last_mut() {
      Some((ref mut vec, label2)) if *label2 == label => vec.push(connected_node),
      _ => {
        state.vecs.push((vec![connected_node], label));
        match label {
          Some(true) => state.true_vecs_count += 1,
          Some(false) => state.false_vecs_count += 1,
          None => state.none_vecs_count += 1,
        }
      }
    }
  }
  if state.vecs.len() > 1 && state.vecs.last().unwrap().1 == state.vecs[0].1 {
    let (partial_vec, label) = state.vecs.pop().unwrap();
    state.vecs[0].0.splice(0..0, partial_vec);
    match label {
      Some(true) => state.true_vecs_count -= 1,
      Some(false) => state.false_vecs_count -= 1,
      None => state.none_vecs_count -= 1,
    }
  }
  state
}

fn generate_layout(
  div: &Division,
  edge_labels: &HashMap<UnorderedPair<Node>, bool>,
) -> Vec<[f64; 4]> {
  let layout_x = generate_1d_layout(div, edge_labels, false);
  let layout_y = generate_1d_layout(div, edge_labels, true);

  return (0..div.regions)
    .map(|region| {
      let (x1, x2) = layout_x[&Region(region)];
      let (y1, y2) = layout_y[&Region(region)];
      debug_assert!(!x1.is_nan() && !x2.is_nan() && !y1.is_nan() && !y2.is_nan());
      [x1, y1, x2, y2]
    })
    .collect();

  fn generate_1d_layout(
    div: &Division,
    edge_labels: &HashMap<UnorderedPair<Node>, bool>,
    axis: bool,
  ) -> HashMap<Node, (f64, f64)> {
    let root = if axis { 0 } else { 3 };
    let mut ranges = HashMap::new();
    ranges.insert(Border(root), (0.0_f64, 1.0_f64));
    let mut node_queue = VecDeque::new();
    node_queue.push_back(Border(root));
    while let Some(node) = node_queue.pop_front() {
      let (start, end) = ranges[&node];
      let mut next_nodes = {
        let mut iter = classify_connected_nodes(node, div, edge_labels)
          .vecs
          .into_iter()
          .filter(|x| {
            true
              && x.1 == Some(axis)
              && !x.0.iter().all(|x| {
                matches!(x, Border(_))
                  || match ranges.get(x) {
                    Some((a, b)) => !a.is_nan() && !b.is_nan(),
                    None => false,
                  }
              })
          });
        match iter.next() {
          Some(x) => {
            assert_eq!(iter.next(), None);
            x.0
          }
          None => continue,
        }
      };
      next_nodes.retain(|x| matches!(x, Region(_)));
      let next_nodes_count = next_nodes.len();
      for (i, next_node) in next_nodes.into_iter().enumerate() {
        println!("");
        let first = i == 0;
        let last = i == next_nodes_count - 1;
        let range = ranges.entry(next_node).or_insert((f64::NAN, f64::NAN));
        dbg!(node, next_node, first, last, &range);
        if range.0.is_nan()
          && (!first
            || edge_labels[&UnorderedPair(
              next_node,
              *div.connections[&next_node].get_item_after(&node).unwrap(),
            )] != axis)
        {
          let t = i as f64 / next_nodes_count as f64;
          range.0 = end * t + start * (1. - t);
        }
        if range.1.is_nan()
          && (!last
            || edge_labels[&UnorderedPair(
              next_node,
              *div.connections[&next_node].get_item_before(&node).unwrap(),
            )] != axis)
        {
          let t = (i + 1) as f64 / next_nodes_count as f64;
          range.1 = end * t + start * (1. - t);
        }
        dbg!(&range);
        node_queue.push_back(next_node);
      }
    }
    ranges
  }
}

#[derive(Debug, Default, Clone, Copy)]
struct UnorderedPair<T>(T, T);

impl<T: Ord> From<UnorderedPair<T>> for (T, T) {
  fn from(pair: UnorderedPair<T>) -> Self {
    if pair.0 < pair.1 {
      (pair.0, pair.1)
    } else {
      (pair.1, pair.0)
    }
  }
}

impl<'a, T: Ord> From<&'a UnorderedPair<T>> for (&'a T, &'a T) {
  fn from(pair: &'a UnorderedPair<T>) -> Self {
    if pair.0 < pair.1 {
      (&pair.0, &pair.1)
    } else {
      (&pair.1, &pair.0)
    }
  }
}

impl<T: Ord + Hash> Hash for UnorderedPair<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    <_ as Into<(_, _)>>::into(self).hash(state);
  }
}

impl<T: Ord + PartialEq> PartialEq for UnorderedPair<T> {
  fn eq(&self, other: &Self) -> bool {
    <_ as Into<(_, _)>>::into(self) == other.into()
  }
}

impl<T: Ord + Eq> Eq for UnorderedPair<T> {}

fn hash(value: impl Hash) -> u64 {
  let mut hasher = DefaultHasher::new();
  value.hash(&mut hasher);
  hasher.finish()
}
