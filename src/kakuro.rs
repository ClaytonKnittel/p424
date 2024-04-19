use std::{
  collections::HashMap,
  fmt::{self, Display},
  fs::File,
  io::{self, BufRead, BufReader},
  iter,
  ops::ControlFlow,
};

use itertools::Itertools;

use crate::{
  dlx::{ColorItem, Constraint, Dlx, HeaderType},
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

  fn sum_range(&self) -> (u32, u32) {
    match self {
      TotalClue::OneDigit(_) => (0, 9),
      TotalClue::TwoDigit { .. } => (10, 45),
    }
  }

  pub fn all_combinations_for_range(
    (min, max): (u32, u32),
    num_tiles: u32,
  ) -> impl Iterator<Item = (u32, Vec<u32>)> {
    debug_assert!((1..=9).contains(&num_tiles));
    let mut choices = Vec::with_capacity(num_tiles as usize);

    // Slack is the amount of extra value we have to add above the min possible
    // permutation (1, 2, 3, 4, ...) to sum to `max`. Slack cannot fall below
    // 0, else the sum of numbers would be larger than `max`.
    let mut slack = max as i32 - (num_tiles * (num_tiles + 1) / 2) as i32;
    // Air is the amount of extra value we have to add above the min possible
    // permutation (1, 2, 3, 4, ...) to sum to `min`. Air must be <= 0, else
    // the sum of numbers would be less than `min`.
    let mut air = min as i32 - (num_tiles * (num_tiles + 1) / 2) as i32;

    {
      let max_extra_from_remainder =
        9 * (num_tiles - 1) - (num_tiles - 1) * (num_tiles.wrapping_sub(2)) / 2;
      let extra = (air.max(0) as u32).saturating_sub(max_extra_from_remainder);

      slack -= (extra * num_tiles) as i32;
      air -= (extra * num_tiles) as i32;
      choices.push(1 + extra);
    }

    iter::once(
      if choices.len() == num_tiles as usize && (air..=slack).contains(&0) {
        Some((*choices.first().unwrap(), choices.clone()))
      } else {
        None
      },
    )
    .chain(
      iter::repeat(()).scan((choices, slack, air), move |(choices, slack, air), _| {
        choices.pop().map(move |top| {
          let choices_len = choices.len() as u32;
          let remaining = num_tiles - choices_len;
          debug_assert_eq!(
            max as i32
              - (choices.iter().sum::<u32>() + top * remaining + remaining * (remaining - 1) / 2)
                as i32,
            *slack
          );
          debug_assert_eq!(
            min as i32
              - (choices.iter().sum::<u32>() + top * remaining + remaining * (remaining - 1) / 2)
                as i32,
            *air
          );

          if *slack < 0 || top == 11 - remaining {
            // Numbers got too big, time to abort.
            if let Some(choice) = choices.pop() {
              choices.push(choice + 1);
              let diff = (remaining * (top - choice - 1)) as i32 - (remaining as i32 + 1);
              *slack += diff;
              *air += diff;
            }
          } else if remaining == 1 {
            debug_assert!(*air <= 0);
            debug_assert!((min..=max).contains(&(choices.iter().sum::<u32>() + top)));

            choices.push(top + 1);
            *slack -= 1;
            *air -= 1;
          } else if *air > 0 {
            choices.push(top);
            let remaining = remaining - 1;

            let max_extra_from_remainder = (remaining - 1) * (9 - remaining - top);
            let extra = (*air as u32).saturating_sub(max_extra_from_remainder);
            choices.push(top + 1 + extra);
            *slack -= (extra * remaining) as i32;
            *air -= (extra * remaining) as i32;
          } else {
            choices.push(top);
            choices.push(top + 1);
          }

          if choices.len() == num_tiles as usize
            && choices.last().is_some_and(|&choice| choice < 10)
            && (*air..=*slack).contains(&0)
          {
            Some(((min as i32 - *air) as u32, choices.clone()))
          } else {
            None
          }
        })
      }),
    )
    .flatten()
  }

  fn all_combinations(
    &self,
    num_tiles: u32,
  ) -> impl Iterator<Item = (Vec<(DlxItem, u32)>, Vec<u32>)> {
    let (min, max) = self.sum_range();
    let self_copy = self.clone();
    Self::all_combinations_for_range((min, max), num_tiles).filter_map(
      move |(total, combination)| match self_copy {
        TotalClue::OneDigit(letter) => {
          Some((vec![(DlxItem::Letter { letter }, total)], combination))
        }
        TotalClue::TwoDigit { ones, tens } => {
          if (ones == tens) == (total % 11 == 0) {
            let ones_value = total % 10;
            let tens_value = total / 10;
            Some((
              vec![
                (DlxItem::Letter { letter: ones }, ones_value),
                (DlxItem::Letter { letter: tens }, tens_value),
              ],
              combination,
            ))
          } else {
            None
          }
        }
      },
    )
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

impl DlxItem {
  fn is_tile(&self) -> bool {
    matches!(self, DlxItem::Tile { .. })
  }
}

pub struct LetterAssignment {
  letters: [u32; 10],
}

impl LetterAssignment {
  fn new() -> Self {
    Self { letters: [10; 10] }
  }

  fn letter_idx(letter: char) -> usize {
    debug_assert!(('A'..='J').contains(&letter));
    letter as usize - 'A' as usize
  }

  pub fn letter_value(&self, letter: char) -> u32 {
    self.letters[Self::letter_idx(letter)]
  }

  fn set_value(&mut self, letter: char, value: u32) {
    debug_assert_eq!(self.letters[Self::letter_idx(letter)], 10);
    self.letters[Self::letter_idx(letter)] = value;
  }

  fn with_value(mut self, letter: char, value: u32) -> Self {
    self.set_value(letter, value);
    self
  }

  fn fill_remaining(&mut self) {
    debug_assert!(self.letters.iter().filter(|&&count| count == 10).count() <= 1);
    if let Some((idx, _)) = self
      .letters
      .iter()
      .enumerate()
      .find(|(_, &value)| value == 10)
    {
      self.letters[idx] = 55 - self.letters.iter().sum::<u32>();
    }
  }

  fn with_filled_remaining(mut self) -> Self {
    self.fill_remaining();
    self
  }

  pub fn int_value(&self) -> u64 {
    debug_assert!(self.letters.iter().all(|value| (0..=9).contains(value)));
    self
      .letters
      .iter()
      .fold(0, |acc, &value| 10 * acc + value as u64)
  }
}

impl Display for LetterAssignment {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ('A'..='J').try_fold((), |_, letter| write!(f, "{letter} "))?;
    writeln!(f)?;
    ('A'..='J').try_fold((), |_, letter| write!(f, "{} ", self.letter_value(letter)))
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
          Tile::Unknown(UnknownTile::Blank) => {
            [Some((DlxItem::Tile { idx }, HeaderType::Secondary)), None]
          }
          _ => [None, None],
        }
        .into_iter()
        .flatten()
      })
      .chain(('A'..='J').enumerate().flat_map(|(value, letter)| {
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

  /// Constructs Dlx constraints from a list of assignments to letters or
  /// tiles. Letter assignments may be repeated, and they will be deduplicated.
  /// If any color assignments conflict among letters (i.e. A=1 and A=2, or A=1
  /// and B=1), then None is returned.
  fn construct_dlx(
    clue_item: DlxItem,
    items: Vec<(DlxItem, u32)>,
  ) -> Option<impl Iterator<Item = Constraint<DlxItem>>> {
    println!("Checking: {clue_item:?}: {items:?}");
    let (letters, values) = match items.iter().try_fold(
      ([(); 10].map(|_| None), [(); 10].map(|_| None)),
      |(mut letters_array, mut values_array), (item, value)| {
        let value = *value;
        match item {
          DlxItem::Letter { letter } => {
            if letters_array[*letter as usize - 'A' as usize]
              .is_some_and(|prev_value| prev_value != value)
              || values_array[value as usize].is_some_and(|prev_letter| prev_letter != *letter)
            {
              ControlFlow::Break(())
            } else {
              letters_array[*letter as usize - 'A' as usize] = Some(value);
              values_array[value as usize] = Some(*letter);
              ControlFlow::Continue((letters_array, values_array))
            }
          }
          _ => ControlFlow::Continue((letters_array, values_array)),
        }
      },
    ) {
      ControlFlow::Break(_) => {
        println!("Filtered!");
        return None;
      }
      ControlFlow::Continue(arrays) => arrays,
    };
    println!("Kept");

    Some(
      iter::once(clue_item.into())
        .chain(
          items
            .into_iter()
            .filter(|(item, _)| matches!(item, DlxItem::Tile { .. }))
            .map(|(item, color)| ColorItem::new(item, color).into()),
        )
        .chain(letters.into_iter().enumerate().filter_map(|(idx, value)| {
          value.map(|value| {
            ColorItem::new(
              DlxItem::Letter {
                letter: (idx as u32 + 'A' as u32) as u8 as char,
              },
              value,
            )
            .into()
          })
        }))
        .chain(values.into_iter().enumerate().filter_map(|(idx, letter)| {
          letter.map(|letter| {
            ColorItem::new(
              DlxItem::LetterValue { value: idx as u32 },
              letter as u32 - 'A' as u32,
            )
            .into()
          })
        })),
    )
  }

  fn print_test(&self, soln: &HashMap<DlxItem, u32>) {
    self.tiles.iter().enumerate().for_each(|(idx, tile)| {
      let out = match tile {
        Tile::Unknown(UnknownTile::Blank) => {
          format!("{}", soln.get(&DlxItem::Tile { idx: idx as u32 }).unwrap())
        }
        Tile::Unknown(UnknownTile::Prefilled { hint }) => {
          format!("{}", soln.get(&DlxItem::Letter { letter: *hint }).unwrap())
        }
        Tile::Total(TotalTile {
          horizontal,
          vertical,
        }) => format!(
          "{},{}",
          match vertical {
            Some(TotalClue::OneDigit(digit)) => {
              format!("{}", soln.get(&DlxItem::Letter { letter: *digit }).unwrap())
            }
            Some(TotalClue::TwoDigit { ones, tens }) => format!(
              "{}{}",
              soln.get(&DlxItem::Letter { letter: *tens }).unwrap(),
              soln.get(&DlxItem::Letter { letter: *ones }).unwrap()
            ),
            None => "".to_string(),
          },
          match horizontal {
            Some(TotalClue::OneDigit(digit)) => {
              format!("{}", soln.get(&DlxItem::Letter { letter: *digit }).unwrap())
            }
            Some(TotalClue::TwoDigit { ones, tens }) => format!(
              "{}{}",
              soln.get(&DlxItem::Letter { letter: *tens }).unwrap(),
              soln.get(&DlxItem::Letter { letter: *ones }).unwrap()
            ),
            None => "".to_string(),
          },
        ),
        Tile::Empty => "X".to_owned(),
      };
      println!("{:10}", out);
    });
  }

  pub fn solve(&self) -> Vec<LetterAssignment> {
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
      let items = items.collect_vec();
      let items_len = items.len();
      clue
        .all_combinations(items.len() as u32)
        .flat_map(move |(total, choices)| {
          choices
            .into_iter()
            .permutations(items_len)
            .map(move |choices| (total.clone(), choices))
        })
        .filter_map(move |(total, choices)| {
          Self::construct_dlx(
            item.clone(),
            total
              .iter()
              .map(Clone::clone)
              .chain(items.iter().map(Clone::clone).zip(choices))
              .collect(),
          )
        })
    });
    let choices = (0u64..).zip(choices);

    let mut dlx = Dlx::new(items, choices);
    // println!("{dlx:?}");

    dlx
      .find_all_solution_colors()
      .map(|soln| {
        // self.print_test(&soln);
        soln
          .into_iter()
          .filter_map(|(item, color)| match item {
            DlxItem::Letter { letter } => Some((letter, color)),
            _ => None,
          })
          .fold(LetterAssignment::new(), |la, (letter, color)| {
            la.with_value(letter, color)
          })
          .with_filled_remaining()
      })
      .collect_vec()
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

#[cfg(test)]
mod test {
  use std::vec;

  use super::TotalClue;

  fn all_combinations(range: (u32, u32), num_tiles: u32) -> Vec<Vec<u32>> {
    TotalClue::all_combinations_for_range(range, num_tiles)
      .map(|(total, nums)| {
        assert_eq!(nums.iter().sum::<u32>(), total);
        nums
      })
      .collect()
  }

  #[test]
  fn test_combinations_one() {
    assert_eq!(
      all_combinations((2, 5), 1),
      vec![vec![2], vec![3], vec![4], vec![5]]
    );
  }

  #[test]
  fn test_combinations_wide_range() {
    assert_eq!(
      all_combinations((0, 12), 1),
      vec![
        vec![1],
        vec![2],
        vec![3],
        vec![4],
        vec![5],
        vec![6],
        vec![7],
        vec![8],
        vec![9]
      ]
    );
  }

  #[test]
  fn test_combinations_two() {
    assert_eq!(
      all_combinations((2, 5), 2),
      vec![vec![1, 2], vec![1, 3], vec![1, 4], vec![2, 3]]
    );
  }

  #[test]
  fn test_combinations_large_range() {
    assert_eq!(
      all_combinations((10, 20), 2),
      vec![
        vec![1, 9],
        vec![2, 8],
        vec![2, 9],
        vec![3, 7],
        vec![3, 8],
        vec![3, 9],
        vec![4, 6],
        vec![4, 7],
        vec![4, 8],
        vec![4, 9],
        vec![5, 6],
        vec![5, 7],
        vec![5, 8],
        vec![5, 9],
        vec![6, 7],
        vec![6, 8],
        vec![6, 9],
        vec![7, 8],
        vec![7, 9],
        vec![8, 9],
      ]
    );
  }

  #[test]
  fn test_combinations_large_range_three() {
    assert_eq!(
      all_combinations((10, 20), 3),
      vec![
        vec![1, 2, 7],
        vec![1, 2, 8],
        vec![1, 2, 9],
        vec![1, 3, 6],
        vec![1, 3, 7],
        vec![1, 3, 8],
        vec![1, 3, 9],
        vec![1, 4, 5],
        vec![1, 4, 6],
        vec![1, 4, 7],
        vec![1, 4, 8],
        vec![1, 4, 9],
        vec![1, 5, 6],
        vec![1, 5, 7],
        vec![1, 5, 8],
        vec![1, 5, 9],
        vec![1, 6, 7],
        vec![1, 6, 8],
        vec![1, 6, 9],
        vec![1, 7, 8],
        vec![1, 7, 9],
        vec![1, 8, 9],
        vec![2, 3, 5],
        vec![2, 3, 6],
        vec![2, 3, 7],
        vec![2, 3, 8],
        vec![2, 3, 9],
        vec![2, 4, 5],
        vec![2, 4, 6],
        vec![2, 4, 7],
        vec![2, 4, 8],
        vec![2, 4, 9],
        vec![2, 5, 6],
        vec![2, 5, 7],
        vec![2, 5, 8],
        vec![2, 5, 9],
        vec![2, 6, 7],
        vec![2, 6, 8],
        vec![2, 6, 9],
        vec![2, 7, 8],
        vec![2, 7, 9],
        vec![2, 8, 9],
        vec![3, 4, 5],
        vec![3, 4, 6],
        vec![3, 4, 7],
        vec![3, 4, 8],
        vec![3, 4, 9],
        vec![3, 5, 6],
        vec![3, 5, 7],
        vec![3, 5, 8],
        vec![3, 5, 9],
        vec![3, 6, 7],
        vec![3, 6, 8],
        vec![3, 6, 9],
        vec![3, 7, 8],
        vec![3, 7, 9],
        vec![3, 8, 9],
        vec![4, 5, 6],
        vec![4, 5, 7],
        vec![4, 5, 8],
        vec![4, 5, 9],
        vec![4, 6, 7],
        vec![4, 6, 8],
        vec![4, 6, 9],
        vec![4, 7, 8],
        vec![4, 7, 9],
        vec![5, 6, 7],
        vec![5, 6, 8],
        vec![5, 6, 9],
        vec![5, 7, 8],
      ]
    );
  }
}
