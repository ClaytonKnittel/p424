use std::{collections::HashMap, hash::Hash};

pub struct ColorItem<I> {
  item: I,
  color: u64,
}

impl<I> ColorItem<I> {
  pub fn new(item: I, color: u64) -> Self {
    ColorItem { item, color }
  }
}

pub enum Constraint<I> {
  Primary(I),
  Secondary(ColorItem<I>),
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

struct Header<I> {
  item: I,
  node: HeaderListNode,
}

type ListNode = ListNodeI<u64>;

enum NodeType {
  Header {
    /// Number of constraints that have this item.
    size: u64,
  },
  Normal {
    /// The assigned color of this node, or None if this is a primary constraint.
    color: Option<u32>,
  },
}

struct Node {
  /// Node in linked list of subset.
  subset_node: ListNode,
  /// Node in linked list of item.
  item_node: ListNode,
  node_type: NodeType,
}

pub struct Dlx {}

impl Dlx {
  pub fn new<I, U, S, C, D>(items: U, subsets: S) -> Self
  where
    I: Hash + Eq,
    U: IntoIterator<Item = I>,
    S: IntoIterator<Item = C>,
    C: IntoIterator<Item = D>,
    D: Into<Constraint<I>>,
  {
    let universe = items
      .into_iter()
      .enumerate()
      .map(|(idx, item)| (item, idx))
      .collect();
    Self::construct(universe, subsets)
  }

  fn construct<I, S, C, D>(universe: HashMap<I, usize>, subsets: S) -> Self
  where
    I: Hash + Eq,
    S: IntoIterator<Item = C>,
    C: IntoIterator<Item = D>,
    D: Into<Constraint<I>>,
  {
    Dlx {}
  }
}
