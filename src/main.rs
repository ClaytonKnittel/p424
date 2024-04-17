use std::io;

use kakuro::Kakuro;

mod dlx;
mod kakuro;
mod linear_solver;
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
        line.0 .1,
        line
          .1
          .map(|(idx, tile)| format!("({} {})", tile, idx))
          .collect::<Vec<_>>()
          .join(", "),
      );
    }
  }

  Ok(())
}
