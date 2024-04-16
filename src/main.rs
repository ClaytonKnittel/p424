use std::io;

use dlx::{Dlx, HeaderType};
use kakuro::Kakuro;

use crate::{
  dlx::{ColorItem, Constraint},
  sudoku::Sudoku,
};

mod dlx;
mod kakuro;
mod parenthesis_split;
mod solver;
mod sudoku;

fn main() -> io::Result<()> {
  let kakuros = Kakuro::from_file("p424_kakuro200.txt")?;
  for kakuro in kakuros.iter().take(1) {
    println!("{}", kakuro);

    for line in kakuro.enumerate_lines() {
      println!(
        "Line: {}: {}",
        line.0,
        line
          .1
          .map(|(idx, tile)| format!("({} {})", tile, idx))
          .collect::<Vec<_>>()
          .join(", "),
      );
    }
  }

  // let mut dlx = Dlx::new(
  //   vec![
  //     (1, HeaderType::Primary),
  //     (3, HeaderType::Secondary),
  //     (2, HeaderType::Primary),
  //   ],
  //   vec![
  //     (0, vec![1.into(), 2.into()]),
  //     (1, vec![Constraint::Secondary(ColorItem::new(3, 1))]),
  //     (2, vec![1.into()]),
  //   ],
  // );

  let mut dlx = Dlx::new(
    vec![
      ('p', HeaderType::Primary),
      ('q', HeaderType::Primary),
      ('a', HeaderType::Secondary),
    ],
    vec![
      (
        0,
        vec![Constraint::Primary('p'), ColorItem::new('a', 1).into()],
      ),
      (1, vec!['p'.into(), ColorItem::new('a', 2).into()]),
      (2, vec!['q'.into(), ColorItem::new('a', 3).into()]),
      (3, vec!['q'.into(), ColorItem::new('a', 1).into()]),
    ],
  );

  println!("{:?}", dlx);

  if let Some(solution) = dlx.find_solution() {
    print!("Solution:");
    for c in solution {
      print!(" {}", c);
    }
    println!();
  } else {
    println!("No solution found");
  }

  let mut sudoku = Sudoku::new([
    [0, 0, 4, 0, 5, 0, 0, 0, 0],
    [9, 0, 0, 7, 3, 4, 6, 0, 0],
    [0, 0, 3, 0, 2, 1, 0, 4, 9],
    [0, 3, 5, 0, 9, 0, 4, 8, 0],
    [0, 9, 0, 0, 0, 0, 0, 3, 0],
    [0, 7, 6, 0, 1, 0, 9, 2, 0],
    [3, 1, 0, 9, 7, 0, 2, 0, 0],
    [0, 0, 9, 1, 8, 2, 0, 0, 3],
    [0, 0, 0, 0, 6, 0, 1, 0, 0],
  ]);

  println!("{sudoku}");
  println!("Solved: {}", sudoku.solve());
  println!("{sudoku}");

  Ok(())
}
