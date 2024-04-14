mod kakuro;
mod parenthesis_split;
mod solver;

fn main() {
  let mut v = Vec::<u32>::new();
  for i in 0..10 {
    v.push(0);
    v[i].count_ones();
  }

  let w: Vec<_> = (0..10).map(|i| i + 1).collect();
  println!("Hello, world!");
}
