use std::io;

use kakuro::Kakuro;

mod dlx;
mod kakuro;
mod linear_solver;
mod parenthesis_split;
mod solver;
mod sudoku;

fn main() -> io::Result<()> {
  // let kakuros = Kakuro::from_file("p424_kakuro200.txt")?;
  let kakuros = Kakuro::from_file("kakuro_test.txt")?;
  for kakuro in kakuros.iter().take(1) {
    println!("{}", kakuro);

    kakuro.solve();
    break;
  }

  Ok(())
}
