use std::io;

use dlx::{Dlx, HeaderType};
use kakuro::Kakuro;

mod dlx;
mod kakuro;
mod parenthesis_split;
mod solver;

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

  let dlx = Dlx::new(
    vec![
      (1, HeaderType::Primary),
      (2, HeaderType::Primary),
      (3, HeaderType::Primary),
    ],
    vec![(0, vec![1, 2]), (1, vec![3])],
  );

  Ok(())
}
