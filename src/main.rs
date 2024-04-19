use std::io;

use kakuro::Kakuro;

pub mod dlx;
mod kakuro;
mod parenthesis_split;
#[cfg(test)]
mod sudoku;

fn main() -> io::Result<()> {
  let kakuros = Kakuro::from_file("p424_kakuro200.txt")?;
  let sums: u64 = kakuros
    .iter()
    .map(|kakuro| {
      let letters = kakuro.solve();
      debug_assert_eq!(letters.len(), 1);
      letters.first().unwrap().int_value()
    })
    .sum();

  println!("Sum: {sums}");

  Ok(())
}
