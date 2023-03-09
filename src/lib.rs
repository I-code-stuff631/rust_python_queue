use pyo3::{prelude::*, types::PyFunction, exceptions::PyIndexError};
#[allow(unused_imports)]
use std::cmp::Ordering::*;


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
            value: self.get_comparison_value_from(item)?,
            left: None,
            right: None,
        });
        let mut next = &mut self.root;
        while let Some(node) = next {
            match new_node.value.as_ref(py).compare(&node.value)? {
                Less => next = &mut node.left,
                Equal | Greater => next = &mut node.right,
            };
        }
        debug_assert!(next.is_none());
        *next = Some(new_node);
        self.length += 1;
        Ok(())
    }

    // /// Pops the next item off the queue
    // fn pop(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
    //     todo!()
    // }
    
    fn clear(&mut self) { 
        self.root = None; // Fine for now, may be tail recursion
        self.length = 0;
     }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    fn __contains__(&self, py: Python<'_>, item: &PyAny) -> PyResult<bool> {
        let value = self.get_comparison_value_from(item)?;
        let mut next = &self.root;
        while let Some(node) = next {
            match value.as_ref(py).compare(&node.value)? {
                Less => next = &node.left,
                Greater => next = &node.right,
                Equal => return Ok(true), // We found a match
            };
        };
        Ok(false)
    }

    fn __getitem__(&self, index: usize) -> PyResult<&Py<PyAny>> {
        self.into_iter().nth(index).ok_or(PyIndexError::new_err("Index out of range!"))
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

impl<'a> IntoIterator for &'a PriorityQueue {
    type Item = &'a Py<PyAny>;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let mut stack = Vec::with_capacity(self.length);
        
        let mut next = self.root.as_deref();
        while let Some(node) = next {
            stack.push(node);
            next = node.right.as_deref();
        }

        Iter {
            stack,
            explored_right: true,
        }
    }
}

struct Iter<'a> {
    stack: Vec<&'a Node>,
    explored_right: bool,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Py<PyAny>;

    fn next(&mut self) -> Option<&'a Py<PyAny>> {
        self.stack.pop().map(|node| {
            if self.explored_right {
                let slice_start = self.stack.len();
                let mut slice_end = slice_start;
                
                let mut next = node.left.as_deref();
                while let Some(left_node) = next {
                    self.stack.push(left_node);
                    next = left_node.left.as_deref();
                    slice_end += 1;
                }
                self.stack[slice_start..slice_end].reverse(); // So they are popped off the stack in the right order

                self.explored_right = false;
                &node.item
            } else {
                let mut next = node.right.as_deref();
                self.stack.push(node);
                while let Some(right_node) = next {
                    self.stack.push(right_node);
                    next = right_node.right.as_deref();
                }
                self.explored_right = true;
                // SAFETY: We just pushed a node on above so we can pop off at least one
                unsafe { &self.stack.pop().unwrap_unchecked().item }
            }
        })
    }
}

/// Rust only
impl PriorityQueue {
    fn get_comparison_value_from(&self, item: &PyAny) -> PyResult<Py<PyAny>> {
        let py = item.py();
        Ok(match &self.get_cmpison_value {
            Some(get_comparison_value) => get_comparison_value.call1(py, (item,))?,
            None => item.into_py(py),
        })
    }

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
}

impl Node {
    #[allow(dead_code)]
    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn python_extension(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PriorityQueue>()?;
    Ok(())
}
