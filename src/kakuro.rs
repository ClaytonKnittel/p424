use std::{
  fmt,
  fs::File,
  io::{self, BufRead, BufReader},
  iter,
};

use crate::parenthesis_split::ParenthesesAwareSplit;

#[derive(Clone)]
pub struct TotalTile {
  horizontal: Option<String>,
  vertical: Option<String>,
}

impl TotalTile {
  fn map_horizontal<F, V>(&self, callback: F) -> Option<V>
  where
    F: FnOnce(String) -> V,
  {
    if let TotalTile {
      horizontal: Some(horizontal),
      vertical: _,
    } = self
    {
      Some(callback(horizontal.clone()))
    } else {
      None
    }
  }

  fn map_vertical<F, V>(&self, callback: F) -> Option<V>
  where
    F: FnOnce(String) -> V,
  {
    if let TotalTile {
      horizontal: _,
      vertical: Some(vertical),
    } = self
    {
      Some(callback(vertical.clone()))
    } else {
      None
    }
  }
}

#[derive(Clone)]
pub enum UnknownTile {
  Blank,
  Prefilled { hint: char },
}

#[derive(Clone)]
pub enum Tile {
  Empty,
  Unknown(UnknownTile),
  Total(TotalTile),
}

impl Tile {
  fn map_total<F, V>(&self, callback: F) -> Option<V>
  where
    F: FnOnce(TotalTile) -> V,
  {
    if let Tile::Total(total) = self {
      Some(callback(total.clone()))
    } else {
      None
    }
  }
}

pub struct Kakuro {
  n: usize,
  tiles: Vec<Tile>,
}

impl Kakuro {
  pub fn from_file() -> io::Result<Vec<Kakuro>> {
    let f = File::open("p424_kakuro200.txt")?;
    let f = BufReader::new(f);

    let mut grids: Vec<Kakuro> = Vec::new();
    let mut sizes: Vec<usize> = Vec::new();
    for line in f.lines() {
      let line_str = line?;
      let parts: Vec<&str> = line_str.split_paren().collect();
      let n: usize = parts[0].parse::<usize>().unwrap();
      sizes.push(n);
      let mut grid = Vec::new();
      for i in 0..n {
        for j in 0..n {
          let idx: usize = i * n + j + 1;
          let part: &str = parts[idx];
          if part == "X" {
            grid.push(Tile::Empty);
          } else if part == "O" {
            grid.push(Tile::Unknown(UnknownTile::Blank));
          } else if ("A"..="J").contains(&part) {
            grid.push(Tile::Unknown(UnknownTile::Prefilled {
              hint: part.chars().next().unwrap(),
            }));
          } else if part.starts_with('(') {
            let sum_rules: Vec<&str> = part[1..part.len()].split(',').collect();
            let mut vert_val: Option<String> = None;
            let mut hori_val: Option<String> = None;
            for rule in sum_rules {
              if rule.starts_with('v') {
                vert_val = Some(part[1..rule.len()].to_string());
              } else if rule.starts_with('v') {
                hori_val = Some(part[1..rule.len()].to_string());
              }
            }
            grid.push(Tile::Total(TotalTile {
              vertical: vert_val,
              horizontal: hori_val,
            }))
          }
        }
      }
      grids.push(Kakuro { tiles: grid, n });
    }
    Ok(grids)
  }

  fn take_unknowns(
    &self,
    row: usize,
    col: usize,
    vertical: bool,
  ) -> impl Iterator<Item = UnknownTile> + '_ {
    let idx = if vertical { row } else { col };
    let step = if vertical { self.n } else { 1 };
    ((idx + 1)..self.n).map_while(move |idx| {
      if let Tile::Unknown(unknown) = self.tiles.get(row * self.n + col + idx * step).unwrap() {
        Some(unknown.clone())
      } else {
        None
      }
    })
  }

  pub fn enumerate_lines(
    &self,
  ) -> impl Iterator<Item = (String, impl Iterator<Item = UnknownTile> + '_)> + '_ {
    (0..self.n).flat_map(move |row| {
      (0..self.n)
        .filter_map(move |col| {
          self
            .tiles
            .get(row * self.n + col)
            .unwrap()
            .map_total(|total| {
              total
                .map_horizontal(|horizontal_clue| {
                  iter::once(Some((
                    horizontal_clue.clone(),
                    self.take_unknowns(row, col, false),
                  )))
                })
                .unwrap_or(iter::once(None))
                .flatten()
                .chain(
                  total
                    .map_vertical(|vertical_clue| {
                      iter::once(Some((
                        vertical_clue.clone(),
                        self.take_unknowns(row, col, true),
                      )))
                    })
                    .unwrap_or(iter::once(None))
                    .flatten(),
                )
            })
        })
        .flatten()
    })
  }
}

impl fmt::Display for Kakuro {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut output = String::new();
    self.tiles.iter().enumerate().for_each(|(idx, tile)| {
      let tile_repr = match tile {
        Tile::Empty => {
          format!("{:10}", "X")
        }
        Tile::Unknown(UnknownTile::Blank) => {
          format!("{:10}", "_")
        }
        Tile::Unknown(UnknownTile::Prefilled { hint }) => {
          format!("{:10}", hint)
        }
        Tile::Total(TotalTile {
          horizontal,
          vertical,
        }) => {
          let horizontal_str = match horizontal {
            Some(x) => x.to_string(),
            None => "".to_string(),
          };
          let vertical_str = match vertical {
            Some(x) => x.to_string(),
            None => "".to_string(),
          };
          format!("{:10}", [horizontal_str, vertical_str].join(","))
        }
      };
      output.push_str(&tile_repr);
      if idx % self.n == self.n - 1 {
        output.push('\n');
      }
    });
    write!(f, "{}", output)
  }
}
