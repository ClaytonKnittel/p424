use std::{
  collections::HashSet,
  fmt::{self, Display},
  fs::File,
  io::{self, BufRead, BufReader},
  iter,
  ops::ControlFlow,
};

use itertools::Itertools;

use crate::{
  dlx::{ColorItem, Constraint, Dlx, HeaderType},
  linear_solver::LinearSolver,
  parenthesis_split::ParenthesesAwareSplit,
};

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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum DlxItem {
  Sum { idx: u32, vertical: bool },
  Tile { idx: u32 },
  Letter { letter: char },
  LetterValue { value: u32 },
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
  ) -> impl Iterator<Item = DlxItem> + '_ {
    let idx = if vertical { row } else { col };
    let step = if vertical { self.n } else { 1 };
    (1..(self.n - idx)).map_while(move |idx| {
      let idx = self.get_idx(row, col) + idx * step;
      match self.tiles.get(idx) {
        Some(Tile::Unknown(UnknownTile::Blank)) => Some(DlxItem::Tile { idx: idx as u32 }),
        Some(Tile::Unknown(UnknownTile::Prefilled { hint })) => {
          Some(DlxItem::Letter { letter: *hint })
        }
        _ => None,
      }
    })
  }

  fn enumerate_lines(
    &self,
  ) -> impl Iterator<Item = ((DlxItem, TotalClue), impl Iterator<Item = DlxItem> + '_)> + '_ {
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
                    (
                      DlxItem::Sum {
                        idx: self.get_idx(row, col) as u32,
                        vertical: false,
                      },
                      horizontal_clue,
                    ),
                    self.take_unknowns(row, col, false),
                  )))
                })
                .unwrap_or(iter::once(None))
                .flatten()
                .chain(
                  total
                    .map_vertical(|vertical_clue| {
                      iter::once(Some((
                        (
                          DlxItem::Sum {
                            idx: self.get_idx(row, col) as u32,
                            vertical: true,
                          },
                          vertical_clue,
                        ),
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

  fn all_items(&self) -> impl Iterator<Item = (DlxItem, HeaderType)> + '_ {
    self
      .tiles
      .iter()
      .enumerate()
      .flat_map(|(idx, tile)| {
        let idx = idx as u32;
        match tile {
          Tile::Total(TotalTile {
            horizontal,
            vertical,
          }) => [
            horizontal.as_ref().map(|_| {
              (
                DlxItem::Sum {
                  idx,
                  vertical: false,
                },
                HeaderType::Primary,
              )
            }),
            vertical.as_ref().map(|_| {
              (
                DlxItem::Sum {
                  idx,
                  vertical: true,
                },
                HeaderType::Primary,
              )
            }),
          ],
          Tile::Empty => [Some((DlxItem::Tile { idx }, HeaderType::Secondary)), None],
          _ => [None, None],
        }
        .into_iter()
        .flatten()
      })
      .chain(('A'..='H').enumerate().flat_map(|(value, letter)| {
        [
          (DlxItem::Letter { letter }, HeaderType::Secondary),
          (
            DlxItem::LetterValue {
              value: value as u32,
            },
            HeaderType::Secondary,
          ),
        ]
        .into_iter()
      }))
  }

  fn construct_dlx(
    clue_item: DlxItem,
    items: Vec<(DlxItem, u32)>,
  ) -> Option<impl Iterator<Item = Constraint<DlxItem>>> {
    let values =
      match items
        .iter()
        .try_fold([(); 10].map(|_| None), |mut values_array, (item, value)| {
          let value = *value;
          match item {
            DlxItem::Letter { .. } => {
              if values_array[value as usize].is_some() {
                ControlFlow::Break(())
              } else {
                values_array[value as usize] = Some(DlxItem::LetterValue { value });
                ControlFlow::Continue(values_array)
              }
            }
            _ => ControlFlow::Continue(values_array),
          }
        }) {
        ControlFlow::Break(_) => {
          return None;
        }
        ControlFlow::Continue(values_array) => values_array,
      };

    Some(
      iter::once(clue_item.into())
        .chain(
          items
            .into_iter()
            .map(|(item, color)| ColorItem::new(item, color).into()),
        )
        .chain(
          values
            .into_iter()
            .enumerate()
            .filter_map(|(idx, item)| item.map(|item| ColorItem::new(item, idx as u32).into())),
        ),
    )
  }

  pub fn solve(&self) {
    for line in self.enumerate_lines() {
      println!(
        "Line: {}: {}",
        line.0 .1,
        line
          .1
          .map(|item| format!("{item:?}"))
          .collect::<Vec<_>>()
          .join(", "),
      );
    }

    let items = self.all_items();

    let choices = self.enumerate_lines().flat_map(|((item, clue), items)| {
      let mut solver = LinearSolver::new();
      match clue {
        TotalClue::OneDigit(letter) => {
          solver.add(DlxItem::Letter { letter }, -1);
        }
        TotalClue::TwoDigit { ones, tens } => {
          solver.add(DlxItem::Letter { letter: ones }, -1);
          solver.add(DlxItem::Letter { letter: tens }, -10);
        }
      }
      for item in items {
        solver.add(item, 1);
      }

      solver
        .find_all_solutions_owned()
        .map(Itertools::collect_vec)
        .flat_map(move |solution| {
          Self::construct_dlx(item.clone(), solution).map(|subset| (0, subset))
        })
    });

    let mut dlx = Dlx::new(items, choices);
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
