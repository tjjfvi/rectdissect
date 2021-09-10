use crate::*;

pub fn divide<'a>(div: &'a Division) -> impl Iterator<Item = Division> + 'a {
  (0..div.regions).flat_map(move |region| {
    let connected_nodes = (&div.connections).get(&Node::region(region)).unwrap();
    connected_nodes
      .iter()
      .enumerate()
      .flat_map(move |(cut_0_ind, cut_0)| {
        connected_nodes
          .iter()
          .enumerate()
          .take(connected_nodes.len() + cut_0_ind - 1)
          .skip(cut_0_ind + 2)
          .flat_map(move |(cut_1_ind, cut_1)| {
            let must_share_0 = cut_1_ind - cut_0_ind < 3
              || cut_0.is_border()
              || connected_nodes.get_item_after(&cut_0).is_border();
            let must_share_1 = cut_0_ind + connected_nodes.len() - cut_1_ind < 3
              || cut_1.is_border()
              || connected_nodes.get_item_after(&cut_1).is_border();
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
  connected_nodes: &CircularOrder<Node>,
  region: u8,
  cut_0_ind: usize,
  cut_0: &Node,
  cut_1_ind: usize,
  cut_1: &Node,
  share_0: bool,
  share_1: bool,
) -> Division {
  let new_region = div.regions;
  let mut new_connections = div.connections.clone();
  {
    let order = new_connections.get_mut(&Node::region(region)).unwrap();
    order.delete_items_between(
      cut_0,
      &(if share_1 {
        *cut_1
      } else {
        *order.get_item_after(cut_1)
      }),
    );
    order.insert_items_after(cut_0, [Node::region(new_region)]);
  }
  {
    let mut order = connected_nodes.clone();
    order.delete_items_between(
      cut_1,
      &(if share_0 {
        *cut_0
      } else {
        *order.get_item_after(cut_0)
      }),
    );
    order.insert_items_after(cut_1, [Node::region(region)]);
    new_connections.insert(Node::region(new_region), order);
  }
  for (i, node) in connected_nodes.iter().enumerate() {
    if i == cut_0_ind && share_0 || i == cut_1_ind && share_1 {
      let order = new_connections.get_mut(node).unwrap();
      order.insert_items_after(
        &(if i == cut_0_ind {
          *order.get_item_before(&Node::region(region))
        } else {
          Node::region(region)
        }),
        [Node::region(new_region)],
      );
    } else if i > cut_0_ind && i <= cut_1_ind {
      let order = new_connections.get_mut(node).unwrap();
      let before = order.get_item_before(&Node::region(region)).clone();
      order.delete_items_between(
        &before,
        &order.get_item_after(&Node::region(region)).clone(),
      );
      order.insert_items_after(&before, [Node::region(new_region)]);
    }
  }
  Division {
    regions: div.regions + 1,
    connections: new_connections,
  }
}
