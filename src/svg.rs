use crate::*;
use std::{collections::HashSet, fmt::Write};

pub fn generate_svg(
  divs: CHashMap<u64, Division>,
  oeis_mode: bool,
  oeis_count: CHashMap<u64, ()>,
) -> String {
  let count = if oeis_mode {
    oeis_count.len()
  } else {
    divs.len()
  };
  let max_row_width = 5;
  let square_size = 100.;
  let padding = 10.;
  let width = std::cmp::min(count, max_row_width);
  let height = (count + max_row_width - 1) / max_row_width;
  let mut str = format!(
    r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" style="height: auto">"#,
    width as f64 * (square_size + padding) + padding,
    height as f64 * (square_size + padding) + padding
  );
  let mut i = 0;
  for (_, div) in divs.into_iter() {
    let mut hashes = HashSet::new();
    for edge_labels in label_edges(&div) {
      let edge_labels_hash = hash_division(&div, Some(&edge_labels));
      if !hashes.insert(edge_labels_hash) {
        continue;
      }
      write!(str, r#"<g id="{:?}">"#, edge_labels_hash).unwrap();
      let layout = generate_layout(&div, &edge_labels);
      for rect in layout {
        let x = i % max_row_width;
        let y = i / max_row_width;
        write!(
          str,
          r#"<rect x="{}" width="{}" y="{}" height="{}" stroke="black" stroke-width="2" fill="none"/>"#,
          rect.x1 * square_size + x as f64 * (square_size + padding) + padding,
          rect.width() * square_size,
          rect.y1 * square_size + y as f64 * (square_size + padding) + padding,
          rect.height() * square_size,
        ).unwrap();
      }
      write!(str, r#"</g>"#).unwrap();
      i += 1;
      if !oeis_mode {
        break;
      }
    }
  }
  str += "</svg>";
  str
}
