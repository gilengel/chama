//! Component children module

use yew::html::Html;
use yew::virtual_dom::{VChild, VNode};
use std::fmt;

pub type DynChildren = DynChildrenRenderer<Html>;
pub type DynChildrenWithProps<CHILD> = DynChildrenRenderer<VChild<CHILD>>;

/// A type used for rendering children html.
#[derive(Clone)]
pub struct DynChildrenRenderer<T> {
    children: Vec<T>,
}

impl<T: PartialEq> PartialEq for DynChildrenRenderer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
    }
}

impl<T> DynChildrenRenderer<T>
where
    T: Clone + Into<VNode>,
{
    /// Create children
    pub fn new(children: Vec<T>) -> Self {
        Self { children }
    }

    /// Children list is empty
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Number of children elements
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Render children components and return `Iterator`
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        // clone each child lazily.
        // This way `self.iter().next()` only has to clone a single node.
        self.children.iter().cloned()
    }

    pub fn push(&mut self, e: T) {
        self.children.push(e);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.children.pop()
    }
}

impl<T> Default for DynChildrenRenderer<T> {
    fn default() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl<T> fmt::Debug for DynChildrenRenderer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DynChildrenRenderer<_>")
    }
}

impl<T> IntoIterator for DynChildrenRenderer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}