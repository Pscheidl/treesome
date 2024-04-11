use std::ops::Index;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::sized::structs::Array;

const LEAF_NODE_MARK: isize = -1;
const ROOT_NODE: isize = 0;

/// A binary tree representation for fast traversal, suitable for dense trees.
/// Special implementation for binary tree is offered for faster traversal times over the generalized
/// `k` tree, where an array of arrays and indices checks happen.
#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BTree<T, const N: usize> {
    pub l_nodes: Array<isize, N>,
    pub r_nodes: Array<isize, N>,
    pub values: Array<T, N>,
}

// Node's children ids
#[derive(Eq, PartialEq, Debug)]
pub struct Children {
    pub left: isize,
    pub right: isize,
}

impl From<(isize, isize)> for Children {
    fn from(value: (isize, isize)) -> Self {
        Children {
            left: value.0,
            right: value.1,
        }
    }
}

impl<T, const N: usize> BTree<T, N> {
    /// Constructs a new tree from array representation
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::sized::BTree;
    ///         let left = [1, 3, 5, -1, -1, -1, -1];
    ///         let right = [2, 4, 6, -1, -1, -1, -1];
    ///         let values = [10, 51, 36, 90, 32, 16, 5];
    ///         let tree = BTree::new(left, right, values);
    /// ```
    pub fn new(l_nodes: [isize; N], r_nodes: [isize; N], values: [T; N]) -> Self {
        Self {
            l_nodes: l_nodes.into(),
            r_nodes: r_nodes.into(),
            values: values.into(),
        }
    }

    /// True if given `node_id` is a leaf node (no children), false otherwise.
    pub fn is_leaf_node(&self, node_id: usize) -> bool {
        self.l_nodes[node_id] == LEAF_NODE_MARK && self.r_nodes[node_id] == LEAF_NODE_MARK
    }

    /// Returns left and right child of a node. The value of `-1` means no child in that direction.
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::sized::BTree;
    ///         let left = [1, 3, 5, -1, -1, -1, -1];
    ///         let right = [2, 4, 6, -1, -1, -1, -1];
    ///         let values = [10, 51, 36, 90, 32, 16, 5];
    ///         let tree = BTree::new(left, right, values);
    ///         assert_eq!(tree.children(0), (1, 2).into());
    ///
    /// ```
    pub fn children(&self, node_id: usize) -> Children {
        Children {
            left: self.l_nodes[node_id],
            right: self.r_nodes[node_id],
        }
    }

    /// Return's node_id of its parent, if it exists.
    /// If there's no parent (root node, non-existent node_id) for given node, `None` is returned.
    /// Computational complexity of the lookup is O(1), as the formula used calculates the exact
    /// position of the parent node.
    /// # Examples
    ///
    /// ```
    ///         use treesome::sized::BTree;
    ///         let left = [1, 3, 5, -1, -1, -1, -1];
    ///         let right = [2, 4, 6, -1, -1, -1, -1];
    ///         let values = [10, 51, 36, 90, 32, 16, 5];
    ///         let tree = BTree::new(left, right, values);
    ///
    ///         assert_eq!(tree.parent(0), None);
    ///         assert_eq!(tree.parent(3).unwrap(), 1);
    ///
    /// ```
    pub fn parent(&self, node_id: isize) -> Option<isize> {
        if node_id <= ROOT_NODE || node_id as usize >= self.values.len() {
            return None; // Root node doesn't have a parent.
        };

        Some((node_id - 1) / 2)
    }
}

impl<T, const N: usize> Index<usize> for BTree<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

/// Walks a binary node by node, back and forth. From root to leaf nodes, and back.
///
///
/// ## Thread safety
/// Not thread safe. Cheap to copy & clone, as this structure serves as a view to a tree with small
/// state overhead.
///
/// ## Example
///
/// ```rust
///        use treesome::sized::{BTree, Walker};
/// let left = [1, 3, 5, -1, -1, -1, -1];
///         let right = [2, 4, 6, -1, -1, -1, -1];
///         let values = [10, 51, 36, 90, 32, 16, 5];
///         let tree = BTree::new(left, right, values);
///
///         let mut walker = Walker::for_tree(&tree);
///         let right_child = walker.go_right();
///         assert_eq!(right_child, Some(&tree.values[2]));
/// ```
///
#[derive(Debug, Copy, Clone)]
pub struct Walker<'a, T, const N: usize> {
    tree: &'a BTree<T, N>,
    curr_node_id: isize,
}

impl<'a, T, const N: usize> Walker<'a, T, N> {
    pub fn for_tree(tree: &'a BTree<T, N>) -> Self {
        Self {
            tree,
            curr_node_id: 0,
        }
    }

    /// Visits the right child of current node and returns its value, of it exists.
    pub fn go_right(&mut self) -> Option<&T> {
        let right_child_id = { self.tree.r_nodes[self.curr_node_id as usize] };
        if right_child_id != LEAF_NODE_MARK {
            self.curr_node_id = right_child_id;
            Some(&self.tree[right_child_id as usize])
        } else {
            None
        }
    }

    /// Visits the left child of current node and returns its value, of it exists.
    pub fn go_left(&mut self) -> Option<&T> {
        let left_child_id = { self.tree.l_nodes[self.curr_node_id as usize] };
        if left_child_id != LEAF_NODE_MARK {
            self.curr_node_id = left_child_id;
            Some(&self.tree[left_child_id as usize])
        } else {
            None
        }
    }

    /// Goes back to parent of the current node and returns its value, if it exists.
    /// Stays on current position and returns `None` if there's no parent.
    /// Root nodes and nodes out of bounds have no parents.
    pub fn go_parent(&mut self) -> Option<&T> {
        let parent = self.tree.parent(self.curr_node_id)?;
        self.curr_node_id = parent;
        Some(&self.tree.values[parent as usize])
    }
}

#[cfg(test)]
mod tests {
    use crate::sized::{BTree, Walker, ROOT_NODE};

    #[test]
    fn walker() {
        let left = [1, 3, 5, -1, -1, -1, -1];
        let right = [2, 4, 6, -1, -1, -1, -1];
        let values = [10, 51, 36, 90, 32, 16, 5];
        let tree = BTree::new(left, right, values);

        let mut walker = Walker::for_tree(&tree);
        let right_child = walker.go_right();
        assert_eq!(right_child, Some(&tree.values[2]));

        let left_child = walker.go_left();
        assert_eq!(left_child, Some(&tree.values[5]));

        // Go back to the root node's right child
        let left_child = walker.go_parent();
        assert_eq!(left_child, Some(&tree.values[2]));
    }

    #[test]
    fn parent() {
        let left = [1, 3, 5, -1, -1, -1, -1];
        let right = [2, 4, 6, -1, -1, -1, -1];
        let values = [10, 51, 36, 90, 32, 16, 5];
        let tree = BTree::new(left, right, values);

        // Root's parent
        assert_eq!(tree.parent(0), None);

        // Overflow - nonexistent node
        assert_eq!(tree.parent(values.len() as isize), None);

        // Valid use cases
        assert_eq!(tree.parent(1).unwrap(), ROOT_NODE);
        assert_eq!(tree.parent(3).unwrap(), 1);
        assert_eq!(tree.parent(4).unwrap(), 1);
        assert_eq!(tree.parent(5).unwrap(), 2);
        assert_eq!(tree.parent(6).unwrap(), 2);
    }

    #[test]
    fn children() {
        let left = [1, 3, 5, -1, -1, -1, -1];
        let right = [2, 4, 6, -1, -1, -1, -1];
        let values = [10, 51, 36, 90, 32, 16, 5];
        let tree = BTree::new(left, right, values);
        assert_eq!(tree.children(0), (1, 2).into());
        assert_eq!(tree.children(2), (5, 6).into());
        assert_eq!(tree.children(6), (-1, -1).into());
    }

    #[test]
    fn leaf_node() {
        let left = [1, 3, 5, -1, -1, -1, -1];
        let right = [2, 4, 6, -1, -1, -1, -1];
        let values = [10, 51, 36, 90, 32, 16, 5];
        let tree = BTree::new(left, right, values);
        assert!(!tree.is_leaf_node(0));
        assert!(tree.is_leaf_node(3));
    }

    #[test]
    fn index() {
        let left = [1, 3, 5, -1, -1, -1, -1];
        let right = [2, 4, 6, -1, -1, -1, -1];
        let values = [10, 51, 36, 90, 32, 16, 5];
        let tree = BTree::new(left, right, values);
        assert_eq!(tree[0], 10);
        assert_eq!(tree[3], 90);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde() {
        let left = [1, 3, 5, -1, -1, -1, -1];
        let right = [2, 4, 6, -1, -1, -1, -1];
        let values = [10, 51, 36, 90, 32, 16, 5];
        let tree = BTree::new(left, right, values);
        let string_repr = serde_json::to_string(&tree).unwrap();
        let deserialized_tree = serde_json::from_str::<BTree<i32, 7>>(&string_repr).unwrap();
        assert_eq!(tree, deserialized_tree);
    }
}
