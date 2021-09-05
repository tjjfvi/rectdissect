use crate::*;
use std::fmt::Write;

pub fn generate_svg(
  layouts: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Layout>>,
) -> String {
  let max_row_width = 5;
  let square_size = 100.;
  let padding = 10.;
  let layouts = layouts.into_iter();
  let width = std::cmp::min(layouts.len(), max_row_width);
  let height = (layouts.len() + max_row_width - 1) / max_row_width;
  let mut str = format!(
    r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
    width as f64 * (square_size + padding) + padding,
    height as f64 * (square_size + padding) + padding
  );
  for (i, layout) in layouts.enumerate() {
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
  }
  str += "</svg>";
  str
}
