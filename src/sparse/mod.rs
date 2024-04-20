use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A growable, non-shrinkable n-ary tree. Traversable in both ways. Suitable for sparse tree structures, at the cost of extra
/// runtime overhead (reference counting).
///
/// ## Node detachment/deletion
/// Node detachment is not possible once nodes are linked together via `create_child`. It'd be at the cost of
/// adding interior mutability to reference-counted nodes, which would lead to runtime interior mutability checks on
/// every node interaction.
///
/// Node deletion can be achieved by:
/// a) Traversing the whole tree while creating a new one with selected nodes only, dropping the old tree if necessary, or
/// b) Delay child node insertion during tree creation until 100% sure the node should be part of the graph.
///
///
/// ## Thread safety
/// Not thread safe (Sync), as this type uses non-atomic reference counting internally to increase speed.
///
/// ## Future work
/// Serialization is unnecessarily expensive. Custom serialization, representing the tree only one way (root -> leaf),
/// without the child -> parent link should be implemented to make the resulting structure more compact.
/// In cases where the resulting tree is "dense enough", converting it to [crate::tree::Tree] would be the most efficient.
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Node<T> {
    parent: Option<Weak<Node<T>>>,
    children: RefCell<Vec<Rc<Node<T>>>>,
    pub value: T,
    this: Weak<Self>,
}

impl<T> Node<T> {
    /// Starts a new tree by creating a root node with no parent.
    ///
    /// # Examples
    /// ```
    ///         use treesome::sparse::Node;
    ///         let root = Node::root(42);
    ///
    ///         assert!(root.is_leaf()); // A stump, really
    ///         assert_eq!(root.value, 42);
    /// ```
    pub fn root(value: T) -> Rc<Self> {
        Rc::new_cyclic(|node| Self {
            parent: None,
            children: RefCell::new(Vec::new()),
            value,
            this: node.clone(),
        })
    }

    /// Creates a new child node bound to this node.
    ///
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::sparse::Node;
    ///         let root = Node::root(42);
    ///         let child = root.create_child(43);
    ///
    ///         assert_eq!(root.children().len(), 1);
    ///         assert_eq!(root.children().first().expect("One child node was created").value, child.value);
    /// ```
    ///
    ///
    pub fn create_child(&self, value: T) -> Rc<Node<T>> {
        let child = Rc::new_cyclic(|child| Self {
            parent: Some(self.this.clone()),
            children: RefCell::new(Vec::new()),
            value,
            this: child.clone(),
        });
        self.children.borrow_mut().push(child.clone());

        child
    }

    pub fn is_leaf(&self) -> bool {
        self.children.borrow().is_empty()
    }

    /// Returns node's parent, if it has one
    ///
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::sparse::Node;
    ///         let root = Node::root(42);
    ///         let child = root.create_child(43);
    ///
    ///         assert_eq!(child.parent().unwrap().value, root.value);
    /// ```
    ///
    ///
    pub fn parent(&self) -> Option<Rc<Node<T>>> {
        self.parent.as_ref()?.upgrade()
    }

    /// Returns a newly allocated vector of node's current children. The vector doesn't
    /// reflect any posterior changes.
    ///
    /// # Examples
    ///
    /// ```
    ///         use treesome::sparse::Node;
    ///         let root = Node::root(42);
    ///         let child = root.create_child(43);
    ///
    ///         assert_eq!(root.children().len(), 1);
    /// ```
    pub fn children(&self) -> Vec<Rc<Node<T>>> {
        self.children.borrow().iter().cloned().collect()
    }
}
