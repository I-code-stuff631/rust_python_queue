use pyo3::{prelude::*, types::PyFunction};
#[allow(unused_imports)]
use pyo3::exceptions::PyIndexError;
#[allow(unused_imports)]
use std::cmp::Ordering::*;
use std::rc::{Rc, Weak};
use std::cell::RefCell;


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
        unsafe {
            // SAFETY: The Rc to which next points is not dropped
            while let Some(left_node) = &(*next).borrow().left {
                next = left_node;
            }
            Rc::clone(&*next)
        }
    }

    /// Returns the rightmost node from the passed node
    fn rightmost(node: &WNode) -> WNode {
        let mut next: *const Rc<RefCell<Node>> = node;
        unsafe {
            // SAFETY: The Rc to which next points is not dropped
            while let Some(right_node) = &(*next).borrow().right {
                next = right_node;
            }
            Rc::clone(&*next)
        }
    }
}

type WNode = Rc<RefCell<Node>>;

/// PriorityQueue implmented with an explicit binary search tree
#[pyclass(sequence, unsendable)]
struct PriorityQueue {
    root: Option<WNode>,
    get_cmpison_value: Option<Py<PyFunction>>,
    length: usize,
}

#[pymethods]
impl PriorityQueue {
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
                        // SAFETY: The Rc to which next points is not dropped
                        (*next).borrow_mut()
                    };
                    next = match new_node.value.as_ref(py).compare(&node.value)? {
                        Less => match &node.left {
                            Some(left_node) => left_node,
                            None => {
                                new_node.parent = unsafe { Rc::downgrade(&*next) };
                                node.left = Some(Rc::new(RefCell::new(new_node)));
                                break;
                            },
                        }
                        Equal | Greater => match &node.right {
                            Some(right_node) => right_node,
                            None => {
                                new_node.parent = unsafe { Rc::downgrade(&*next) };
                                node.right = Some(Rc::new(RefCell::new(new_node)));
                                break;
                            },
                        }
                    }
                };
            },
            None => self.root = Some(Rc::new(RefCell::new(new_node))),
        }

        self.length += 1;
        Ok(())
    }

    /// Pops the next item off the queue, will return None if the queue is empty.
    fn pop(&mut self) -> Option<Py<PyAny>> {
        self.greatest_node().map(|greatest_node| {
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
                        new_root.borrow_mut().parent = Weak::new(); // It is the root node so it has no parent
                        new_root
                    });
                }
            }

            self.length -= 1;
            // Now that greatest node is out of the queue
            Py::clone(&greatest_node.item)
        })
    }

    /// Access the next item without removing it from the queue, this method will return None if the queue 
    /// is empty.
    /// It is a logical error to modify the item returned by this method in such a way that its
    /// comparison value would change.
    fn peek(&self) -> Option<Py<PyAny>> {
        self.greatest_node().map(|node| Py::clone(&node.borrow().item))
    }

    // fn __contains__(&self, py: Python<'_>, item: &PyAny) -> PyResult<bool> {
    //     let value = self.get_comparison_value_for(item)?;

    //     match &self.root {
    //         Some(root_node) => {
    //         },
    //         None => Ok(false),
    //     }
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
        let node_for_removal = if index == 0 {
            self.greatest_node() // Saves an allocation
        } else if index == self.length.saturating_sub(1) {
            self.least_node() // Saves an allocation and the time it would take to iterate to the last item
        } else {
            self.into_iter().nth(index)
        }.ok_or_else(|| PyIndexError::new_err(format!("Index: {index} out of range!")))?;
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

        self.length -= 1;
        Ok(())
    }

    fn __getitem__(&self, index: usize) -> PyResult<Py<PyAny>> {
        if index == 0 {
            self.greatest_node() // Saves an allocation
        } else if index == self.length.saturating_sub(1) {
            self.least_node() // Saves an allocation and the time it would take to iterate to the last item
        } else {
            self.into_iter().nth(index)
        }.map(|node| Py::clone(&node.borrow().item))
        .ok_or_else(|| PyIndexError::new_err(format!("Index: {index} out of range!")))
    }

    // fn __setitem__(&self, index: usize) -> PyResult<Py<PyAny>>;

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        let mut string = String::with_capacity(2);
        string.push('[');

        let mut length_remaining = self.length;
        let mut iter = self.into_iter().into_py_any();
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

impl IntoIterator for &PriorityQueue {
    type Item = WNode;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        // If the tree was height-balanced this capacity could be smaller
        let mut stack = Vec::with_capacity(self.length);

        let mut next = self.root.clone();
        while let Some(node) = next {
            next = (*node).borrow().right.clone(); // Will be pushed on in next iteration
            stack.push(node);
        }

        IntoIter { stack }
    }
}

struct IntoIter {
    // If the iter uses strong refs the user does not know which nodes, when .pop'ed, will still be yielded by
    // the iter and which ones will not.
    // If when the nodes are pop'ed their left IS .taken then the user does not know which nodes, when .pop'ed,
    // will both still be yielded by the iter and cause other nodes to be skipped.

    // If the queue uses weak refs and just skips past any that can not be upgraded then the user does not know
    // which nodes, when popped, will cause the iter to skip sections.
    // If the queue uses weak refs and stops iteration as soon as it fails to upgrade one, then the user does not
    // know which nodes, when popped and reached in iteration, cause the iteration to stop early and which ones
    // will not cause that.
    stack: Vec<Rc<RefCell<Node>>>,
}

impl Iterator for IntoIter {
    type Item = WNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            let mut next = node.borrow().left.clone();
            while let Some(node) = next {
                next = node.borrow().right.clone(); // Will be pushed on in next iteration
                self.stack.push(node);
            }
            
            // Should always be the current greatest node
            node
        })
    }
}

/// Rust only
impl PriorityQueue {
    fn get_comparison_value_for(&self, item: &PyAny) -> PyResult<Py<PyAny>> {
        let py = item.py();
        Ok(match &self.get_cmpison_value {
            Some(get_comparison_value) => get_comparison_value.call1(py, (item,))?,
            None => item.into_py(py),
        })
    }

    // Gets the greatest node in the tree without allocating.
    fn greatest_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.root.as_ref().map(|root_node| Node::rightmost(root_node))
    }

    // Gets the least node in the tree without allocating.
    fn least_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.root.as_ref().map(|root_node| Node::leftmost(root_node))
    }
}

impl IntoIter {
    /// Return `Py<PyAny>` instead of `WNode`
    fn into_py_any(self) -> PyIter {
        PyIter(self)
    }
}

/// IntoIter but it returns `Py<PyAny>` instead of `WNode`
#[pyclass(unsendable)]
struct PyIter(IntoIter);

impl Iterator for PyIter {
    type Item = Py<PyAny>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|node| Py::clone(&node.borrow().item))
    }
}

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
    m.add_class::<PriorityQueue>()?;
    Ok(())
}
