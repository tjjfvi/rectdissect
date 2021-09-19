use std::{
  alloc,
  fmt::Debug,
  ops::{Index, IndexMut},
  ptr::{self, NonNull},
};

use crate::*;

pub struct Division {
  regions: u8,
  max_connections: u8,
  ptr: NonNull<u8>,
}

unsafe impl Send for Division {}
unsafe impl Sync for Division {}

impl Division {
  pub unsafe fn new_raw(regions: u8, max_connections: u8) -> Division {
    let size = Division::data_size(regions, max_connections);
    let layout = alloc::Layout::array::<u8>(size).unwrap();
    let ptr = alloc::alloc(layout);
    let ptr = match NonNull::new(ptr) {
      Some(p) => p,
      None => alloc::handle_alloc_error(layout),
    };
    Division {
      regions,
      max_connections,
      ptr,
    }
  }
  pub unsafe fn from_data(regions: u8, max_connections: u8, data: &[u8]) -> Division {
    assert_eq!(data.len(), Division::data_size(regions, max_connections));
    let division = Division::new_raw(regions, max_connections);
    ptr::copy(data.as_ptr(), division.ptr.as_ptr(), data.len());
    division
  }
  pub const fn data_size(regions: u8, max_connections: u8) -> usize {
    (regions + 4) as usize * (max_connections + 1) as usize
  }
  pub fn num_regions(&self) -> u8 {
    self.regions
  }
  pub fn regions(&self) -> std::iter::Map<std::ops::Range<u8>, fn(u8) -> Node> {
    (0..self.regions).map(Node::region)
  }
  pub fn max_connections(&self) -> u8 {
    self.max_connections
  }
  pub fn nodes(&self) -> std::iter::Map<std::ops::Range<u8>, fn(u8) -> Node> {
    (0..self.regions + 4).map(Node)
  }
}

impl Default for Division {
  fn default() -> Division {
    division! {
      b0: [b1, r0, b3],
      b1: [b2, r0, b0],
      b2: [b3, r0, b1],
      b3: [b0, r0, b2],
      r0: [b0, b1, b2, b3],
    }
  }
}

impl Index<Node> for Division {
  type Output = ConnectedNodes;
  fn index(&self, node: Node) -> &Self::Output {
    assert!(node.0 < self.regions + 4);
    unsafe {
      &*(self
        .ptr
        .as_ptr()
        .offset(node.0 as isize * (self.max_connections + 1) as isize)
        as *const ConnectedNodes)
    }
  }
}

impl IndexMut<Node> for Division {
  fn index_mut(&mut self, node: Node) -> &mut Self::Output {
    assert!(node.0 < self.regions + 4);
    unsafe {
      &mut *(self
        .ptr
        .as_ptr()
        .offset(node.0 as isize * (self.max_connections + 1) as isize)
        as *mut ConnectedNodes)
    }
  }
}

impl Debug for Division {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug_map = f.debug_map();
    for node in self.nodes() {
      debug_map.entry(&node, &self[node]);
    }
    debug_map.finish()
  }
}

impl Drop for Division {
  fn drop(&mut self) {
    let size = Division::data_size(self.regions, self.max_connections);
    let layout = alloc::Layout::array::<u8>(size).unwrap();
    unsafe { alloc::dealloc(self.ptr.as_ptr(), layout) }
  }
}

#[macro_export]
macro_rules! division {

  ( $( $key:ident : [ $( $item:ident ),+ $(,)? ] ),+ $(,)? ) => {{
    const REGIONS: u8 = division!(_ regions $([$($item)+])+);
    const MAX_CONNECTIONS: u8 = division!(_ max_connections $([$($item)+])+);
    const SIZE: usize = $crate::Division::data_size(REGIONS, MAX_CONNECTIONS);
    const DATA: [u8; SIZE] = {
      let mut data = [0; SIZE];
      division!( _ generate_data data MAX_CONNECTIONS $($key [$($item)*])*);
      data
    };
    unsafe { Division::from_data(REGIONS, MAX_CONNECTIONS, &DATA) }
  }};

  ( _ max_connections $([])* ) => { 0 };
  ( _ max_connections $([$($h:tt $($t:tt)*)?])*) => {
    1 + division!( _ max_connections $([$($($t)*)?])* )
  };

  ( _ count ) => { 0 };
  ( _ count $x:tt $($y:tt)* ) => {
    1 + division!( _ count $($y)* )
  };

  ( _ regions $($x:tt)* ) => {
    division!( _ count $($x)* ) - 4
  };

  ( _ generate_data $data:ident $max_connections:ident ) => {};
  ( _ generate_data $data:ident $max_connections:ident $key:ident [$($item:tt)*] $($rest:tt)* ) => {
    let start = ($crate::Node::$key.0 as usize) * ($max_connections + 1) as usize;
    $data[start] = division!( _ count $($item)* );
    division!( _ generate_data_row $data (start + 1) $($item)* );
    division!( _ generate_data $data $max_connections $($rest)* );
  };

  ( _ generate_data_row $data:ident $offset:tt ) => {};
  ( _ generate_data_row $data:ident $offset:tt $item:ident $($rest:tt)* ) => {
    $data[$offset] = $crate::Node::$item.0;
    division!( _ generate_data_row $data ($offset + 1) $($rest)* );
  };

}
