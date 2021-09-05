use crate::*;

pub struct Rect {
  pub x1: f64,
  pub y1: f64,
  pub x2: f64,
  pub y2: f64,
}

impl Rect {
  pub fn width(&self) -> f64 {
    self.x2 - self.x1
  }
  pub fn height(&self) -> f64 {
    self.y2 - self.y1
  }
}

pub type Layout = Vec<Rect>;

pub fn generate_layout(div: &Division, edge_labels: &EdgeLabels) -> Layout {
  let layout_x = generate_1d_layout(div, edge_labels, false);
  let layout_y = generate_1d_layout(div, edge_labels, true);

  return (0..div.regions)
    .map(|region| {
      let [x1, x2] = layout_x[&Region(region)];
      let [y1, y2] = layout_y[&Region(region)];
      debug_assert!(!x1.is_nan() && !x2.is_nan() && !y1.is_nan() && !y2.is_nan());
      Rect { x1, y1, x2, y2 }
    })
    .collect();

  fn generate_1d_layout(
    div: &Division,
    edge_labels: &EdgeLabels,
    axis: bool,
  ) -> HashMap<Node, [f64; 2]> {
    let root = if axis { 0 } else { 3 };
    let mut ranges = HashMap::new();
    ranges.insert(Border(root), [0.0_f64, 1.0_f64]);
    let mut node_queue = VecDeque::new();
    node_queue.push_back(Border(root));
    while let Some(node) = node_queue.pop_front() {
      let [start, end] = ranges[&node];
      let mut next_nodes = {
        let mut iter = classify_connected_nodes(node, div, edge_labels)
          .vecs
          .into_iter()
          .filter(|(vec, label)| {
            true
              && label == &Some(axis)
              && vec.iter().any(|node| {
                matches!(node, Region(_))
                  && match ranges.get(node) {
                    Some([a, b]) => a.is_nan() || b.is_nan(),
                    None => true,
                  }
              })
          });
        match iter.next() {
          Some(x) => {
            debug_assert_eq!(iter.next(), None);
            x.0
          }
          None => continue,
        }
      };
      next_nodes.retain(|node| matches!(node, Region(_)));
      let next_nodes_count = next_nodes.len();
      for (i, next_node) in next_nodes.into_iter().enumerate() {
        let first = i == 0;
        let last = i == next_nodes_count - 1;
        let range = ranges.entry(next_node).or_insert([f64::NAN, f64::NAN]);
        if range[0].is_nan()
          && (!first
            || edge_labels[&UnorderedPair(
              next_node,
              *div.connections[&next_node].get_item_after(&node).unwrap(),
            )] != axis)
        {
          let t = i as f64 / next_nodes_count as f64;
          range[0] = end * t + start * (1. - t);
        }
        if range[1].is_nan()
          && (!last
            || edge_labels[&UnorderedPair(
              next_node,
              *div.connections[&next_node].get_item_before(&node).unwrap(),
            )] != axis)
        {
          let t = (i + 1) as f64 / next_nodes_count as f64;
          range[1] = end * t + start * (1. - t);
        }
        node_queue.push_back(next_node);
      }
    }
    ranges
  }
}
