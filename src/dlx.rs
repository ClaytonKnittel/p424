use std::{
  collections::HashMap,
  fmt::{self, Display, Formatter},
  hash::Hash,
};

pub struct ColorItem<I> {
  item: I,
  color: u32,
}

impl<I> ColorItem<I> {
  pub fn new(item: I, color: u32) -> Self {
    ColorItem { item, color }
  }
}

pub enum Constraint<I> {
  Primary(I),
  Secondary(ColorItem<I>),
}

impl<I> Constraint<I> {
  fn item(&self) -> &I {
    match self {
      Constraint::Primary(item) | Constraint::Secondary(ColorItem { item, color: _ }) => item,
    }
  }

  fn color(&self) -> Option<u32> {
    match self {
      Constraint::Primary(_) => None,
      Constraint::Secondary(ColorItem { item: _, color }) => Some(*color),
    }
  }
}

impl<I> From<I> for Constraint<I> {
  fn from(value: I) -> Self {
    Constraint::Primary(value)
  }
}

struct ListNodeI<I> {
  prev: I,
  next: I,
}

type HeaderListNode = ListNodeI<u32>;

pub enum HeaderType {
  Primary,
  Secondary,
}

struct Header<I> {
  item: Option<I>,
  node: HeaderListNode,
  header_type: HeaderType,
}

impl<I> Display for Header<I>
where
  I: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    writeln!(
      f,
      "{} (prev: {}, next: {}) ({})",
      match &self.item {
        Some(item) => item.to_string(),
        None => "[None]".to_string(),
      },
      self.node.prev,
      self.node.next,
      match self.header_type {
        HeaderType::Primary => "Primary",
        HeaderType::Secondary => "Secondary",
      }
    )
  }
}

type ListNode = ListNodeI<u64>;

enum NodeType {
  Header {
    /// Number of constraints that have this item.
    size: u64,
  },
  Body {
    /// The assigned color of this node, or None if this is a primary constraint.
    color: Option<u32>,
  },
}

enum Node<N> {
  Boundary {
    /// The name of the subset listed to the left of this boundary.
    name: Option<N>,
    /// The index of the first node in the subset that comes before this
    /// boundary.
    first_for_prev: u64,
    /// The index of the last node in the subset that comes after this
    /// boundary.
    last_for_next: u64,
  },
  Normal {
    /// Node in linked list of item.
    item_node: ListNode,
    node_type: NodeType,
  },
}

impl<I> Node<I> {
  fn inc_size(&mut self) {
    match self {
      Node::Normal {
        item_node: _,
        node_type: NodeType::Header { size },
      } => *size += 1,
      _ => unreachable!("Cannot call Node::inc_size() on a non-Header node"),
    }
  }

  fn prev(&self) -> usize {
    match self {
      Node::Normal {
        item_node,
        node_type: _,
      } => item_node.prev as usize,
      Node::Boundary {
        name: _,
        first_for_prev: _,
        last_for_next: _,
      } => unreachable!("Cannot call Node::prev() on a Boundary node"),
    }
  }

  fn set_prev(&mut self, idx: usize) {
    match self {
      Node::Normal {
        item_node,
        node_type: _,
      } => item_node.prev = idx as u64,
      Node::Boundary {
        name: _,
        first_for_prev: _,
        last_for_next: _,
      } => unreachable!("Cannot call Node::set_prev() on a Boundary node"),
    }
  }

  fn next(&self) -> usize {
    match self {
      Node::Normal {
        item_node,
        node_type: _,
      } => item_node.next as usize,
      Node::Boundary {
        name: _,
        first_for_prev: _,
        last_for_next: _,
      } => unreachable!("Cannot call Node::next() on a Boundary node"),
    }
  }

  fn set_next(&mut self, idx: usize) {
    match self {
      Node::Normal {
        item_node,
        node_type: _,
      } => item_node.next = idx as u64,
      Node::Boundary {
        name: _,
        first_for_prev: _,
        last_for_next: _,
      } => unreachable!("Cannot call Node::set_next() on a Boundary node"),
    }
  }
}

impl<N> Display for Node<N>
where
  N: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Node::Boundary {
        name,
        first_for_prev,
        last_for_next,
      } => {
        writeln!(
          f,
          "{}: (first_prev: {}, last_next: {})",
          match name {
            Some(name) => name.to_string(),
            None => "[None]".to_string(),
          },
          first_for_prev,
          last_for_next
        )
      }
      Node::Normal {
        item_node: ListNodeI { prev, next },
        node_type,
      } => {
        writeln!(
          f,
          "(prev: {}, next: {}) ({})",
          prev,
          next,
          match node_type {
            NodeType::Header { size } => {
              format!("Header (size: {})", size)
            }
            NodeType::Body { color } => {
              format!(
                "Body{}",
                match color {
                  Some(color) => format!(" (color: {color})"),
                  None => "".to_string(),
                }
              )
            }
          }
        )
      }
    }
  }
}

pub struct Dlx<I, N> {
  headers: Vec<Header<I>>,
  body: Vec<Node<N>>,
}

impl<I, N> Dlx<I, N>
where
  I: Hash + Eq + Clone,
{
  pub fn new<U, S, C, D>(items: U, subsets: S) -> Self
  where
    U: IntoIterator<Item = (I, HeaderType)>,
    S: IntoIterator<Item = (N, C)>,
    C: IntoIterator<Item = D>,
    D: Into<Constraint<I>>,
  {
    Self::construct(items, subsets)
  }

  fn construct<U, S, C, D>(items: U, subsets: S) -> Self
  where
    U: IntoIterator<Item = (I, HeaderType)>,
    S: IntoIterator<Item = (N, C)>,
    C: IntoIterator<Item = D>,
    D: Into<Constraint<I>>,
  {
    let mut headers = vec![Header {
      item: None,
      node: ListNodeI { prev: 0, next: 1 },
      header_type: HeaderType::Primary,
    }];
    let mut item_map = HashMap::new();
    let mut body = Vec::new();
    let mut last_start_index = 0;

    headers.extend(
      items
        .into_iter()
        .enumerate()
        .map(|(idx, (item, header_type))| {
          let new_idx = idx as u32 + 1;
          item_map.insert(item.clone(), new_idx);

          Header {
            item: Some(item),
            node: ListNodeI {
              prev: new_idx - 1,
              next: new_idx + 1,
            },
            header_type,
          }
        }),
    );
    let last_idx = headers.len() - 1;
    headers.get_mut(0).unwrap().node.prev = last_idx as u32;
    headers.get_mut(last_idx).unwrap().node.next = 0;

    body.push(Node::Boundary {
      name: None,
      first_for_prev: 0,
      last_for_next: 0,
    });

    for (name, constraints) in subsets {
      last_start_index = body.len();
      constraints.into_iter().for_each(|constraint| {
        let constraint: Constraint<I> = constraint.into();
        let idx = body.len();

        let header_idx = *item_map.get(constraint.item()).unwrap() as usize;
        let header = body.get_mut(header_idx).unwrap();
        let prev_idx = header.prev();

        header.set_prev(idx);
        header.inc_size();
        body.get_mut(prev_idx).unwrap().set_next(idx);

        body.push(Node::Normal {
          item_node: ListNodeI {
            prev: prev_idx as u64,
            next: header_idx as u64,
          },
          node_type: NodeType::Body {
            color: constraint.color(),
          },
        });
      });

      let last_idx = body.len() as u64 - 1;
      if let Some(Node::Boundary {
        name: _,
        first_for_prev: _,
        last_for_next,
      }) = body.get_mut(last_start_index - 1)
      {
        *last_for_next = last_idx;
      } else {
        unreachable!();
      }

      body.push(Node::Boundary {
        name: Some(name),
        first_for_prev: last_start_index as u64,
        last_for_next: 0,
      });
    }

    Dlx { headers, body }
  }
}

impl<I, N> Display for Dlx<I, N>
where
  I: Display,
  N: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    for header in self.headers.iter() {
      writeln!(f, "H: {}", header)?;
    }
    for node in self.body.iter() {
      writeln!(f, "N: {}", node)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::{Dlx, HeaderType};

  #[test]
  fn test_simple() {
    let dlx = Dlx::new(vec![(1, HeaderType::Primary)], vec![(0, vec![1])]);
  }
}
