use std::io;

use kakuro::Kakuro;

use crate::parenthesis_split::ParenthesesAwareSplit;

mod kakuro;
mod parenthesis_split;
mod solver;

fn main() -> io::Result<()> {
  let s = "a, b, c, (d, e), f";
  for x in s.split_paren() {
    println!("{x}");
  }

  let kakuros = Kakuro::from_file("p424_kakuro200.txt")?;
  for kakuro in kakuros.iter() {
    println!("{}", kakuro);
  }

  Ok(())
}
