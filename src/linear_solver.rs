use std::iter::repeat;

use itertools::{FoldWhile, Itertools};

#[derive(Clone)]
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
    if let Some(idx) = self
      .vars
      .iter()
      .enumerate()
      .find(|(_, term)| term.var == var)
      .map(|(idx, _)| idx)
    {
      &mut self.vars[idx]
    } else {
      self.vars.push(Term { var, factor: 0 });
      self.vars.last_mut().unwrap()
    }
  }

  pub fn add(&mut self, var: V, factor: i32) {
    self.find(var).factor += factor;
  }

  pub fn find_all_solutions_owned(self) -> impl Iterator<Item = impl Iterator<Item = (V, u32)>> {
    repeat(())
      .take(10usize.pow(self.vars.len() as u32))
      .scan(
        (self.vars.iter().map(|_| 0).collect::<Vec<_>>(), 0),
        move |(digs, total), _| {
          digs
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
            .is_done();
          Some((self.vars.clone().into_iter().zip(digs.clone()), *total))
        },
      )
      .filter(|&(_, total)| total == 0)
      .map(|(digs, _)| digs.map(|(Term { var, .. }, digit)| (var.clone(), digit)))
  }
}

#[cfg(test)]
mod test {
  use std::iter;

  use itertools::Itertools;

  use super::LinearSolver;

  #[test]
  fn test_easy() {
    #[derive(Clone, PartialEq, Eq, Debug)]
    enum Vars {
      X,
    }

    let mut slv = LinearSolver::new();
    slv.add(Vars::X, 1);

    assert!(slv
      .find_all_solutions_owned()
      .map(|soln| soln.collect_vec())
      .eq(iter::once(vec![(Vars::X, 0)])));
  }

  #[test]
  fn test_two_vars() {
    #[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
    enum Vars {
      X,
      Y,
    }

    let mut slv = LinearSolver::new();
    slv.add(Vars::X, -2);
    slv.add(Vars::Y, 3);

    assert!(slv
      .find_all_solutions_owned()
      .map(|soln| soln.collect_vec())
      .sorted()
      .eq(
        [
          vec![(Vars::X, 0), (Vars::Y, 0)],
          vec![(Vars::X, 3), (Vars::Y, 2)],
          vec![(Vars::X, 6), (Vars::Y, 4)],
          vec![(Vars::X, 9), (Vars::Y, 6)]
        ]
        .into_iter()
      ));
  }
}
