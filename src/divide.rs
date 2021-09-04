use crate::*;

pub fn divide(div: Division, mut cb: impl FnMut(Division)) {
  for region in 0..div.regions {
    let connected_nodes = &div.connections[&Region(region)];
    for (cut_0_ind, cut_0) in connected_nodes.iter().enumerate() {
      for (cut_1_ind, cut_1) in connected_nodes
        .iter()
        .enumerate()
        .take(connected_nodes.len() + cut_0_ind - 1)
        .skip(cut_0_ind + 2)
      {
        let must_share_0 = cut_1_ind - cut_0_ind < 3;
        let must_share_1 = cut_0_ind + connected_nodes.len() - cut_1_ind < 3;
        for share_0 in [true, false] {
          if must_share_0 && !share_0 {
            continue;
          }
          for share_1 in [true, false] {
            if must_share_1 && !share_1 {
              continue;
            }
            let new_region = div.regions;
            let mut new_connections = div.connections.clone();
            {
              let order = new_connections.get_mut(&Region(region)).unwrap();
              order
                .delete_items_between(
                  cut_0,
                  &(if share_1 {
                    *cut_1
                  } else {
                    *order.get_item_after(cut_1).unwrap()
                  }),
                )
                .unwrap();
              order
                .insert_items_after(cut_0, [Region(new_region)])
                .unwrap();
            }
            {
              let mut order = connected_nodes.clone();
              order
                .delete_items_between(
                  cut_1,
                  &(if share_0 {
                    *cut_0
                  } else {
                    *order.get_item_after(cut_0).unwrap()
                  }),
                )
                .unwrap();
              order.insert_items_after(cut_1, [Region(region)]).unwrap();
              new_connections.insert(Region(new_region), order);
            }
            for (i, node) in connected_nodes.iter().enumerate() {
              if i == cut_0_ind && share_0 || i == cut_1_ind && share_1 {
                let order = new_connections.get_mut(node).unwrap();
                order
                  .insert_items_after(
                    &(if i == cut_0_ind {
                      *order.get_item_before(&Region(region)).unwrap()
                    } else {
                      Region(region)
                    }),
                    [Region(new_region)],
                  )
                  .unwrap();
              } else if i > cut_0_ind && i <= cut_1_ind {
                let order = new_connections.get_mut(node).unwrap();
                let before = order.get_item_before(&Region(region)).unwrap().clone();
                order
                  .delete_items_between(
                    &before,
                    &order.get_item_after(&Region(region)).unwrap().clone(),
                  )
                  .unwrap();
                order
                  .insert_items_after(&before, [Region(new_region)])
                  .unwrap();
              }
            }
            let div = Division {
              regions: div.regions + 1,
              connections: new_connections,
            };
            cb(div);
          }
        }
      }
    }
  }
}
