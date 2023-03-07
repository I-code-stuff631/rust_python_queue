use pyo3::{prelude::*, types::PyFunction};
use std::cmp::Ordering::*;
use std::ptr;

struct Node {
    /// This should NOT be used for comparisions, use value instead
    item: Py<PyAny>,
    /// This should be used for any comparisions
    value: Py<PyAny>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

/// PriorityQueue implmented with an explicit binary search tree
#[pyclass]
struct PriorityQueue {
    root: Option<Box<Node>>,
    get_cmpison_value: Option<Py<PyFunction>>,
    length: usize,
}

#[pymethods]
impl PriorityQueue {
    /// comparison_value defines for each item the value that will be used for comparisions.
    /// If it is None then the item will be used for comparisions.
    #[new]
    fn new(comparison_value: Option<Py<PyFunction>>) -> Self {
        Self {
            root: None,
            get_cmpison_value: comparison_value,
            length: 0
        }
    }

    /// Pushes the specified item onto the queue. It is a logical error to modify the item after it has been pushed.
    fn push(&mut self, py: Python<'_>, item: &PyAny) -> PyResult<()>{
        let new_node = Box::new(Node {
            item: item.into_py(py),
            value: match &self.get_cmpison_value {
                Some(get_comparison_value) => get_comparison_value.call1(py, (item,))?,
                None => item.into_py(py),
            },
            left: None,
            right: None,
        });
        let mut next = &mut self.root;
        if let Some(node) = next {
            match new_node.value.as_ref(py).compare(&node.value)? {
                Less => next = &mut node.left,
                Equal | Greater => next = &mut node.right,
            }
        }
        debug_assert!(next.is_none());
        *next = Some(new_node);
        self.length += 1;
        Ok(())
    }

    /// Pops the next item off the queue
    fn pop(&mut self) -> Option<Py<PyAny>> { todo!() }
    
    fn clear(&mut self) { 
        self.root = None; // Fine for now, may be tail recursion
        self.length = 0;
     }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    fn __contains__(&self, py: Python<'_>, item: &PyAny) -> PyResult<bool> {
        let item = match &self.get_cmpison_value {
            Some(get_comparison_value) => get_comparison_value.call1(py, (item,))?,
            None => item.into_py(py),
        };
        let mut next = &self.root;
        while let Some(node) = next {
            match item.as_ref(py).compare(&node.value)? {
                Less => next = &node.left,
                Greater => next = &node.right,
                Equal => return Ok(true), // We found a match
            }
        };
        Ok(false)
    }
    
    /// Access the next item without removing it from the queue.
    /// It is a logical error to modify the item returned by this method.
    fn peek(&self) -> Option<&Py<PyAny>> {
        self.greatest_node().map(|node| &node.item)
    }

    fn __len__(&self) -> usize {
        self.length
    }
}

/// Rust only
impl PriorityQueue {
    /// Returns a ref to the node with the highest value in the tree
    fn greatest_node(&self) -> Option<&Node> {
        self.root.as_deref().map(|root_node| {
            let mut next = root_node;
            while let Some(right_node) = &next.right {
                next = right_node;
            };
            next
        })
    }

    /// Returns a ref to the parent node of *node*.
    /// Returns None if *node* has no parent (ie. is the root node).
    /// # Panics
    /// If *node* is not in the tree
    fn parent_of(&self, node: &Node, py: Python<'_>) -> PyResult<Option<&Node>> {
        let mut next = self.root.as_deref();
        if let Some(root_node) = next {
            if ptr::eq(node, root_node) {
                return Ok(None);
            }
        } else {
            panic!("The node is not in the tree!");
        }
        while let Some(next_node) = next {
            match node.value.as_ref(py).compare(&next_node.value)? {
                Less => if let Some(left_node) = next_node.left.as_deref() {
                    if ptr::eq(node, left_node) {
                        break;
                    } else {
                        next = Some(left_node);
                    }
                } else {
                    panic!("The node is not in the tree!");
                }
                Equal | Greater => if let Some(right_node) = next_node.right.as_deref() {
                    if ptr::eq(node, right_node) {
                        break;
                    } else {
                        next = Some(right_node);
                    }
                } else {
                    panic!("The node is not in the tree!");
                }
            }
        }
        Ok(next)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn python_extension(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PriorityQueue>()?;
    Ok(())
}
