use std::io;

use kakuro::Kakuro;

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
          .map(|tile| tile.to_string())
          .collect::<Vec<_>>()
          .join(", "),
      );
    }
  }

  Ok(())
}
