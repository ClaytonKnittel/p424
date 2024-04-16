use std::{collections::HashSet, fmt::Display};

use termion::style;

pub struct Sudoku {
  grid: [[u32; 9]; 9],
}

impl Sudoku {
  pub fn new(grid: [[u32; 9]; 9]) -> Self {
    Self { grid }
  }

  pub fn solve(&mut self) -> bool {
    #[derive(PartialEq, Eq, Hash, Clone)]
    enum Item {
      Cell { row: u32, col: u32 },
      Row { col: u32, digit: u32 },
      Col { row: u32, digit: u32 },
      Box { idx: u32, digit: u32 },
    }

    let mut items: HashSet<Item> = (0..81)
      .flat_map(|i| {
        let row = i % 9;
        let col = i / 9;
        [
          Item::Cell { row, col },
          Item::Row {
            col,
            digit: row + 1,
          },
          Item::Col {
            row,
            digit: col + 1,
          },
          Item::Box {
            idx: row,
            digit: col + 1,
          },
        ]
        .into_iter()
      })
      .collect();

    let valid = self
      .grid
      .iter()
      .enumerate()
      .fold(true, |valid, (row, digits)| {
        let row = row as u32;
        valid
          && digits
            .iter()
            .enumerate()
            .filter(|(_, digit)| **digit != 0)
            .fold(true, |valid, (col, digit)| {
              let col = col as u32;
              let digit = *digit;
              let idx = (row / 3) * 3 + col / 3;

              valid
                && items.remove(&Item::Cell { row, col })
                && items.remove(&Item::Row { col, digit })
                && items.remove(&Item::Col { row, digit })
                && items.remove(&Item::Box { idx, digit })
            })
      });

    if !valid {
      return false;
    }

    true
  }
}

impl Display for Sudoku {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "+")?;
    (0..9).try_fold((), |_, _| write!(f, "===+"))?;
    writeln!(f)?;

    self.grid.iter().enumerate().try_fold((), |_, (y, row)| {
      write!(f, "H")?;
      row.iter().enumerate().try_fold((), |_, (x, digit)| {
        write!(
          f,
          " {} ",
          if *digit == 0 {
            " ".to_string()
          } else {
            digit.to_string()
          }
        )?;
        if x % 3 == 2 {
          write!(f, "H",)
        } else {
          write!(f, "|")
        }
      })?;
      writeln!(f)?;

      write!(f, "+")?;
      (0..9).try_fold((), |_, _| {
        if y % 3 == 2 {
          write!(f, "===+")
        } else {
          write!(f, "---+")
        }
      })?;
      if y < 8 {
        writeln!(f)?;
      }

      Ok(())
    })
  }
}
