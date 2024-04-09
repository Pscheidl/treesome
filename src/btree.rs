use std::ops::Index;

const LEAF_NODE_MARK: isize = -1;
const ROOT_NODE: isize = 0;

/// A binary tree representation for fast traversal, suitable for dense trees.
/// Special implementation for binary tree is offered for faster traversal times over the generalized
/// `k` tree, where an array of arrays and indices checks happen.
#[derive(Eq, PartialEq, Debug)]
pub struct BTree<T, const N: usize> {
    pub l_nodes: [isize; N],
    pub r_nodes: [isize; N],
    pub values: [T; N],
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
    ///         use treesome::btree::BTree;
    ///         let left = [1, 3, 5, -1, -1, -1, -1];
    ///         let right = [2, 4, 6, -1, -1, -1, -1];
    ///         let values = [10, 51, 36, 90, 32, 16, 5];
    ///         let tree = BTree::new(left, right, values);
    /// ```
    pub fn new(l_nodes: [isize; N], r_nodes: [isize; N], values: [T; N]) -> Self {
        Self {
            l_nodes,
            r_nodes,
            values,
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
    ///         use treesome::btree::BTree;
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
    ///         use treesome::btree::BTree;
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
            return None;
        }; // Root node doesn't have a parent.
        let parent_level = (node_id + 1).checked_ilog2()? - 1; // On which level the parent node is. Root is level 0.
        let level_start_idx = 2_isize.pow(parent_level) - 1; // Whe the parent's level start in the backing arrays

        // The parent's index is starting offset of the tree's level where the parent resides + offset on that level.
        // Offset on that level is calculated as follows:
        // The `node_id value represents node's offset from the tree's root. By subtracting the amount of nodes before current level,
        // the result `x` shows current node is `nth` from the beginning of the current level. As this is a binary tree, and there are exactly
        // double the amount of nodes in each level, dividing `x` by `2` results in previous level's offset.
        let nodes_before_current_level = 2_isize.pow(parent_level + 1) - 1; // How many nodes are there on levels below `node_id`'s level
        let level_offset = (node_id - nodes_before_current_level) / 2;

        // Combine the level start with offset on that level gives exact coordinates
        Some(level_start_idx + level_offset)
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
///        use treesome::btree::{BTree, Walker};
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
    use crate::btree::{BTree, Walker, ROOT_NODE};

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
}
