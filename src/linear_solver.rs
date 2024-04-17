use std::iter::repeat;

use itertools::{FoldWhile, Itertools};

struct Term<V> {
  var: V,
  factor: i32,
}

pub struct LinearSolver<V> {
  vars: Vec<Term<V>>,
}

impl<V> LinearSolver<V>
where
  V: Clone + Eq,
{
  pub fn new() -> Self {
    Self { vars: Vec::new() }
  }

  fn find(&mut self, var: V) -> &mut Term<V> {
    match self.vars.iter_mut().find(|&term| term.var == var) {
      Some(term) => term,
      None => {
        self.vars.push(Term { var, factor: 0 });
        self.vars.last_mut().unwrap()
      }
    }
  }

  pub fn add(&mut self, var: V, factor: i32) {
    self.find(var).factor += factor;
  }

  pub fn find_all_solutions(
    &self,
  ) -> impl Iterator<Item = impl Iterator<Item = (V, u32)> + '_> + '_ {
    repeat(())
      .scan(
        (self.vars.iter().map(|_| 0).collect::<Vec<_>>(), 0),
        move |(digs, total), _| {
          if !digs
            .iter_mut()
            .zip(self.vars.iter())
            .fold_while((), |_, (digit, var)| {
              if *digit < 9 {
                *digit += 1;
                *total += var.factor;
                FoldWhile::Done(())
              } else {
                *digit = 0;
                *total -= 9 * var.factor;
                FoldWhile::Continue(())
              }
            })
            .is_done()
          {
            None
          } else {
            Some((digs.clone(), *total))
          }
        },
      )
      .filter(|&(_, total)| total == 0)
      .map(|(digs, _)| {
        self
          .vars
          .iter()
          .zip(digs.into_iter())
          .map(|(Term { var, .. }, digit)| (var.clone(), digit))
      })
  }
}
