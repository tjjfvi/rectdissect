use crate::*;

#[derive(Clone, Debug)]
pub struct Division {
  pub regions: u8,
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
            Node::border(i),
            CircularOrder::new(vec![
              Node::border(i + 1),
              Node::region(0),
              Node::border(i + 3),
            ]),
          );
        }
        connections.insert(
          Node::region(0),
          CircularOrder::new(vec![
            Node::border(0),
            Node::border(1),
            Node::border(2),
            Node::border(3),
          ]),
        );
        connections
      },
    }
  }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node(u8);

impl Node {
  pub const fn border(n: u8) -> Node {
    Node(n % 4)
  }
  pub const fn region(n: u8) -> Node {
    Node(n + 4)
  }
  pub const fn is_border(&self) -> bool {
    self.0 < 4
  }
  pub const fn is_region(&self) -> bool {
    self.0 >= 4
  }
}

impl Debug for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0 < 4 {
      write!(f, "b{}", self.0)
    } else {
      write!(f, "r{}", self.0 - 4)
    }
  }
}

macro_rules! _node_consts {
  ($kind:ident $($name:ident $num:expr)+) => {
    $(pub const $name: Node = Node::$kind($num);)+
  };
}

impl Node {
  #![allow(non_upper_case_globals, dead_code)]
  _node_consts!(border b0 0 b1 1 b2 2 b3 3);
  _node_consts!(region r0 0 r1 1 r2 2 r3 3 r4 4 r5 5 r6 6 r7 7 r8 8 r9 9 r10 10 r11 11 r12 12 r13 13 r14 14 r15 15);
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
