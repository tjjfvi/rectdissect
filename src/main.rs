use std::{
  collections::{hash_map::DefaultHasher, HashMap, HashSet},
  fmt::Debug,
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
  for i in 1..5 {
    let mut x = 0;
    println!("{}", set.len());
    for div in set.drain() {
      foo(div, |new_div| {
        if check_valid(&new_div) {
          x += 1;
          new_set.insert(new_div);
        }
      })
    }
    std::mem::swap(&mut set, &mut new_set);
    dbg!(x);
  }
  println!("{}", set.len());
  // let mut x = Division::default();
  // foo(Division::default(), |y| {
  //   x = y;
  // });
  // let mut y = Division::default();
  // foo(x, |z| {
  //   y = z;
  // });
  // check_valid(&y);
}

fn hash_division(div: &Division) -> u64 {
  let mut hash = u64::MAX;
  for (&node_0, connected_nodes) in &div.connections {
    for &node_1 in connected_nodes.iter() {
      for dir in [true, false] {
        let mut hasher = DefaultHasher::new();
        let mut last_node = node_0;
        let mut cur_node = node_1;
        let mut node_id_map = HashMap::new();
        node_id_map.insert(node_0, 0);
        let mut node_id_max = 0;
        let mut visited_edges = HashSet::new();
        visited_edges.insert((last_node, cur_node));
        loop {
          let cur_node_id = *node_id_map.entry(cur_node).or_insert_with(|| {
            node_id_max += 1;
            node_id_max
          });
          hasher.write_u32(cur_node_id);
          let visited = cur_node_id != node_id_max;
          let mut next_node = None;
          for &node in next_node_candidates(&div, dir, &cur_node, &last_node) {
            if (visited || node != last_node) && visited_edges.insert((cur_node, node)) {
              next_node = Some(node);
              break;
            }
          }
          if let Some(next_node) = next_node {
            last_node = cur_node;
            cur_node = next_node;
          } else {
            break;
          }
        }
        debug_assert_eq!(cur_node, node_0);
        debug_assert_eq!(node_id_max, div.regions + 3);
        debug_assert_eq!(
          visited_edges.len(),
          div.connections.values().map(|x| x.len()).sum()
        );
        let new_hash = hasher.finish();
        if new_hash < hash {
          hash = new_hash
        }
      }
    }
  }
  return hash;
  fn next_node_candidates<'a>(
    div: &'a Division,
    dir: bool,
    from: &Node,
    start: &'a Node,
  ) -> impl Iterator<Item = &'a Node> + 'a {
    let iter = div.connections.get(&from).unwrap().iter_starting_at(&start);
    if dir {
      Either::Left(iter)
    } else {
      Either::Right(iter.rev())
    }
  }
}

fn check_valid(div: &Division) -> bool {
  // dbg!(div);
  #[derive(Clone, Debug)]
  struct State<'a> {
    edge_labels: HashMap<UnorderedPair<Node>, bool>,
    div: &'a Division,
    /// These edges are part of a 0-1-? triangle, and should be guessed at first
    ambiguous_edges: Vec<UnorderedPair<Node>>,
    unlabeled_edges: HashSet<UnorderedPair<Node>>,
  }
  type Result = std::result::Result<(), ()>;
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
      if add_label(
        &mut state,
        Border(border_n),
        *node,
        matches!(node, Border(_)) || border_n % 2 == 0,
      )
      .is_err()
      {
        return false;
      }
    }
  }
  return finish_state(state).is_ok();

  fn finish_state(mut state: State) -> Result {
    if state.unlabeled_edges.len() == 0 {
      return Ok(()); // All edges have been labeled successfully
    }
    let edge = state
      .ambiguous_edges
      .pop()
      .or_else(|| state.unlabeled_edges.iter().next().map(|&x| x))
      .unwrap();
    for guess in [true, false] {
      let mut state_clone = state.clone();
      if add_label(&mut state_clone, edge.0, edge.1, guess)
        .and_then(|_| finish_state(state_clone))
        .is_ok()
      {
        return Ok(()); // One of the guesses worked
      };
    }
    Err(()) // Neither of the guesses worked
  }

  fn add_label(state: &mut State, a: Node, b: Node, label: bool) -> Result {
    state.unlabeled_edges.remove(&UnorderedPair(a, b));
    if matches!((a, b), (Border(_), Border(_))) {
      return Ok(());
    }
    if let Some(prev_label) = state.edge_labels.insert(UnorderedPair(a, b), label) {
      return if label == prev_label { Ok(()) } else { Err(()) };
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
          (_, None, None) => Ok(()),
          (true, Some(true), _) => add_label(state, b, c, false),
          (false, Some(false), _) => add_label(state, b, c, true),
          (_, Some(_), None) => Ok(state.ambiguous_edges.push(UnorderedPair(b, c))),
          (true, _, Some(true)) => add_label(state, a, c, false),
          (false, _, Some(false)) => add_label(state, a, c, true),
          (_, None, Some(_)) => Ok(state.ambiguous_edges.push(UnorderedPair(a, c))),
          (false, Some(true), Some(true)) => Ok(()),
          (true, Some(false), Some(false)) => Ok(()),
        }?;
      } else {
        add_label(state, a, c, !label)?;
        add_label(state, b, d, !label)?;
        add_label(state, c, d, label)?;
      }
    }
    check_node(state, a)?;
    check_node(state, b)?;
    return Ok(());

    fn check_node(state: &mut State, node: Node) -> Result {
      if matches!(node, Border(_)) {
        return Ok(());
      }
      let connected_nodes = state.div.connections.get(&node).unwrap();
      let mut all_true = vec![];
      let mut all_false = vec![];
      let mut all_none = vec![];
      let mut vecs: Vec<(Vec<Node>, Option<bool>)> = vec![];
      let mut true_vecs_count = 0;
      let mut false_vecs_count = 0;
      let mut _none_vecs_count = 0;
      for &connected_node in connected_nodes.iter() {
        let label = state
          .edge_labels
          .get(&UnorderedPair(node, connected_node))
          .cloned();
        match state.edge_labels.get(&UnorderedPair(node, connected_node)) {
          Some(true) => &mut all_true,
          Some(false) => &mut all_false,
          None => &mut all_none,
        }
        .push(connected_node.clone());
        match vecs.last_mut() {
          Some((ref mut vec, label2)) if *label2 == label => vec.push(connected_node),
          _ => {
            vecs.push((vec![connected_node], label));
            match label {
              Some(true) => true_vecs_count += 1,
              Some(false) => false_vecs_count += 1,
              None => _none_vecs_count += 1,
            }
          }
        }
      }
      if vecs.len() > 1 && vecs.last().unwrap().1 == vecs[0].1 {
        let (partial_vec, label) = vecs.pop().unwrap();
        vecs[0].0.splice(0..0, partial_vec);
        match label {
          Some(true) => true_vecs_count -= 1,
          Some(false) => false_vecs_count -= 1,
          None => _none_vecs_count -= 1,
        }
      }
      if all_true.len() + all_none.len() < 2 || all_false.len() + all_none.len() < 2 {
        return Err(());
      }
      if all_none.len() != 0 {
        if all_true.len() + all_none.len() == 2 {
          for &connected_node in &all_none {
            add_label(state, node, connected_node, true)?;
          }
          return Ok(());
        }
        if all_true.len() + all_none.len() == 2 {
          for &connected_node in &all_none {
            add_label(state, node, connected_node, false)?;
          }
          return Ok(());
        }
      }
      if all_true.len() == 0 || all_false.len() == 0 {
        return Ok(());
      }
      if all_none.len() == 0 && (true_vecs_count != 2 || false_vecs_count != 2) {
        return Err(());
      }
      Ok(())
    }
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
