use std::ops::Index;

const LEAF_NODE_MARK: isize = -1;

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
    pub fn children(&self, node_id: usize) -> Children {
        Children {
            left: self.l_nodes[node_id],
            right: self.r_nodes[node_id],
        }
    }

    pub fn parent(&self, node_id: isize) -> isize {
        let parent_level = (node_id + 1).ilog2() - 1; // On which level the parent node is. Root is level 0.
        let level_start_idx = 2_isize.pow(parent_level) - 1; // Whe the parent's level start in the backing arrays

        // The parent's index is starting offset of the tree's level where the parent resides + offset on that level.
        // Offset on that level is calculated as follows:
        // The `node_id value represents node's offset from the tree's root. By subtracting the amount of nodes before current level,
        // the result `x` shows current node is `nth` from the beginning of the current level. As this is a binary tree, and there are exactly
        // double the amount of nodes in each level, dividing `x` by `2` results in previous level's offset.
        let nodes_before_current_level = 2_isize.pow(parent_level + 1) - 1; // How many nodes are there on levels below `node_id`'s level
        let level_offset = (node_id - nodes_before_current_level) / 2;

        // Combine the level start with offset on that level gives exact coordinates
        level_start_idx + level_offset
    }
}

impl<T, const N: usize> Index<usize> for BTree<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::btree::BTree;

    #[test]
    fn it_works() {}

    #[test]
    fn parent() {
        let left = [1, 3, 5, -1, -1, -1, -1];
        let right = [2, 4, 6, -1, -1, -1, -1];
        let values = [10, 51, 36, 90, 32, 16, 5];
        let tree = BTree::new(left, right, values);

        assert_eq!(tree.parent(3), 1);
        assert_eq!(tree.parent(4), 1);
        assert_eq!(tree.parent(5), 2);
        assert_eq!(tree.parent(6), 2);
        // Works even if out of bounds, the result is based on mere calculation, the function doesn't
        // do a bound check to ensure fast tree traversal times. The caller is responsible for providing
        // a node_id actually present in the tree.
        assert_eq!(tree.parent(7), 3);
        assert_eq!(tree.parent(8), 3);
        assert_eq!(tree.parent(9), 4);
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
