use std::fmt::Display;

use termion::style;

pub struct Sudoku {
  grid: [[u32; 9]; 9],
}

impl Sudoku {
  pub fn new(grid: [[u32; 9]; 9]) -> Self {
    Self { grid }
  }

  pub fn solve(&mut self) -> bool {
    false
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
