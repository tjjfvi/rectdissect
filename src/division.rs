use crate::*;

#[derive(Clone, Debug)]
pub struct Division {
  pub regions: u32,
  pub connections: HashMap<Node, CircularOrder<Node>>,
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
pub enum Node {
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

pub use Node::*;
