use itertools::{FoldWhile, Itertools};

struct ParenthesesAwareSplit<'a> {
  inner: &'a str,
}

impl<'a> Iterator for ParenthesesAwareSplit<'a> {
  type Item = &'a str;

  fn next(&mut self) -> Option<Self::Item> {
    match self
      .inner
      .chars()
      .enumerate()
      .fold_while(0, |depth, (idx, c)| match c {
        '(' => FoldWhile::Continue(depth + 1),
        ')' => FoldWhile::Continue(depth - 1),
        ',' => {
          if depth == 0 {
            FoldWhile::Done(idx)
          } else {
            FoldWhile::Continue(depth)
          }
        }
        _ => FoldWhile::Continue(depth),
      }) {
      FoldWhile::Done(end) => {
        let tmp = self.inner;
        self.inner = &self.inner[(end + 1)..];
        Some(&tmp[..end])
      }
      FoldWhile::Continue(_) => {
        let tmp = self.inner;
        self.inner = &self.inner[self.inner.len()..];
        if tmp.len() > 0 {
          Some(tmp)
           } else {
             None}
        }
      },
    }
  }
}

fn parentheses_aware_split<'a>(input: &'a str) -> ParenthesesAwareSplit<'a> {
  ParenthesesAwareSplit { inner: input }
}
