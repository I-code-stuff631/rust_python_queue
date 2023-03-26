use pyo3::{prelude::*, types::PyFunction};
#[allow(unused_imports)]
use pyo3::exceptions::PyIndexError;
#[allow(unused_imports)]
use std::cmp::Ordering::*;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::iter::FusedIterator;


struct Node {
    /// This should NOT be used for comparisions, use value instead
    item: Py<PyAny>,
    /// This should be used for any comparisions
    value: Py<PyAny>,
    parent: Weak<RefCell<Node>>,
    left: Option<WNode>,
    right: Option<WNode>,
}

impl Node {
    /// Returns the leftmost node from the passed node
    fn leftmost(node: &WNode) -> WNode {
        let mut next: *const Rc<RefCell<Node>> = node;
        while let Some(left_node) = unsafe {
            // SAFETY: next points to an Rc whos contents have not been dropped
            let node_ptr: *const Node = (*next).as_ptr(); // Saves dynamic borrow check
            // SAFETY: If there are refs to this node stored somewhere the immutability guarantee
            // is still upheld since we dont mutate it
            (*node_ptr).left.as_ptr() // I use as_ptr here because the pointer is not guaranteed to be valid
            // (ie. it could be invalidated via '(*left_node).borrow().parent.upgrade().unwrap().borrow_mut().left = None')
        } {
            next = left_node;
        }
        unsafe {
            // SAFETY: next points to an Rc whos contents have not been dropped
            Rc::clone(&*next)
        }
    }

    /// Returns the rightmost node from the passed node
    fn rightmost(node: &WNode) -> WNode {
        let mut next: *const Rc<RefCell<Node>> = node;
        while let Some(right_node) = unsafe {
            // SAFETY: next points to an Rc whos contents have not been dropped
            let node_ptr: *const Node = (*next).as_ptr(); // Saves dynamic borrow check
            // SAFETY: If there are refs to this node stored somewhere the immutability guarantee
            // is still upheld since we dont mutate it
            (*node_ptr).right.as_ptr() // I use as_ptr here because the pointer is not guaranteed to be valid (ie. it
            // could be invalidated via '(*right_node).borrow().parent.upgrade().unwrap().borrow_mut().right = None')
        } {
            next = right_node;
        }
        unsafe {
            // SAFETY: next points to an Rc whos contents have not been dropped
            Rc::clone(&*next)
        }
    }
}

/// Wrapped node
type WNode = Rc<RefCell<Node>>;

/// Double ended priority queue implmented with an explicit binary search tree
#[pyclass(sequence, unsendable)]
struct DoublePriorityQueue {
    root: Option<WNode>,
    get_cmpison_value: Option<Py<PyFunction>>,
    length: usize,
}

#[pymethods]
impl DoublePriorityQueue {
    /// comparison_value defines for each item the value that will be used for comparisions.
    /// If it is None then the item will be used as this comparision value.
    #[new]
    fn new(comparison_value: Option<Py<PyFunction>>) -> Self {
        Self {
            root: None,
            get_cmpison_value: comparison_value,
            length: 0,
        }
    }

    /// Pushes the specified item onto the queue.
    /// It is a logical error to modify the item in such a way that its comparison value
    /// would change after it has been pushed.
    fn push(&mut self, py: Python<'_>, item: &PyAny) -> PyResult<()>{
        let mut new_node = Node {
            item: item.into_py(py),
            value: self.get_comparison_value_for(item)?,
            parent: Weak::new(),
            left: None,
            right: None,
        };
        #[inline]
        fn wrap_node(node: Node) -> WNode {
            Rc::new(RefCell::new(node))
        }

        match &self.root {
            Some(root_node) => {
                let mut next: *const Rc<RefCell<Node>> = root_node;
                loop {
                    let mut node = unsafe {
                        // SAFETY: The Rc to which next points has not been dropped
                        (*next).borrow_mut()
                    };
                    next = match new_node.value.as_ref(py).compare(&node.value)? {
                        Less => match &node.left {
                            Some(left_node) => left_node,
                            None => {
                                new_node.parent = unsafe { Rc::downgrade(&*next) };
                                node.left = Some(wrap_node(new_node));
                                break;
                            },
                        }
                        Equal | Greater => match &node.right {
                            Some(right_node) => right_node,
                            None => {
                                new_node.parent = unsafe { Rc::downgrade(&*next) };
                                node.right = Some(wrap_node(new_node));
                                break;
                            },
                        }
                    }
                };
            },
            None => self.root = Some(wrap_node(new_node)),
        }

        self.length += 1;
        Ok(())
    }

    /// Pops the next item with the greatest priority off the queue, will return None if the queue is empty.
    fn pop_max(&mut self) -> Option<Py<PyAny>> {
        self.greatest_node().map(|greatest_node| {
            { // Remove the node from the tree
                let mut greatest_node = greatest_node.borrow_mut();
                match greatest_node.parent.upgrade() {
                    Some(parent) => {
                        parent.borrow_mut().right = greatest_node.left.take().map(|left_node| {
                            left_node.borrow_mut().parent = Rc::downgrade(&parent); // Update parent node
                            left_node
                        });
                    }
                    None => {
                        // The greatest node is the root node
                        self.root = greatest_node.left.take().map(|new_root| {
                            new_root.borrow_mut().parent = Weak::new();
                            new_root
                        });
                    }
                }
            };

            self.length -= 1;
            Rc::try_unwrap(greatest_node).ok().expect("The node should no longer be in the tree").into_inner().item
        })
    }

    /// Pops the next item with the lowest priority off the queue, will return None if the queue is empty.
    fn pop_min(&mut self) -> Option<Py<PyAny>> {
        self.least_node().map(|least_node| {
            { // Remove the node from the tree
                let mut least_node = least_node.borrow_mut();
                match least_node.parent.upgrade() {
                    Some(parent) => {
                        parent.borrow_mut().left = least_node.right.take().map(|right_node| {
                            right_node.borrow_mut().parent = Rc::downgrade(&parent); // Update parent node
                            right_node
                        });
                    }
                    None => {
                        // The least node is the root node
                        self.root = least_node.right.take().map(|new_root| {
                            new_root.borrow_mut().parent = Weak::new();
                            new_root
                        });
                    }
                }
            };

            self.length -= 1;
            Rc::try_unwrap(least_node).ok().expect("The node should no longer be in the tree").into_inner().item
        })
    }

    /// Pops off an item that satisfys the condition and is closest in value to the item specified.
    /// Returns None if no such item exists.
    fn pop_with_if_closest(&mut self, item: &PyAny, condition: &PyFunction) -> Option<Py<PyAny>> {
        todo!();
    }

    /// Access the next item with the greatest priority without removing it from the queue,
    /// this method will return None if the queue is empty.
    /// It is a logical error to modify the item returned by this method in such a way that its
    /// comparison value would change.
    fn peek_max(&self) -> Option<Py<PyAny>> {
        self.greatest_node().map(|node| Py::clone(&node.borrow().item))
    }

    /// Access the next item with the lowest priority without removing it from the queue,
    /// this method will return None if the queue is empty.
    /// It is a logical error to modify the item returned by this method in such a way that its
    /// comparison value would change.
    fn peek_min(&self) -> Option<Py<PyAny>> {
        self.least_node().map(|node| Py::clone(&node.borrow().item))
    }

    // fn __contains__(&self, py: Python<'_>, item: &PyAny) -> PyResult<bool> {
    //     let value = self.get_comparison_value_for(item)?;

    //     let mut next = self.root.as_ptr();
    //     if let Some(node) = next {}
    //     // match &self.root {
    //     //     Some(root_node) => {
    //     //         todo!();
    //     //     },
    //     //     None => Ok(false),
    //     // }
    // }

    // /// Ez impl by calling __delitem__ when it is impled
    // fn remove(&mut self) { todo!() }

    fn clear(&mut self) { 
        self.root = None;
        self.length = 0;
    }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    fn __len__(&self) -> usize {
        self.length
    }

    fn __iter__(&self) -> PyIter {
        PyIter(self.into_iter())
    }

    fn __delitem__(&mut self, index: usize) -> PyResult<()> {
        let node_for_removal = self.node_at(index)?;
        // This is effectively a Ref because it is not declared as mut
        let node_for_removal_ref = node_for_removal.borrow_mut();

        match node_for_removal_ref.parent.upgrade() {
            Some(parent_node) => {
                let replacement = {
                    let mut node_for_removal = node_for_removal_ref;
                    if let Some(left_node) = node_for_removal.left.take() {
                        match node_for_removal.right.take() {
                            Some(right_node) => {
                                Node::leftmost(&right_node).borrow_mut().left = Some(left_node);
                                Some(right_node)
                            },
                            None => Some(left_node),
                        }
                    } else if let Some(right_node) = node_for_removal.right.take() {
                        match node_for_removal.left.take() {
                            Some(left_node) => {
                                Node::leftmost(&right_node).borrow_mut().left = Some(left_node);
                                Some(right_node)
                            },
                            None => Some(right_node),
                        }
                    } else {
                        None
                    }
                };
                // What side of its parent is node_for_removal on?
                let mut parent_node = parent_node.borrow_mut();
                if let Some(right_node) = parent_node.right.clone() {
                    if Rc::ptr_eq(&node_for_removal, &right_node) { // It is on the right
                        parent_node.right = replacement;
                    } else { // It should be on the left
                        debug_assert!(parent_node.left.is_some());
                        parent_node.left = replacement;
                    }
                } else { // It should be on the left
                    debug_assert!(parent_node.left.is_some());
                    parent_node.left = replacement;
                }
            }
            None => { // The node for removal is the root node
                let mut node_for_removal = node_for_removal_ref;
                // If node for removal has a left or right node then it will be the new root
                self.root = if let Some(new_root) = node_for_removal.left.take() {
                    new_root.borrow_mut().parent = Weak::new();
                    Node::rightmost(&new_root).borrow_mut().right = node_for_removal.right.take();
                    Some(new_root)
                } else if let Some(new_root) = node_for_removal.right.take() {
                    new_root.borrow_mut().parent = Weak::new();
                    Node::leftmost(&new_root).borrow_mut().left = node_for_removal.left.take();
                    Some(new_root)
                } else {
                    None
                };
            }
        }

        debug_assert_eq!(1, Rc::strong_count(&node_for_removal)); // node has been removed from the tree
        self.length -= 1;
        Ok(())
    }

    fn __getitem__(&self, index: usize) -> PyResult<Py<PyAny>> {
        self.node_at(index).map(|node| Py::clone(&node.borrow().item))
    }

    // fn __setitem__(&self, index: usize) -> PyResult<Py<PyAny>>;

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        let mut string = String::with_capacity(2);
        string.push('[');

        let mut length_remaining = self.length;
        let mut iter = self.into_iter().yield_py_any();
        while let Some(item) = iter.next() {
            length_remaining -= 1;
            string.push_str(item.as_ref(py).str()?.to_str()?);
            if length_remaining != 0 { // Not the last item
                string.push_str(", ")
            }
        }

        string.push(']');
        Ok(string)
    }
}

impl IntoIterator for &DoublePriorityQueue {
    type Item = WNode;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        // If the tree was height-balanced this capacity could be smaller
        let mut stack = Vec::with_capacity(self.length);

        let mut next = self.root.clone();
        while let Some(node) = next {
            stack.push(Rc::downgrade(&node));
            next = node.borrow().right.clone();
        }

        Iter { stack }
    }
}

struct Iter {
    // # Weak refs should prob be used anyways
    // If the queue uses weak refs and just skips past any that can not be upgraded then the user does not know
    // which nodes, when popped, will cause the iter to skip sections.
    // If the queue uses weak refs and stops iteration as soon as it fails to upgrade one, then the user does not
    // know which nodes, when popped and reached in iteration, cause the iteration to stop early and which ones
    // will not cause that. (This one should be easier to program and also should preserve the users expectation
    // about the order of iteration by stopping when that expectation would be violated)
    stack: Vec<Weak<RefCell<Node>>>,
}

impl Iterator for Iter {
    type Item = WNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().and_then(|weakref| {
            match weakref.upgrade() {
                Some(node) => {
                    let mut next = node.borrow().left.clone();
                    while let Some(node) = next {
                        self.stack.push(Rc::downgrade(&node));
                        next = node.borrow().right.clone();
                    }

                    // Should be the current greatest node
                    Some(node)
                }
                None => {
                    // Stop iteration early
                    self.stack.clear();
                    self.stack.shrink_to_fit();
                    None
                }
            }
        })
    }
}
impl FusedIterator for Iter {}

/// Rust only
impl DoublePriorityQueue {
    fn get_comparison_value_for(&self, item: &PyAny) -> PyResult<Py<PyAny>> {
        let py = item.py();
        Ok(match &self.get_cmpison_value {
            Some(get_comparison_value) => get_comparison_value.call1(py, (item,))?,
            None => item.into_py(py),
        })
    }

    // Returns the rightmost node from the root.
    fn greatest_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.root.as_ref().map(|root_node| Node::rightmost(root_node))
    }

    // Returns the leftmost node from the root.
    fn least_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.root.as_ref().map(|root_node| Node::leftmost(root_node))
    }

    /// Gets the node at the specified index
    fn node_at(&self, index: usize) -> PyResult<WNode> {
        if index == 0 {
            self.greatest_node() // Saves an allocation
        } else if index == self.length.saturating_sub(1) {
            self.least_node() // Saves an allocation and the time it would take to iterate to the last item
        } else {
            self.into_iter().nth(index)
        }.ok_or_else(|| PyIndexError::new_err(format!("Index: {index} out of range!")))
    }
}

impl Iter {
    /// Causes the iterator to yield `Py<PyAny>` instead of `WNode`
    fn yield_py_any(self) -> PyIter { // This SHOULD be always being inlined as idk how it would be able to not be
        // (u might want to test that tho) #[repr(transparent)]?
        PyIter(self)
    }
}

/// Iter but it returns `Py<PyAny>` instead of `WNode`
#[pyclass(unsendable)]
struct PyIter(Iter);

impl Iterator for PyIter {
    type Item = Py<PyAny>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|node| Py::clone(&node.borrow().item))
    }
}
impl FusedIterator for PyIter {}

#[pymethods]
impl PyIter {
    fn __next__(&mut self) -> Option<<Self as Iterator>::Item> {
        self.next()
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn rust_queue(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DoublePriorityQueue>()?;
    Ok(())
}

trait AsPtr {
    type Type;

    fn as_ptr(&self) -> Self::Type;
}

impl<T> AsPtr for Option<T> {
    type Type = Option<*const T>;

    /// Converts from `&Option<T>` to `Option<*const T>`.
    #[inline]
    fn as_ptr(&self) -> Option<*const T> {
        match *self {
            Some(ref ptr) => Some(ptr), // Coerced to *const T
            None => None,
        }
    }
}
