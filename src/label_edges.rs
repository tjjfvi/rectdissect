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

pub fn label_edges(div: &Division) -> Option<EdgeLabels> {
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
  let mut labels_todo = Vec::new();
  let mut nodes_todo = HashSet::new();
  for border_n in 0..4 {
    for &node in div.connections[&Border(border_n)].iter() {
      labels_todo.push((
        Border(border_n),
        node,
        matches!(node, Border(_)) || border_n % 2 == 0,
      ));
    }
  }
  flush_todos(&mut state, &mut labels_todo, &mut nodes_todo)?;
  let mut states = vec![state];
  while let Some(mut state) = states.pop() {
    if state.unlabeled_edges.len() == 0 {
      return Some(state.edge_labels); // All edges have been labeled successfully
    }
    let edge = state
      .ambiguous_edges
      .pop()
      .or_else(|| state.unlabeled_edges.iter().next().map(|&x| x))
      .unwrap();
    for (guess, state) in [(true, state.clone()), (false, state)] {
      let mut state = state.clone();
      labels_todo.push((edge.0, edge.1, guess));
      match flush_todos(&mut state, &mut labels_todo, &mut nodes_todo) {
        Some(()) => states.push(state),
        None => {
          labels_todo.clear();
          nodes_todo.clear();
        }
      }
    }
  }
  None
}

#[must_use]
fn flush_todos(
  state: &mut State,
  labels_todo: &mut Vec<(Node, Node, bool)>,
  nodes_todo: &mut HashSet<Node>,
) -> Option<()> {
  loop {
    if let Some((a, b, label)) = labels_todo.pop() {
      state.unlabeled_edges.remove(&UnorderedPair(a, b));
      if matches!((a, b), (Border(_), Border(_))) {
        continue;
      }
      if let Some(prev_label) = state.edge_labels.insert(UnorderedPair(a, b), label) {
        if label == prev_label {
          continue;
        } else {
          return None;
        };
      }
      let a_connected_nodes = &state.div.connections[&a];
      let b_connected_nodes = &state.div.connections[&b];
      let (c0, c1) = a_connected_nodes.get_items_around(&b);
      let (d1, d0) = b_connected_nodes.get_items_around(&a);
      for (&c, &d) in [(c0, d0), (c1, d1)] {
        if c == d {
          let a_leg = state.edge_labels.get(&UnorderedPair(a, c));
          let b_leg = state.edge_labels.get(&UnorderedPair(b, c));
          match (label, a_leg, b_leg) {
            (_, None, None)
            | (false, Some(true), Some(true))
            | (true, Some(false), Some(true))
            | (true, Some(true), Some(false))
            | (true, Some(false), Some(false))
            | (false, Some(true), Some(false))
            | (false, Some(false), Some(true)) => {}
            (false, Some(false), Some(false)) | (true, Some(true), Some(true)) => return None,
            (true, Some(true), None) => labels_todo.push((b, c, false)),
            (false, Some(false), None) => labels_todo.push((b, c, true)),
            (_, Some(_), None) => state.ambiguous_edges.push(UnorderedPair(b, c)),
            (true, None, Some(true)) => labels_todo.push((a, c, false)),
            (false, None, Some(false)) => labels_todo.push((a, c, true)),
            (_, None, Some(_)) => state.ambiguous_edges.push(UnorderedPair(a, c)),
          }
        } else if state.div.connections[&c].contains_item(&d) {
          labels_todo.push((a, c, !label));
          labels_todo.push((b, d, !label));
          labels_todo.push((c, d, label));
        } else {
          return None;
        }
      }
      nodes_todo.insert(a);
      nodes_todo.insert(b);
    } else if let Some(node) = nodes_todo.drain().next() {
      if matches!(node, Border(_)) {
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
        if all_true.len() + all_none.len() == 2 {
          for &connected_node in &all_none {
            labels_todo.push((node, connected_node, true));
          }
          continue;
        }
      }
      if all_true.len() == 0 || all_false.len() == 0 {
        continue;
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
  pub true_vecs_count: u32,
  pub false_vecs_count: u32,
  pub none_vecs_count: u32,
}

pub fn classify_connected_nodes(
  node: Node,
  div: &Division,
  edge_labels: &EdgeLabels,
) -> ConnectedNodesClassification {
  let mut state = ConnectedNodesClassification::default();
  let connected_nodes = &div.connections[&node];
  for &connected_node in connected_nodes.iter() {
    let label = edge_labels
      .get(&UnorderedPair(node, connected_node))
      .cloned();
    match label {
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
