use crate::*;

pub type EdgeLabels = HashMap<UnorderedPair<Node>, bool>;

pub fn label_edges(div: &Division) -> Option<EdgeLabels> {
  // dbg!(div);
  #[derive(Clone, Debug)]
  struct State<'a> {
    edge_labels: EdgeLabels,
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
    for node in div.connections[&Border(border_n)].iter() {
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
    let a_connected_nodes = &state.div.connections[&a];
    let b_connected_nodes = &state.div.connections[&b];
    let (c0, c1) = a_connected_nodes.get_items_around(&b);
    let (d1, d0) = b_connected_nodes.get_items_around(&a);
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
