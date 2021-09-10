use crate::*;

pub fn divide<'a>(div: &'a Division) -> impl Iterator<Item = Division> + 'a {
  (0..div.regions()).flat_map(move |region| {
    let connected_nodes = &div[Node::region(region)];
    connected_nodes
      .iter()
      .enumerate()
      .flat_map(move |(cut_0_ind, cut_0)| {
        connected_nodes
          .iter()
          .enumerate()
          .take(connected_nodes.len() as usize + cut_0_ind - 1)
          .skip(cut_0_ind + 2)
          .flat_map(move |(cut_1_ind, cut_1)| {
            let must_share_0 = cut_1_ind - cut_0_ind < 3
              || cut_0.is_border()
              || connected_nodes.get_item_after(cut_0).is_border();
            let must_share_1 = cut_0_ind + connected_nodes.len() as usize - cut_1_ind < 3
              || cut_1.is_border()
              || connected_nodes.get_item_after(cut_1).is_border();
            [true, false]
              .iter()
              .flat_map(move |&share_0| {
                if must_share_0 && !share_0 {
                  return None;
                }
                Some(
                  [true, false]
                    .iter()
                    .filter_map::<Division, _>(move |&share_1| {
                      if must_share_1 && !share_1 {
                        return None;
                      }
                      return Some(_divide(
                        div,
                        connected_nodes,
                        region,
                        cut_0_ind,
                        cut_0,
                        cut_1_ind,
                        cut_1,
                        share_0,
                        share_1,
                      ));
                    }),
                )
              })
              .flatten()
          })
      })
  })
}

fn _divide(
  div: &Division,
  connected_nodes: &ConnectedNodes,
  region: u8,
  cut_0_ind: usize,
  cut_0: Node,
  cut_1_ind: usize,
  cut_1: Node,
  share_0: bool,
  share_1: bool,
) -> Division {
  let new_region = div.regions();
  let expand = false
    || (share_0 && div[cut_0].len() == div.max_connections())
    || (share_1 && div[cut_1].len() == div.max_connections());
  unsafe {
    let mut new_div = Division::new_raw(div.regions() + 1, div.max_connections() + expand as u8);
    for i in 0..(new_div.regions() + 4) {
      let node = Node(i);
      let old_order = &div[if node == Node::region(new_region) {
        Node::region(region)
      } else {
        node
      }];
      let order = &mut new_div[node];
      std::ptr::copy_nonoverlapping::<u8>(
        old_order as *const _ as _,
        order as *mut _ as _,
        (div.max_connections() + 1) as usize,
      );
      if node == Node::region(region) {
        order.delete_items_between(
          cut_0,
          if share_1 {
            cut_1
          } else {
            order.get_item_after(cut_1)
          },
        );
        order.insert_item_after(cut_0, Node::region(new_region));
      } else if node == Node::region(new_region) {
        // dbg!(&order);
        order.delete_items_between(
          cut_1,
          if share_0 {
            cut_0
          } else {
            order.get_item_after(cut_0)
          },
        );
        order.insert_item_after(cut_1, Node::region(region));
      }
    }
    for (i, node) in connected_nodes.iter().enumerate() {
      let order = &mut new_div[node];
      if i == cut_0_ind && share_0 || i == cut_1_ind && share_1 {
        order.insert_item_after(
          if i == cut_0_ind {
            order.get_item_before(Node::region(region))
          } else {
            Node::region(region)
          },
          Node::region(new_region),
        );
      } else if i > cut_0_ind && i <= cut_1_ind {
        order.replace_item(Node::region(region), Node::region(new_region));
      }
    }
    new_div
  }
}
