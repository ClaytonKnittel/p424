use std::{
  collections::HashSet,
  fmt::{self, Display},
  fs::File,
  io::{self, BufRead, BufReader},
  iter,
};

use crate::parenthesis_split::ParenthesesAwareSplit;

#[derive(Clone)]
pub enum TotalClue {
  OneDigit(char),
  TwoDigit { ones: char, tens: char },
}

impl TotalClue {
  fn new(clue: &str) -> TotalClue {
    if clue.len() == 1 {
      TotalClue::OneDigit(clue.chars().next().unwrap())
    } else if clue.len() == 2 {
      let mut chars = clue.chars();
      TotalClue::TwoDigit {
        tens: chars.next().unwrap(),
        ones: chars.next().unwrap(),
      }
    } else {
      unreachable!("Tried to construct clue with wrong number of digits: \"{clue}\"")
    }
  }
}

impl Display for TotalClue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TotalClue::OneDigit(digit) => write!(f, "{digit}"),
      TotalClue::TwoDigit { ones, tens } => write!(f, "{tens}{ones}"),
    }
  }
}

#[derive(Clone)]
pub struct TotalTile {
  horizontal: Option<TotalClue>,
  vertical: Option<TotalClue>,
}

impl TotalTile {
  fn map_horizontal<F, V>(&self, callback: F) -> Option<V>
  where
    F: FnOnce(TotalClue) -> V,
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
    F: FnOnce(TotalClue) -> V,
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

impl fmt::Display for UnknownTile {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      UnknownTile::Blank => "_".fmt(f),
      UnknownTile::Prefilled { hint } => hint.fmt(f),
    }
  }
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

impl fmt::Display for Tile {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Tile::Empty => "X".fmt(f),
      Tile::Unknown(unknown_tile) => unknown_tile.fmt(f),
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
        [vertical_str, horizontal_str].join(",").fmt(f)
      }
    }
  }
}

pub struct Kakuro {
  n: usize,
  tiles: Vec<Tile>,
}

impl Kakuro {
  pub fn from_file(path: &str) -> io::Result<Vec<Kakuro>> {
    let f = File::open(path)?;
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
          } else if let Some(line) = part
            .strip_prefix('(')
            .and_then(|line| line.strip_suffix(')'))
          {
            let total_tile = line.split(',').fold(
              TotalTile {
                vertical: None,
                horizontal: None,
              },
              |total_tile, rule| {
                if let Some(vert) = rule.strip_prefix('v') {
                  TotalTile {
                    vertical: Some(TotalClue::new(vert)),
                    ..total_tile
                  }
                } else if let Some(hori) = rule.strip_prefix('h') {
                  TotalTile {
                    horizontal: Some(TotalClue::new(hori)),
                    ..total_tile
                  }
                } else {
                  total_tile
                }
              },
            );
            grid.push(Tile::Total(total_tile));
          }
        }
      }
      grids.push(Kakuro { tiles: grid, n });
    }
    Ok(grids)
  }

  fn get_idx(&self, row: usize, col: usize) -> usize {
    row * self.n + col
  }

  fn take_unknowns(
    &self,
    row: usize,
    col: usize,
    vertical: bool,
  ) -> impl Iterator<Item = (usize, UnknownTile)> + '_ {
    let idx = if vertical { row } else { col };
    let step = if vertical { self.n } else { 1 };
    (1..(self.n - idx)).map_while(move |idx| {
      let idx = self.get_idx(row, col) + idx * step;
      if let Tile::Unknown(unknown) = self.tiles.get(idx).unwrap() {
        Some((idx, unknown.clone()))
      } else {
        None
      }
    })
  }

  pub fn enumerate_lines(
    &self,
  ) -> impl Iterator<
    Item = (
      (usize, TotalClue),
      impl Iterator<Item = (usize, UnknownTile)> + '_,
    ),
  > + '_ {
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
                    (self.get_idx(row, col), horizontal_clue),
                    self.take_unknowns(row, col, false),
                  )))
                })
                .unwrap_or(iter::once(None))
                .flatten()
                .chain(
                  total
                    .map_vertical(|vertical_clue| {
                      iter::once(Some((
                        (self.get_idx(row, col), vertical_clue),
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

  fn solve(&self) {
    enum Item {
      Sum { idx: u32 },
      Tile { idx: u32 },
      Letter { letter: char },
    }
    let mut items = HashSet::new();

    for ((clue_idx, clue), tiles) in self.enumerate_lines() {
      for (tile_idx, tile) in tiles {}
    }
  }
}

impl fmt::Display for Kakuro {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.tiles.iter().enumerate().try_for_each(|(idx, tile)| {
      write!(f, "{:10}", tile)?;
      if (idx + 1) % self.n == 0 {
        writeln!(f)?;
      }
      Ok(())
    })
  }
}
