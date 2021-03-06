use std::collections::{HashMap, HashSet};

use crate::*;

pub type EdgeLabels = HashMap<UnorderedPair<Node>, bool>;

#[derive(Clone, Debug)]
struct State<'a> {
  edge_labels: EdgeLabels,
  div: &'a Division,
  /// These edges are part of a 0-1-? triangle, and should be guessed at first
  ambiguous_edges: Vec<UnorderedPair<Node>>,
  unlabeled_edges: HashSet<UnorderedPair<Node>>,
}

pub struct LabelEdgesIter<'a> {
  div: &'a Division,
  labels_todo: Vec<(Node, Node, bool)>,
  nodes_todo: Vec<Node>,
  states: Vec<State<'a>>,
}

pub fn label_edges(div: &Division) -> LabelEdgesIter<'_> {
  let mut state = State {
    edge_labels: HashMap::new(),
    ambiguous_edges: vec![],
    div,
    unlabeled_edges: div
      .nodes()
      .flat_map(|a| div[a].iter().map(move |b| UnorderedPair(a, b)))
      .collect(),
  };
  let mut labels_todo = Vec::new();
  let mut nodes_todo = Vec::new();
  for border_n in 0..4 {
    for node in div[Node::border(border_n)].iter() {
      labels_todo.push((
        Node::border(border_n),
        node,
        node.is_border() || border_n % 2 == 0,
      ));
    }
  }
  let states = if flush_todos(&mut state, &mut labels_todo, &mut nodes_todo).is_some() {
    vec![state]
  } else {
    vec![]
  };
  LabelEdgesIter {
    div,
    labels_todo,
    nodes_todo,
    states,
  }
}

impl Iterator for LabelEdgesIter<'_> {
  type Item = EdgeLabels;
  fn next(&mut self) -> Option<EdgeLabels> {
    while let Some(mut state) = self.states.pop() {
      debug_assert!(self.labels_todo.is_empty());
      debug_assert!(self.nodes_todo.is_empty());
      if state.unlabeled_edges.len() == 0 {
        if cfg!(debug_assertions) {
          self.nodes_todo.extend(self.div.regions());
          assert!(flush_todos(&mut state, &mut self.labels_todo, &mut self.nodes_todo).is_some());
          assert!(self.nodes_todo.is_empty());
        }
        return Some(state.edge_labels); // All edges have been labeled successfully
      }
      let edge = state
        .ambiguous_edges
        .pop()
        .or_else(|| state.unlabeled_edges.iter().next().map(|&x| x))
        .unwrap();
      for (guess, state) in [(true, state.clone()), (false, state)] {
        let mut state = state.clone();
        self.labels_todo.push((edge.0, edge.1, guess));
        match flush_todos(&mut state, &mut self.labels_todo, &mut self.nodes_todo) {
          Some(()) => self.states.push(state),
          None => {
            self.labels_todo.clear();
            self.nodes_todo.clear();
          }
        }
      }
    }
    None
  }
}

#[must_use]
fn flush_todos(
  state: &mut State,
  labels_todo: &mut Vec<(Node, Node, bool)>,
  nodes_todo: &mut Vec<Node>,
) -> Option<()> {
  loop {
    if let Some((a, b, label)) = labels_todo.pop() {
      state.unlabeled_edges.remove(&UnorderedPair(a, b));
      if a.is_border() && b.is_border() {
        continue;
      }
      if let Some(prev_label) = state.edge_labels.insert(UnorderedPair(a, b), label) {
        if label == prev_label {
          continue;
        } else {
          return None;
        };
      }
      let a_connected_nodes = &state.div[a];
      let b_connected_nodes = &state.div[b];
      let (c0, c1) = a_connected_nodes.get_items_around(b);
      let (d1, d0) = b_connected_nodes.get_items_around(a);
      for (c, d) in [(c0, d0), (c1, d1)] {
        if c == d {
          let a_leg = state.edge_labels.get(&UnorderedPair(a, c));
          let b_leg = state.edge_labels.get(&UnorderedPair(b, c));
          match (label, a_leg, b_leg) {
            (false, Some(false), Some(false)) | (true, Some(true), Some(true)) => return None,
            (_, Some(_), Some(_)) => {}
            (_, None, None) => {}
            (true, Some(true), None) => labels_todo.push((b, c, false)),
            (false, Some(false), None) => labels_todo.push((b, c, true)),
            (_, Some(_), None) => state.ambiguous_edges.push(UnorderedPair(b, c)),
            (true, None, Some(true)) => labels_todo.push((a, c, false)),
            (false, None, Some(false)) => labels_todo.push((a, c, true)),
            (_, None, Some(_)) => state.ambiguous_edges.push(UnorderedPair(a, c)),
          }
        } else if state.div[c].contains_item(d) {
          labels_todo.push((a, c, !label));
          labels_todo.push((b, d, !label));
          labels_todo.push((c, d, label));
        } else {
          return None;
        }
      }
      nodes_todo.push(a);
      nodes_todo.push(b);
    } else if let Some(node) = nodes_todo.pop() {
      if node.is_border() {
        continue;
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
            labels_todo.push((node, connected_node, true));
          }
          continue;
        }
        if all_false.len() + all_none.len() == 2 {
          for &connected_node in &all_none {
            labels_todo.push((node, connected_node, false));
          }
          continue;
        }
      }
      if all_none.len() == 0 && (true_vecs_count != 2 || false_vecs_count != 2) {
        return None;
      }
    } else {
      return Some(());
    }
  }
}

#[derive(Default, Debug, Clone)]
pub struct ConnectedNodesClassification {
  pub all_true: Vec<Node>,
  pub all_false: Vec<Node>,
  pub all_none: Vec<Node>,
  pub vecs: Vec<(Vec<Node>, Option<bool>)>,
  pub true_vecs_count: u8,
  pub false_vecs_count: u8,
  pub none_vecs_count: u8,
}

pub fn classify_connected_nodes(
  node: Node,
  div: &Division,
  edge_labels: &EdgeLabels,
) -> ConnectedNodesClassification {
  let mut state = ConnectedNodesClassification::default();
  for connected_node in div[node].iter() {
    let label = edge_labels
      .get(&UnorderedPair(node, connected_node))
      .copied();
    match label {
      Some(true) => &mut state.all_true,
      Some(false) => &mut state.all_false,
      None => &mut state.all_none,
    }
    .push(connected_node);
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
