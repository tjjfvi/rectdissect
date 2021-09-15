use std::collections::{HashMap, VecDeque};

use crate::*;

#[derive(Debug, Clone, Copy)]
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
  let layout_x_0 = generate_1d_layout(div, edge_labels, 0);
  let layout_x_1 = generate_1d_layout(div, edge_labels, 2);
  let layout_y_0 = generate_1d_layout(div, edge_labels, 1);
  let layout_y_1 = generate_1d_layout(div, edge_labels, 3);

  debug_assert_eq!(layout_x_0.len() as u8, div.regions() + 1);
  debug_assert_eq!(layout_x_1.len() as u8, div.regions() + 1);
  debug_assert_eq!(layout_y_0.len() as u8, div.regions() + 1);
  debug_assert_eq!(layout_y_1.len() as u8, div.regions() + 1);

  return (0..div.regions())
    .map(|region| {
      let [x1_0, x2_0] = layout_x_0[&Node::region(region)];
      let [x2_1, x1_1] = layout_x_1[&Node::region(region)];
      let [y1_0, y2_0] = layout_y_0[&Node::region(region)];
      let [y2_1, y1_1] = layout_y_1[&Node::region(region)];
      debug_assert!(!x1_0.is_nan() && !x2_0.is_nan() && !y1_0.is_nan() && !y2_0.is_nan());
      debug_assert!(!x1_1.is_nan() && !x2_1.is_nan() && !y1_1.is_nan() && !y2_1.is_nan());
      Rect {
        x1: (x1_0 + x1_1) / 2.,
        x2: (x2_0 + x2_1) / 2.,
        y1: (y1_0 + y1_1) / 2.,
        y2: (y2_0 + y2_1) / 2.,
      }
    })
    .collect();

  fn generate_1d_layout(
    div: &Division,
    edge_labels: &EdgeLabels,
    root: u8,
  ) -> HashMap<Node, [f64; 2]> {
    let axis = root % 2 == 0;
    let mut ranges = HashMap::new();
    ranges.insert(
      Node::border(root),
      if root > 1 {
        [1.0_f64, 0.0_f64]
      } else {
        [0., 1.]
      },
    );
    let mut node_queue = VecDeque::new();
    node_queue.push_back((Node::border(root), Node::border(root)));
    while let Some((node, last)) = node_queue.pop_front() {
      let [start, end] = ranges[&node];
      let mut next_nodes = {
        let classification = classify_connected_nodes(node, div, edge_labels);
        if node.is_region() {
          debug_assert_eq!(
            (
              classification.vecs.len(),
              classification.true_vecs_count,
              classification.false_vecs_count,
              classification.all_none.len(),
              classification.all_true.len() == 0,
              classification.all_false.len() == 0,
            ),
            (4, 2, 2, 0, false, false)
          );
        }
        let mut iter = classification
          .vecs
          .into_iter()
          .filter(|(vec, label)| label == &Some(axis) && !vec.contains(&last));
        match iter.next() {
          Some(x) => {
            debug_assert_eq!(iter.next(), None);
            x.0
          }
          None => continue,
        }
      };
      next_nodes.retain(Node::is_region);
      let next_nodes_count = next_nodes.len();
      for (i, next_node) in next_nodes.into_iter().enumerate() {
        let first = i == 0;
        let last = i == next_nodes_count - 1;
        let range = ranges.entry(next_node).or_insert([f64::NAN, f64::NAN]);
        if range[0].is_nan()
          && (!first
            || edge_labels[&UnorderedPair(next_node, div[next_node].get_item_after(node))] != axis)
        {
          let t = i as f64 / next_nodes_count as f64;
          range[0] = end * t + start * (1. - t);
        }
        if range[1].is_nan()
          && (!last
            || edge_labels[&UnorderedPair(next_node, div[next_node].get_item_before(node))] != axis)
        {
          let t = (i + 1) as f64 / next_nodes_count as f64;
          range[1] = end * t + start * (1. - t);
        }
        node_queue.push_back((next_node, node));
      }
    }
    ranges
  }
}
