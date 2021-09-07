use crate::*;

#[derive(Clone, Debug)]
pub struct Division {
  pub regions: u32,
  pub connections: HashMap<Node, CircularOrder<Node>>,
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
            CircularOrder::new(vec![Border((i + 1) % 4), Region(0), Border((i + 3) % 4)]),
          );
        }
        connections.insert(
          Region(0),
          CircularOrder::new(vec![Border(0), Border(1), Border(2), Border(3)]),
        );
        connections
      },
    }
  }
}

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

pub use Node::{Border, Region};

macro_rules! _node_consts {
  ($kind:ident $($name:ident $num:expr)+) => {
    $(pub const $name: Node = $kind($num);)+
  };
}

impl Node {
  #![allow(non_upper_case_globals, dead_code)]
  _node_consts!(Border b0 0 b1 1 b2 2 b3 3);
  _node_consts!(Region r0 0 r1 1 r2 2 r3 3 r4 4 r5 5 r6 6 r7 7 r8 8 r9 9 r10 10 r11 11 r12 12 r13 13 r14 14 r15 15);
}

#[macro_export]
macro_rules! division {
  ( regions : $regions:expr, connections : { $( $a:ident : [ $( $b:ident , )+ ] , )+ }, ) => {
    $crate::Division {
      regions: $regions,
      connections: {
        let mut map = std::collections::HashMap::with_capacity($regions + 4);
        $(map.insert(Node::$a, $crate::CircularOrder::new(vec![$(Node::$b),+]));)+
        map
      },
    }
  };
}
