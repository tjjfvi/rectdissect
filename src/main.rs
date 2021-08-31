use std::{
  cmp::min,
  collections::{hash_map::DefaultHasher, HashMap, HashSet},
  hash::{Hash, Hasher},
  ops::{Div, Neg},
};

use pairhashmap::PairHashMap;

mod pairhashmap;

#[derive(Clone, Debug)]
struct Division {
  regions: u32,
  connections: PairHashMap<Node, ()>,
}

impl Hash for Division {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    let mut hash = u64::MAX;
    let mut done_nodes = HashSet::new();
    for &node_0 in self.connections.keys() {
      done_nodes.insert(node_0);
      for &node_1 in self.connections.get_all(&node_0).unwrap().keys() {
        if done_nodes.contains(&node_1) {
          continue;
        }
        let hasher = DefaultHasher::new();
        let mut last_node = node_0;
        let mut cur_node = node_1;
        let next_node_id = 2;
        let mut node_id_map = HashMap::new();
        node_id_map.insert(node_0, 0);
        node_id_map.insert(node_1, 1);
        let next_node_id = 2;
        let mut visisted_edges = HashSet::new();
        visisted_edges.insert((node_0, node_1));
        let new_hash = hasher.finish();
        if new_hash < hash {
          hash = new_hash
        }
      }
    }
    state.write_u64(hash)
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Node {
  Border(u8),
  Region(u32),
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
        dbg!((cut_ind_0, cut_ind_1));
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
            for i in cut_ind_0..cut_ind_1 + share_1 as usize {
              new_connections.remove(&Region(region), &connected_nodes[i]);
            }
            for i in (cut_ind_1..connected_nodes.len()).chain(0..cut_ind_0 + share_0 as usize) {
              new_connections.add(Region(new_region), connected_nodes[i], ());
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
  let root = Division::default();
  foo(root, |x| {
    dbg!(x);
  });
}

fn get_connected_nodes(node: Node, connections: &PairHashMap<Node, ()>) -> Vec<Node> {
  _get_connected_nodes(
    node,
    connections,
    *connections.get_all(&node).unwrap().keys().next().unwrap(),
  )
}

fn _get_connected_nodes(node: Node, connections: &PairHashMap<Node, ()>, first: Node) -> Vec<Node> {
  let set = connections.get_all(&node).unwrap();
  let mut vec = vec![];
  let mut cur = first;
  loop {
    if Some(&cur) == vec.get(0) {
      break;
    }
    let new_cur = *connections
      .get_all(&cur)
      .unwrap()
      .keys()
      .chain(
        connections
          .get_all(&cur)
          .unwrap()
          .keys()
          .flat_map(|x| connections.get_all(&x).unwrap().keys()),
      )
      .find(|x| set.contains_key(x) && Some(*x) != vec.last())
      .unwrap();
    vec.push(cur);
    cur = new_cur;
  }
  vec
}
