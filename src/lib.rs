use pyo3::{prelude::*, types::PyFunction, exceptions::PyIndexError};
#[allow(unused_imports)]
use std::cmp::Ordering::*;
use std::sync::{Arc, RwLock};


struct Node {
    /// This should NOT be used for comparisions, use value instead
    item: Py<PyAny>,
    /// This should be used for any comparisions
    value: Py<PyAny>,
    left: Option<WNode>,
    right: Option<WNode>,
}

/// Wrapped Node
type WNode = Arc<RwLock<Node>>;

/// PriorityQueue implmented with an explicit binary search tree
#[pyclass]
struct PriorityQueue {
    root: Option<WNode>,
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
            length: 0,
        }
    }

    /// Pushes the specified item onto the queue.
    /// It is a logical error to modify the item in such a way that its comparison value
    /// would change after it has been pushed.
    fn push(&mut self, item: &PyAny) -> PyResult<()>{
        let py = item.py();
        let new_node = Node {
            item: item.into_py(py),
            value: self.get_comparison_value_for(item)?,
            left: None,
            right: None,
        };

        // Find position to put new node
        let mut next = &mut self.root; // Needs to be &mut
        while let Some(node) = next {
            let mut node = node.get_mut().unwrap();
            next = match new_node.value.as_ref(py).compare(&node.value)? {
                Less => &mut node.left,
                Equal | Greater => &mut node.right,
            };
        }
        debug_assert!(next.is_none());

        *next = Some(Arc::new(RwLock::new(new_node))); // Put node
        self.length += 1;
        Ok(())
    }

    /// Pops the next item off the queue
    fn pop(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> { todo!() }
    
    fn clear(&mut self) { 
        self.root = None; // Fine for now, may be tail recursion
        self.length = 0;
     }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Access the next item without removing it from the queue.
    /// It is a logical error to modify the item returned by this method in such a way that its
    /// comparison value would change.
    fn peek(&self) -> Option<&Py<PyAny>> {
        self.root.map(|ref root_node| {
            let mut next = root_node.read().unwrap();
            while let Some(right_node) = &next.right {
                next = right_node.read().unwrap();
            };
            next
        }).map(|node| &node.item)
    }

    fn __contains__(&self, py: Python<'_>, item: &PyAny) -> PyResult<bool> {
        let value = self.get_comparison_value_for(item)?;
        let mut next = &self.root;
        while let Some(node) = next {
            let node = node.read().unwrap();
            match value.as_ref(py).compare(&node.value)? {
                Less => next = &node.left,
                Greater => next = &node.right,
                Equal => return Ok(true), // We found a match
            };
        };
        Ok(false)
    }

    /// Ez impl by calling __delitem__ when it is impled
    fn remove(&mut self) { todo!() }

    fn __getitem__(&self, index: usize) -> PyResult<Py<PyAny>> {
        self.into_iter().nth(index).ok_or(PyIndexError::new_err(
            format!("Index: {index} out of range!")
        ))
    }

    fn __len__(&self) -> usize {
        self.length
    }

    fn __iter__(&self) -> IntoIter {
        self.into_iter()
    }
}

impl IntoIterator for &PriorityQueue {
    type Item = Py<PyAny>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut stack = Vec::with_capacity(self.length);

        // Iter will do the rest
        if let Some(root_node) = self.root {
            stack.push((false, Arc::clone(&root_node), false));
        }
        // if explored false then add the nodes in the left path, the current node, and then the nodes in the right
        // path to the stack, explored is only one bool and prob should go after the node in the tupil. Muetex should
        // be able to restore mutability.
        
        IntoIter { stack }
    }
}

#[pyclass]
struct IntoIter {
    stack: Vec<(bool, WNode, bool)>,
}

impl Iterator for IntoIter {
    type Item = Py<PyAny>;

    fn next(&mut self) -> Option<Py<PyAny>> {
        self.stack.pop().map(|(explored_left, node, explored_right)| {
            if !explored_right {
                self.stack.push((explored_left, node, true));

                let mut next = node.right.as_ref();
                while let Some(right_node) = next {
                    self.stack.push((false, Arc::clone(&right_node), true));
                    next = right_node.right.as_ref();
                }

                unsafe {
                    // SAFETY: We just pushed a node on above so we can pop off at least one
                    return Py::clone(&self.stack.pop().unwrap_unchecked().1.item);
                }
            }
            if !explored_left {
                let slice_start = self.stack.len();
                let mut slice_end = slice_start;

                let mut next = node.left.as_ref();
                while let Some(left_node) = next {
                    self.stack.push((true, Arc::clone(&left_node), false));
                    next = left_node.left.as_ref();
                    slice_end += 1;
                }
                self.stack[slice_start..slice_end].reverse(); // So they are popped off the stack in the right order

                return Py::clone(&node.item);
            }
            panic!()
        })
    }
}

#[pymethods]
impl IntoIter {
    fn __next__(&mut self) -> Option<<Self as Iterator>::Item> {
        self.next()
    }

    fn __iter__(&self) -> &Self {
        self
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

    /// Returns a ref to the node with the highest value in the tree
    fn greatest_node(&self) -> Option<&Node> {

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
