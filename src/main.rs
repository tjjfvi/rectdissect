use std::{collections::HashMap, ops::Neg};

#[derive(Default, Clone, Debug)]
struct Connections(HashMap<u32, HashMap<u32, Dir>>);

impl Connections {
  pub fn new() -> Connections {
    <_>::default()
  }
  pub fn add(&mut self, a: u32, b: u32, dir: Dir) {
    self.0.entry(a).or_insert_with(<_>::default).insert(b, dir);
    self.0.entry(b).or_insert_with(<_>::default).insert(a, -dir);
  }
  pub fn remove(&mut self, a: u32, b: u32) {
    self.0.entry(a).and_modify(|x| {
      x.remove(&b);
    });
    self.0.entry(b).and_modify(|x| {
      x.remove(&a);
    });
  }
}

#[derive(Default, Clone, Debug)]
struct Division {
  regions: u32,
  connections: Connections,
}

struct DivideIter {
  division: Division,
  region_ind: u32,
  dir: u8,
  cut_ind_0: u32,
  cut_ind_1: u32,
}

impl DivideIter {
  pub fn new(division: Division) -> DivideIter {
    DivideIter {
      division,
      region_ind: 0,
      dir: 0,
      cut_ind_0: 0,
      cut_ind_1: 0,
    }
  }
}

impl Iterator for DivideIter {
  type Item = Division;
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.region_ind >= self.division.regions {
        return None;
      }
      let edges = self.division.connections.0.get(&self.region_ind).unwrap();
      if self.dir >= 2 {
        self.dir = 0;
        self.region_ind += 1;
        continue;
      }
      let dir_0 = (self.dir, 0).into();
      let count_0 = edges.iter().filter(|x| *x.1 == dir_0).count() as u32;
      let max_cut_ind_0 = if count_0 == 0 { 1 } else { count_0 * 2 - 1 };
      if self.cut_ind_0 >= max_cut_ind_0 {
        self.cut_ind_0 = 0;
        self.dir += 1;
        continue;
      }
      let dir_1 = (self.dir, 0).into();
      let count_1 = edges.iter().filter(|x| *x.1 == dir_1).count() as u32;
      let max_cut_ind_1 = if count_1 == 0 { 1 } else { count_1 * 2 - 1 };
      if self.cut_ind_1 >= max_cut_ind_1 {
        self.cut_ind_1 = 0;
        self.cut_ind_0 += 1;
        continue;
      }

      let chunk


      self.cut_ind_1 += 1;
    }
  }
}

fn main() {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Dir {
  N,
  S,
  E,
  W,
}

impl Neg for Dir {
  type Output = Dir;
  fn neg(self) -> Self::Output {
    match self {
      Dir::N => Dir::S,
      Dir::S => Dir::N,
      Dir::E => Dir::W,
      Dir::W => Dir::E,
    }
  }
}

impl From<(u8, u8)> for Dir {
  fn from((axis, pos): (u8, u8)) -> Self {
    match (axis, pos) {
      (0, 0) => Dir::W,
      (0, 1) => Dir::E,
      (1, 0) => Dir::N,
      (1, 1) => Dir::S,
      _ => panic!("Invalid (axis, pos) pair"),
    }
  }
}
