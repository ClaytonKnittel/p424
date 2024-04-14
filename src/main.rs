use crate::parenthesis_split::ParenthesesAwareSplit;

mod kakuro;
mod parenthesis_split;
mod solver;

fn main() {
  let s = "a, b, c, (d, e), f";
  for x in s.split_paren() {
    println!("{x}");
  }
}
