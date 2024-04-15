use std::{
  collections::{HashMap, HashSet},
  fmt::{self, Debug, Display, Formatter},
  hash::Hash,
  iter,
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
    write!(
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

type ListNode = ListNodeI<usize>;

enum NodeType {
  Header {
    /// Number of constraints that have this item.
    size: usize,
  },
  Body {
    /// The assigned color of this node, or None if this is a primary constraint.
    color: Option<u32>,
    /// The index of the header node associated with this node.
    top: u32,
  },
}

enum Node<N> {
  Boundary {
    /// The name of the subset listed to the left of this boundary.
    name: Option<N>,
    /// The index of the first node in the subset that comes before this
    /// boundary.
    first_for_prev: usize,
    /// The index of the last node in the subset that comes after this
    /// boundary.
    last_for_next: usize,
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
      } => item_node.prev,
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
      } => item_node.prev = idx,
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
      } => item_node.next,
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
      } => item_node.next = idx,
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
        write!(
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
        write!(
          f,
          "(prev: {}, next: {}) ({})",
          prev,
          next,
          match node_type {
            NodeType::Header { size } => {
              format!("Header (size: {})", size)
            }
            NodeType::Body { color, top } => {
              format!(
                "Body (top: {top}){}",
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
  I: Hash + Eq + Clone + Debug,
  N: Hash + Eq + Clone + Debug,
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
    let mut last_start_index;
    let mut subset_names = HashSet::new();

    // Push phony node to first element of body.
    body.push(Node::Boundary {
      name: None,
      first_for_prev: 0,
      last_for_next: 0,
    });

    let (primary_headers, secondary_headers): (Vec<_>, Vec<_>) =
      items
        .into_iter()
        .partition(|(_, header_type)| match header_type {
          HeaderType::Primary => true,
          HeaderType::Secondary => false,
        });

    let primary_headers_len = primary_headers.len() as u32;
    headers.extend(
      primary_headers
        .into_iter()
        .chain(secondary_headers)
        .enumerate()
        .map(|(idx, (item, header_type))| {
          let new_idx = idx + 1;
          if item_map.insert(item.clone(), new_idx).is_some() {
            panic!("Duplicate item {:?}", item);
          }
          body.push(Node::Normal {
            item_node: ListNodeI {
              prev: new_idx,
              next: new_idx,
            },
            node_type: NodeType::Header { size: 0 },
          });

          Header {
            item: Some(item),
            node: ListNodeI {
              prev: new_idx as u32 - 1,
              next: new_idx as u32 + 1,
            },
            header_type,
          }
        }),
    );
    let last_idx = headers.len();
    headers.push(Header {
      item: None,
      node: ListNodeI {
        prev: last_idx as u32 - 1,
        next: primary_headers_len + 1,
      },
      header_type: HeaderType::Secondary,
    });
    headers.get_mut(0).unwrap().node.prev = primary_headers_len;
    headers.get_mut(last_idx).unwrap().node.next = primary_headers_len + 1;

    body.push(Node::Boundary {
      name: None,
      first_for_prev: 0,
      last_for_next: 0,
    });

    for (name, constraints) in subsets {
      if !subset_names.insert(name.clone()) {
        panic!("Duplicate subset name: {name:?}");
      }

      last_start_index = body.len();
      constraints.into_iter().for_each(|constraint| {
        let constraint: Constraint<I> = constraint.into();
        let idx = body.len();

        let header_idx = *item_map
          .get(constraint.item())
          .unwrap_or_else(|| panic!("Unknown item {:?}", constraint.item()));
        let header = body.get_mut(header_idx).unwrap();
        let prev_idx = header.prev();

        debug_assert!(
          matches!(
            (headers.get(header_idx).unwrap(), &constraint),
            (
              Header {
                item: _,
                node: _,
                header_type: HeaderType::Primary,
              },
              Constraint::Primary(_),
            ) | (
              Header {
                item: _,
                node: _,
                header_type: HeaderType::Secondary,
              },
              Constraint::Secondary(_),
            )
          ),
          "Expect constraint type to match item type (primary vs. secondary)"
        );

        header.set_prev(idx);
        header.inc_size();
        body.get_mut(prev_idx).unwrap().set_next(idx);

        body.push(Node::Normal {
          item_node: ListNodeI {
            prev: prev_idx,
            next: header_idx,
          },
          node_type: NodeType::Body {
            color: constraint.color(),
            top: header_idx as u32,
          },
        });
      });

      let last_idx = body.len() - 1;
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
        first_for_prev: last_start_index,
        last_for_next: 0,
      });
    }

    Dlx { headers, body }
  }

  fn num_primary_items(&self) -> usize {
    self.headers.first().unwrap().node.prev as usize
  }

  fn header(&self, idx: usize) -> &Header<I> {
    debug_assert!((..self.headers.len()).contains(&idx));
    unsafe { self.headers.get_unchecked(idx) }
  }

  fn header_mut(&mut self, idx: usize) -> &mut Header<I> {
    debug_assert!((..self.headers.len()).contains(&idx));
    unsafe { self.headers.get_unchecked_mut(idx) }
  }

  fn body_header(&self, idx: usize) -> &Node<N> {
    debug_assert!((1..(self.headers.len() - 1)).contains(&idx));
    unsafe { self.body.get_unchecked(idx) }
  }

  fn body_header_mut(&mut self, idx: usize) -> &mut Node<N> {
    debug_assert!((1..(self.headers.len() - 1)).contains(&idx));
    unsafe { self.body.get_unchecked_mut(idx) }
  }

  fn body_node(&self, idx: usize) -> &Node<N> {
    debug_assert!((self.headers.len()..self.body.len()).contains(&idx));
    unsafe { self.body.get_unchecked(idx) }
  }

  fn body_node_mut(&mut self, idx: usize) -> &mut Node<N> {
    debug_assert!((self.headers.len()..self.body.len()).contains(&idx));
    unsafe { self.body.get_unchecked_mut(idx) }
  }

  /// Remove the subset containing the node at `idx` from the grid.
  fn hide(&mut self, idx: usize) {
    let mut q = idx + 1;
    while q != idx {
      match self.body_node(q) {
        Node::Boundary {
          name: _,
          first_for_prev,
          last_for_next: _,
        } => {
          q = *first_for_prev;
        }
        Node::Normal {
          item_node,
          node_type: NodeType::Body { color, top },
        } => {
          let top = *top as usize;

          if color.is_some() {
            let prev_idx = item_node.prev;
            let next_idx = item_node.next;
            self.body_node_mut(prev_idx).set_next(next_idx);
            self.body_node_mut(next_idx).set_prev(prev_idx);
          }
          if let Node::Normal {
            item_node: _,
            node_type: NodeType::Header { size },
          } = self.body_node_mut(top)
          {
            *size -= 1;
          } else {
            unreachable!("Unexpected non-header at index {top}");
          }
          q += 1;
        }
        Node::Normal {
          item_node: _,
          node_type: NodeType::Header { size: _ },
        } => unreachable!("Unexpected header encountered in hide() at index {q}"),
      }
    }
  }

  /// Reverts `hide(idx)`, assuming the state of Dlx was exactly as it was when
  /// `hide(idx)` was called.
  fn unhide(&mut self, idx: usize) {
    let mut q = idx - 1;
    while q != idx {
      match self.body_node(q) {
        Node::Boundary {
          name: _,
          first_for_prev: _,
          last_for_next,
        } => {
          q = *last_for_next;
        }
        Node::Normal {
          item_node,
          node_type: NodeType::Body { color, top },
        } => {
          let top = *top as usize;

          if color.is_some() {
            let prev_idx = item_node.prev;
            let next_idx = item_node.next;
            self.body_node_mut(prev_idx).set_next(q);
            self.body_node_mut(next_idx).set_prev(q);
          }
          if let Node::Normal {
            item_node: _,
            node_type: NodeType::Header { size },
          } = self.body_node_mut(top)
          {
            *size += 1;
          } else {
            unreachable!("Unexpected non-header at index {top}");
          }
          q -= 1;
        }
        Node::Normal {
          item_node: _,
          node_type: NodeType::Header { size: _ },
        } => unreachable!("Unexpected header encountered in unhide() at index {q}"),
      }
    }
  }

  /// Remove all subsets which contain the header item `idx`, and hide the item
  /// from the items list.
  fn cover(&mut self, idx: usize) {
    debug_assert!((1..=self.num_primary_items()).contains(&idx));
    let mut p = self.body_header(idx).next();
    while p != idx {
      self.hide(p);
      p = self.body_node(p).next();
    }

    // Hide this item in the items list.
    let header = self.header(idx);
    let prev_idx = header.node.prev;
    let next_idx = header.node.next;
    self.header_mut(prev_idx as usize).node.next = next_idx;
    self.header_mut(next_idx as usize).node.prev = prev_idx;
  }

  /// Reverts `cover(idx)`, assuming the state of Dlx was exactly as it was
  /// when `cover(idx)` was called.
  fn uncover(&mut self, idx: usize) {
    debug_assert!((1..=self.num_primary_items()).contains(&idx));
    // Put this item back in the items list.
    let header = self.header(idx);
    let prev_idx = header.node.prev;
    let next_idx = header.node.next;
    self.header_mut(prev_idx as usize).node.next = idx as u32;
    self.header_mut(next_idx as usize).node.prev = idx as u32;

    let mut p = self.body_header(idx).prev();
    while p != idx {
      self.unhide(p);
      p = self.body_node(p).prev();
    }
  }

  /// Covers all subsets with secondary constraints which don't have the same
  /// color as the constraint at index `idx`.
  fn purify(&mut self, idx: usize) {
    debug_assert!(((self.num_primary_items() + 1)..self.headers.len()).contains(&idx));
    let (color, top) = match self.body_node(idx) {
      Node::Normal {
        item_node: _,
        node_type: NodeType::Body {
          color: Some(color),
          top,
        },
      } => (*color, *top as usize),
      _ => unreachable!("Unexpected uncolored node for secondary constraint at index {idx}."),
    };

    let mut p = self.body_header(top).next();
    while p != idx {
      if let Node::Normal {
        item_node: _,
        node_type: NodeType::Body {
          color: p_color,
          top: _,
        },
      } = self.body_node_mut(p)
      {
        if *p_color == Some(color) {
          *p_color = None;
        } else {
          self.hide(p);
        }
      } else {
        unreachable!("Unexpected non-body node at index {p}");
      }
      p = self.body_node(p).next();
    }
  }

  /// Reverts `purify(idx)`, assuming the state of Dlx was exactly as it was
  /// when `purify(idx)` was called.
  fn unpurify(&mut self, idx: usize) {
    debug_assert!(((self.num_primary_items() + 1)..self.headers.len()).contains(&idx));
    let (color, top) = match self.body_node(idx) {
      Node::Normal {
        item_node: _,
        node_type: NodeType::Body {
          color: Some(color),
          top,
        },
      } => (*color, *top as usize),
      _ => unreachable!("Unexpected uncolored node for secondary constraint at index {idx}."),
    };

    let mut p = self.body_header(top).prev();
    while p != idx {
      if let Node::Normal {
        item_node: _,
        node_type: NodeType::Body {
          color: p_color,
          top: _,
        },
      } = self.body_node_mut(p)
      {
        if p_color.is_none() {
          *p_color = Some(color);
        } else {
          self.unhide(p);
        }
      } else {
        unreachable!("Unexpected non-body node at index {p}");
      }
      p = self.body_node(p).prev();
    }
  }

  pub fn find_solution(&mut self) -> impl Iterator<Item = N> {
    iter::empty()
  }
}

impl<I, N> Display for Dlx<I, N>
where
  I: Display,
  N: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    for (idx, header) in self.headers.iter().enumerate() {
      writeln!(f, "{idx:<3} H: {header}")?;
    }
    for (idx, node) in self.body.iter().enumerate() {
      writeln!(f, "{idx:<3} N: {}", node)?;
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
