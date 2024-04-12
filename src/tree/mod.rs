#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::sized::{LEAF_NODE, ROOT_NODE};
use crate::tree::TreeError::CorruptedTree;

#[derive(Debug, Clone)]
pub enum TreeError {
    CorruptedTree(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tree<T> {
    nodes: Vec<Vec<isize>>,
    values: Vec<T>,
}

impl<T> Tree<T> {
    pub fn new(nodes: Vec<Vec<isize>>, values: Vec<T>) -> Result<Self, TreeError> {
        if nodes
            .iter()
            .any(|nodes_vec| nodes_vec.len() != values.len())
        {
            Err(CorruptedTree(format!(
                "Tree nodes and values length do not match. Expected length: {}",
                values.len()
            )))
        } else {
            Ok(Self { nodes, values })
        }
    }
    /// True if given `node_id` is a leaf node (no children), false otherwise.
    ///
    ///
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::tree::Tree;
    ///         let left = vec![1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let mid = vec![2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let right = vec![3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    ///         let tree = Tree::new(vec![left, mid, right], values).expect("Tree has a valid structure");
    ///         assert!(!tree.is_leaf_node(0));
    ///         assert!(tree.is_leaf_node(4));
    /// ```
    pub fn is_leaf_node(&self, node_id: usize) -> bool {
        self.nodes
            .iter()
            .enumerate()
            .all(|(m, _)| self.nodes[m][node_id] == LEAF_NODE)
    }

    /// Returns a [Vec] of size `n` with node's children indices, or [LEAF_NODE] as a placeholder for every missing child.
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::tree::Tree;
    ///         let left = vec![1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let mid = vec![2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let right = vec![3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    ///         let tree = Tree::new(vec![left, mid, right], values).expect("Tree has a valid structure");
    ///         assert_eq!(tree.children(0), vec![1, 2, 3]);
    ///         assert_eq!(tree.children(2), vec![7, 8, 9]);
    ///         assert_eq!(tree.children(6), vec![-1, -1, -1]); // Leaf node, no children
    /// ```
    pub fn children(&self, node_id: usize) -> Vec<isize> {
        self.nodes
            .iter()
            .map(|dimension| dimension[node_id])
            .collect()
    }

    /// Returns index of a node's parent, if the node has a parent. `None` otherwise.
    /// E.g. root nodes don't have a parent.
    ///
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::sized::ROOT_NODE;
    ///         use treesome::tree::Tree;
    ///         let left = vec![1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let mid = vec![2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let right = vec![3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    ///         let tree_dimension = values.len();
    ///         let tree = Tree::new(vec![left, mid, right], values).expect("Tree has a valid structure");
    ///
    ///        ///Root's parent
    ///        assert_eq!(tree.parent(0), None);
    ///
    ///         // Overflow - nonexistent node
    ///         assert_eq!(tree.parent(tree_dimension as isize), None);
    ///
    ///         // Valid use cases
    ///         assert_eq!(tree.parent(1).unwrap(), ROOT_NODE);
    ///         assert_eq!(tree.parent(3).unwrap(), ROOT_NODE);
    ///         assert_eq!(tree.parent(4).unwrap(), 1);
    ///         assert_eq!(tree.parent(7).unwrap(), 2);
    ///         assert_eq!(tree.parent(9).unwrap(), 2);
    ///         assert_eq!(tree.parent(10).unwrap(), 3);
    /// ```
    pub fn parent(&self, node_id: isize) -> Option<isize> {
        if node_id <= ROOT_NODE || node_id as usize >= self.values.len() {
            return None; // Root node doesn't have a parent.
        };

        let tree_dimension = self.nodes.len();
        Some((node_id - 1) / tree_dimension as isize)
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::Tree;

    #[test]
    fn new_validation() {
        let left = vec![1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
        let mid = vec![2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
        let right = vec![3, 6, 9, 12]; // One dimension shorter on purpose to test the validation
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let err = Tree::new(vec![left, mid, right], values);

        if err.is_ok() {
            panic!("Tree structure validation shouldn't have passed due to input length error.")
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde() {
        let left = vec![1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
        let mid = vec![2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
        let right = vec![3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let tree = Tree::new(vec![left, mid, right], values).expect("Tree has a valid structure");
        let string_repr = serde_json::to_string(&tree).unwrap();
        let deserialized_tree: Tree<i32> = serde_json::from_str(&string_repr).unwrap();
        assert_eq!(tree, deserialized_tree);
    }
}
