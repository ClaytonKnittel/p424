use itertools::{FoldWhile, Itertools};

pub struct ParenthesesAwareSplitIter<'a> {
  inner: &'a str,
}

impl<'a> Iterator for ParenthesesAwareSplitIter<'a> {
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
        if !tmp.is_empty() {
          Some(tmp)
        } else {
          None
        }
      }
    }
  }
}

pub trait ParenthesesAwareSplit<'a>: Into<&'a str> {
  fn split_paren(self) -> ParenthesesAwareSplitIter<'a> {
    ParenthesesAwareSplitIter { inner: self.into() }
  }
}

impl<'a, T> ParenthesesAwareSplit<'a> for T where T: Into<&'a str> {}
