use std::ops::Index;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::sized::structs::Array;

pub const LEAF_NODE: isize = -1;
pub const ROOT_NODE: isize = 0;

/// N-ary tree, using an array representation of nodes and edges internally. Suitable for dense graphs
/// and fast serialization/deserialization.
#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tree<T, const M: usize, const N: usize> {
    nodes: Array<Array<isize, N>, M>,
    values: Array<T, N>,
}

impl<T, const M: usize, const N: usize> Tree<T, M, N> {
    pub fn from_arrays(nodes: Array<Array<isize, N>, M>, values: Array<T, N>) -> Self {
        Self { nodes, values }
    }
    pub fn new(nodes: [[isize; N]; M], values: [T; N]) -> Self {
        let node_indices: Vec<Array<isize, N>> = nodes
            .into_iter()
            .map(|node| {
                let arr: Array<isize, N> = node.into();
                arr
            })
            .collect();
        let node_indices_array: [Array<isize, N>; M] =
            node_indices.try_into().unwrap_or_else(|_| {
                unreachable!("Input size is guaranteed by constant generic args <M,N>")
            });
        let array_of_nodes: Array<Array<isize, N>, M> = node_indices_array.into();
        Self {
            nodes: array_of_nodes,
            values: values.into(),
        }
    }

    /// True if given `node_id` is a leaf node (no children), false otherwise.
    ///
    ///
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::sized::Tree;
    ///         let left = [1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let mid = [2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let right = [3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    ///         let tree = Tree::new([left, mid, right], values);
    ///         assert!(!tree.is_leaf_node(0));
    ///         assert!(tree.is_leaf_node(4));
    /// ```
    pub fn is_leaf_node(&self, node_id: usize) -> bool {
        self.nodes
            .iter()
            .enumerate()
            .all(|(m, _)| self.nodes[m][node_id] == LEAF_NODE)
    }

    /// Returns an array of size [M] with node's children indices, or [LEAF_NODE] as a placeholder for every missing child.
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::sized::Tree;
    ///         let left = [1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let mid = [2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let right = [3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
    ///         let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    ///         let tree = Tree::new([left, mid, right], values);
    ///         assert_eq!(tree.children(0), [1, 2, 3]);
    ///         assert_eq!(tree.children(2), [7, 8, 9]);
    ///         assert_eq!(tree.children(6), [-1, -1, -1]); // Leaf node, no children
    /// ```
    pub fn children(&self, node_id: usize) -> [isize; M] {
        let mut children = [0_isize; M];
        for (m, _) in self.nodes.iter().enumerate() {
            children[m] = self.nodes[m][node_id];
        }
        children
    }

    /// Returns index of a node's parent, if the node has a parent. `None` otherwise.
    /// E.g. root nodes don't have a parent.
    ///
    ///
    /// # Examples
    ///
    /// ```
    ///        use treesome::sized::{ROOT_NODE, Tree};
    ///        let left = [1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
    ///        let mid = [2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
    ///        let right = [3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
    ///        let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    ///        let tree = Tree::new([left, mid, right], values);
    ///
    ///        ///Root's parent
    ///        assert_eq!(tree.parent(0), None);
    ///
    ///         // Overflow - nonexistent node
    ///         assert_eq!(tree.parent(values.len() as isize), None);
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

        Some((node_id - 1) / M as isize)
    }
}

impl<T, const M: usize, const N: usize> Index<usize> for Tree<T, M, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::sized::Tree;

    #[test]
    fn index() {
        let left = [1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
        let mid = [2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
        let right = [3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
        let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let tree = Tree::new([left, mid, right], values);
        assert_eq!(tree[0], 1);
        assert_eq!(tree[3], 4);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde() {
        let left = [1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1];
        let mid = [2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1];
        let right = [3, 6, 9, 12, -1, -1, -1, -1, -1, -1, -1, -1];
        let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let tree = Tree::new([left, mid, right], values);
        let string_repr = serde_json::to_string(&tree).unwrap();
        let deserialized_tree = serde_json::from_str::<Tree<i32, 3, 12>>(&string_repr).unwrap();
        assert_eq!(tree, deserialized_tree);
    }
}
