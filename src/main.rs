use std::{
  cmp::min,
  collections::{hash_map::DefaultHasher, HashMap, HashSet},
  fmt::Debug,
  hash::{Hash, Hasher},
  ops::{Div, Index, Neg},
};

use either::Either;
use pairhashmap::PairHashMap;

mod pairhashmap;

#[derive(Clone, Debug)]
struct Division {
  regions: u32,
  connections: PairHashMap<Node, ()>,
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
        let mut connections = PairHashMap::new();
        for i in 0..4 {
          connections.add(Border(i), Region(0), ());
          connections.add(Border(i), Border((i + 1) % 4), ());
        }
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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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
    let connected_nodes = get_connected_nodes(Region(region), &div.connections);
    for cut_ind_0 in 0..connected_nodes.len() {
      let cut_ind_1_min = cut_ind_0 + 2;
      let cut_ind_1_max = min(
        connected_nodes.len() - 1,
        connected_nodes.len() + cut_ind_0 - 2,
      );
      for cut_ind_1 in cut_ind_1_min..=cut_ind_1_max {
        let must_share_0 = cut_ind_1 - cut_ind_0 < 3;
        let must_share_1 = cut_ind_0 + connected_nodes.len() - cut_ind_1 < 3;
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
            for i in cut_ind_0 + share_0 as usize..cut_ind_1 {
              new_connections.remove(&Region(region), &connected_nodes[i]);
            }
            for i in cut_ind_0..cut_ind_1 + share_1 as usize {
              new_connections.add(Region(new_region), connected_nodes[i], ());
            }
            new_connections.add(Region(region), Region(new_region), ());
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
  let mut x = Division::default();
  foo(Division::default(), |y| {
    x = y;
  });
  let mut y = Division::default();
  foo(x, |z| {
    y = z;
  });
  check_valid(&y);
}

fn get_connected_nodes(node: Node, connections: &PairHashMap<Node, ()>) -> Vec<Node> {
  _get_connected_nodes(
    node,
    connections,
    vec![*connections.get_all(&node).unwrap().keys().next().unwrap()],
  )
}

fn _get_connected_nodes(
  node: Node,
  connections: &PairHashMap<Node, ()>,
  mut vec: Vec<Node>,
) -> Vec<Node> {
  let set = connections.get_all(&node).unwrap();
  let mut reverse = false;
  for _ in 0..set.len() - vec.len() {
    let mut cur = vec.last().unwrap();
    if matches!((node, cur), (Border(_), Border(_))) {
      vec.reverse();
      reverse = !reverse;
      cur = vec.last().unwrap();
    }
    let new = *connections
      .get_all(cur)
      .unwrap()
      .keys()
      .chain(
        connections
          .get_all(&cur)
          .unwrap()
          .keys()
          .flat_map(|x| connections.get_all(&x).unwrap().keys()),
      )
      .find(|x| set.contains_key(x) && !vec.contains(x))
      .unwrap();
    vec.push(new);
  }
  debug_assert_eq!(vec.len(), set.len());
  debug_assert_eq!(vec.iter().collect::<HashSet<_>>().len(), vec.len());
  if reverse {
    vec.reverse();
  }
  vec
}

fn get_embedding(div: &Division) -> HashMap<Node, Vec<Node>> {
  let mut embedding = HashMap::new();

  let mut todo = vec![];
  {
    let node = Region(0);
    todo.push((
      node,
      vec![
        *div
          .connections
          .get_all(&node)
          .unwrap()
          .iter()
          .next()
          .unwrap()
          .0,
      ],
    ));
  }
  while let Some((node, vec)) = todo.pop() {
    let vec = _get_connected_nodes(node, &div.connections, vec);
    for (i, &next) in vec.iter().enumerate() {
      if embedding.contains_key(&next) {
        continue;
      }
      let prev = vec[(if i == 0 { vec.len() } else { i }) - 1];
      todo.push((
        next,
        vec![
          node,
          if div.connections.get(&next, &prev).is_some() {
            prev
          } else {
            *div
              .connections
              .get_all(&next)
              .unwrap()
              .iter()
              .map(|x| x.0)
              .find(|x| div.connections.get(x, &prev).is_some() && **x != node)
              .unwrap()
          },
        ],
      ))
    }
    embedding.insert(node, vec);
  }
  embedding
}

fn hash_division(div: &Division) -> u64 {
  let mut hash = u64::MAX;
  let embedding = get_embedding(div);
  for &node_0 in div.connections.keys() {
    for &node_1 in div.connections.get_all(&node_0).unwrap().keys() {
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
          for node in next_node_candidates(&embedding, dir, cur_node, last_node) {
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
        let new_hash = hasher.finish();
        if new_hash < hash {
          hash = new_hash
        }
      }
    }
  }
  return hash;
  fn next_node_candidates<'a>(
    embedding: &'a HashMap<Node, Vec<Node>>,
    dir: bool,
    from: Node,
    start: Node,
  ) -> impl Iterator<Item = Node> + 'a {
    let vec = embedding.get(&from).unwrap();
    let start = vec.iter().position(|x| *x == start).unwrap();
    if dir {
      Either::Left((start..vec.len()).chain(0..start))
    } else {
      Either::Right((start + 1..vec.len()).chain(0..=start).rev())
    }
    .map(move |i| vec[i])
  }
}

fn check_valid(div: &Division) -> bool {
  dbg!(div);
  struct State<'a> {
    edge_labels: PairHashMap<Node, bool>,
    div: &'a Division,
    ambiguous_edges: Vec<(Node, Node)>,
  }
  type Result = std::result::Result<(), ()>;
  let mut state = State {
    edge_labels: PairHashMap::new(),
    ambiguous_edges: vec![],
    div,
  };
  for border_n in 0..4 {
    for node in div.connections.get_all(&Border(border_n)).unwrap().keys() {
      add_label(
        &mut state,
        Border(border_n),
        *node,
        matches!(node, Border(_)) || border_n % 2 == 0,
      )
      .unwrap();
    }
  }
  dbg!(state.edge_labels);
  return true;

  fn add_label(state: &mut State, a: Node, b: Node, label: bool) -> Result {
    if let Some(prev_label) = state.edge_labels.add(a, b, label) {
      return if label == prev_label { Ok(()) } else { Err(()) };
    }
    for &c in state
      .div
      .connections
      .get_all(&a)
      .unwrap()
      .keys()
      .filter(|c| state.div.connections.get(&b, c).is_some())
      .collect::<Vec<_>>()
    {
      let a_leg = state.edge_labels.get(&a, &c);
      let b_leg = state.edge_labels.get(&b, &c);
      match (label, a_leg, b_leg) {
        (_, None, None) => Ok(()),
        (true, Some(true), None) => add_label(state, b, c, false),
        (false, Some(false), None) => add_label(state, b, c, true),
        (true, None, Some(true)) => add_label(state, a, c, false),
        (false, None, Some(false)) => add_label(state, a, c, true),
        (a, Some(&b), Some(&c)) => {
          if (a as u8 + b as u8 + c as u8) % 3 != 0 {
            Ok(())
          } else {
            Err(())
          }
        }
        (_, Some(_), None) => Ok(state.ambiguous_edges.push((b, c))),
        (_, None, Some(_)) => Ok(state.ambiguous_edges.push((a, c))),
      }?;
    }
    check_node(state, a)?;
    check_node(state, b)?;
    return Ok(());

    fn check_node(state: &mut State, node: Node) -> Result {
      if !matches!(node, Region(_)) {
        return Ok(());
      }
      let mut vec_true = vec![];
      let mut vec_false = vec![];
      let mut vec_none = vec![];
      for &connected_node in state.div.connections.get_all(&node).unwrap().keys() {
        let vec = match state.edge_labels.get(&node, &connected_node) {
          Some(true) => &mut vec_true,
          Some(false) => &mut vec_false,
          None => &mut vec_none,
        };
        vec.push(connected_node);
      }
      if vec_true.len() + vec_none.len() < 2 || vec_false.len() + vec_none.len() < 2 {
        return Err(());
      }
      if vec_none.len() != 0 {
        if vec_true.len() + vec_none.len() == 2 {
          println!("hi");
          for &connected_node in &vec_none {
            add_label(state, node, connected_node, true)?;
          }
          return Ok(());
        }
        if vec_true.len() + vec_none.len() == 2 {
          println!("hi");
          for &connected_node in &vec_none {
            add_label(state, node, connected_node, false)?;
          }
          return Ok(());
        }
      }
      if vec_true.len() == 0 || vec_false.len() == 0 {
        return Ok(());
      }
      let (mut vec_true_0, mut vec_true_1) = segregate(state, vec_true);
      let (mut vec_false_0, mut vec_false_1) = segregate(state, vec_false);
      dbg!(&vec_true_0, &vec_true_1, &vec_false_0, &vec_false_1);
      let mut empty_vecs_iter = <_>::into_iter([
        (&mut vec_true_0, true),
        (&mut vec_true_1, true),
        (&mut vec_false_0, false),
        (&mut vec_false_1, false),
      ])
      .filter(|x| x.0.len() == 0);
      if vec_none.len() < 2 {
        if let Some((_, label)) = empty_vecs_iter.next() {
          if vec_none.len() == 0 {
            return Err(());
          } else {
            return add_label(state, node, vec_none[0], label);
          }
        }
      } else {
        if vec_none.len() < empty_vecs_iter.count() {
          return Err(());
        }
      }
      return Ok(());
      fn segregate(state: &mut State, mut vec: Vec<Node>) -> (Vec<Node>, Vec<Node>) {
        let mut vec_0 = vec![];
        let mut edges = vec![vec.pop().unwrap()];
        let mut new_edges = vec![];
        while edges.len() != 0 {
          new_edges.clear();
          for &node in &vec {
            if edges
              .iter()
              .any(|&edge| state.div.connections.get(&node, &edge).is_some())
            {
              new_edges.push(node);
            }
          }
          if edges.len() != 0 {
            vec.retain(|x| !new_edges.contains(x));
          }
          vec_0.extend(edges.drain(..));
          std::mem::swap(&mut edges, &mut new_edges);
        }
        (vec_0, vec)
      }
    }
  }
}
