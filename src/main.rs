use std::io;

use kakuro::{Kakuro, TotalClue};
use sudoku::Sudoku;

mod dlx;
mod kakuro;
mod linear_solver;
mod parenthesis_split;
mod solver;
mod sudoku;

fn main() -> io::Result<()> {
  // TotalClue::all_combinations_for_range((2, 5), 1);
  // TotalClue::all_combinations_for_range((2, 5), 2);
  // TotalClue::all_combinations_for_range((10, 20), 2);
  TotalClue::all_combinations_for_range((10, 20), 3, |combination| {
    println!("{combination:?}");
  });

  /*
  let kakuros = Kakuro::from_file("p424_kakuro200.txt")?;
  // let kakuros = Kakuro::from_file("kakuro_test.txt")?;
  let sums: u64 = kakuros
    .iter()
    .take(2)
    .map(|kakuro| {
      println!("{}", kakuro);

      let letters = kakuro.solve();
      println!("Letters:");
      for letters in letters.iter() {
        println!("{letters}");
      }
      letters.first().unwrap().int_value()
    })
    .sum();

  println!("Sum: {sums}");

  if false {
    let mut s = Sudoku::new([[0; 9]; 9]);
    s.solve();
    println!("{s}");
  }
  */

  Ok(())
}
