use crate::*;
use std::{collections::HashMap, fmt::Write};

pub fn generate_svg(divs: &HashMap<u64, Division>) -> String {
  let max_row_width = 5;
  let square_size = 100.;
  let padding = 10.;
  let width = std::cmp::min(divs.len(), max_row_width);
  let height = (divs.len() + max_row_width - 1) / max_row_width;
  let mut str = format!(
    r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" style="height: auto">"#,
    width as f64 * (square_size + padding) + padding,
    height as f64 * (square_size + padding) + padding
  );
  for (i, (hash, div)) in divs.iter().enumerate() {
    write!(str, r#"<g id="{}">"#, hash).unwrap();
    let layout = generate_layout(div, &label_edges(div).unwrap());
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
  }
  str += "</svg>";
  str
}
