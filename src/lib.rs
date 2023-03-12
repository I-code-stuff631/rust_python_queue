use pyo3::{prelude::*, types::PyFunction};
#[allow(unused_imports)]
use pyo3::exceptions::PyIndexError;
#[allow(unused_imports)]
use std::cmp::Ordering::*;
use std::sync::{Arc, RwLock};
use std::iter::FusedIterator;


struct Node {
    /// This should NOT be used for comparisions, use value instead
    item: Py<PyAny>,
    /// This should be used for any comparisions
    value: Py<PyAny>,
    left: Option<Arc<RwLock<Node>>>,
    right: Option<Arc<RwLock<Node>>>,
}

/// PriorityQueue implmented with an explicit binary search tree
#[pyclass]
struct PriorityQueue {
    root: Option<Arc<RwLock<Node>>>,
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
    fn push(&mut self, item: &PyAny) -> PyResult<()>{
        let py = item.py();
        let new_node = Node {
            item: item.into_py(py),
            value: self.get_comparison_value_for(item)?,
            left: None,
            right: None,
        };

        let mut next = self.root.clone();
        while let Some(node) = next {
            let read_node = node.read().unwrap();
            match new_node.value.as_ref(py).compare(&read_node.value)? {
                Less => {
                    if let Some(left_node) = read_node.left.clone() {
                        next = Some(left_node);
                    } else {
                        drop(read_node);
                        next = Some(node); // Put back
                        break;
                    }
                },
                Equal | Greater => {
                    if let Some(right_node) = read_node.right.clone() {
                        next = Some(right_node)
                    } else {
                        drop(read_node);
                        next = Some(node); // Put back
                        break; 
                    }
                },
            }
        }

        // Put node
        if let Some(parent_node) = next {
            let mut parent_node = parent_node.write().unwrap();
            match new_node.value.as_ref(py).compare(&parent_node.value)? {
                Less => parent_node.left = Some(Arc::new(RwLock::new(new_node))),
                Equal | Greater => parent_node.right = Some(Arc::new(RwLock::new(new_node))),
            }
        } else {
            self.root = Some(Arc::new(RwLock::new(new_node)))
        }

        self.length += 1;
        Ok(())
    }

    /// Pops the next item off the queue
    fn pop(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        // Getting the parent node will prob be the hardest part of impling this and I dont think that will actually
        // be hard
        todo!()
    }
    
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
    fn peek(&self) -> Option<Py<PyAny>> {
        // This function should only return None when there is no root node

        // Find the rightmost node
        // The alternative to all of these seeming unnecessary clones is making a heap allocation and storing
        // a bunch of read locks in it. (which is obv worse both readability wise and performance wise).
        // The clones safely allow not always having some kind of lock.
        let mut next = self.root.clone(); 
        while let Some(node) = next {
            let right = node.read().unwrap().right.clone();
            match right {
                Some(right_node) => next = Some(right_node),
                None => {
                    next = Some(node); // Put back
                    break;
                }
            }
        }
        
        next.map(|node| Py::clone(&node.read().unwrap().item))
    }

    fn __contains__(&self, py: Python<'_>, item: &PyAny) -> PyResult<bool> {
        let value = self.get_comparison_value_for(item)?;
        let mut next = self.root.clone();
        while let Some(node) = next {
            let node = node.read().unwrap();
            match value.as_ref(py).compare(&node.value)? {
                Less => next = node.left.clone(),
                Greater => next = node.right.clone(),
                Equal => return Ok(true), // We found a match
            };
        };
        Ok(false)
    }

    /// Ez impl by calling __delitem__ when it is impled
    fn remove(&mut self) { todo!() }

    fn __getitem__(&self, index: usize) -> PyResult<Py<PyAny>> {
        self.into_iter().nth(index).ok_or_else(|| PyIndexError::new_err(
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
        // If the tree was height-balanced this capacity could be smaller
        let mut stack = Vec::with_capacity(self.length);

        // Push Arc to the root node and Arcs to all of the nodes directly right of it
        let mut next = self.root.clone(); // Clone Arc
        while let Some(node) = next {
            next = node.read().unwrap().right.clone(); // Will be pushed on in next iteration
            stack.push(node);
        }

        IntoIter { stack }
    }
}

#[pyclass]
struct IntoIter {
    stack: Vec<Arc<RwLock<Node>>>,
}

impl Iterator for IntoIter {
    type Item = Py<PyAny>;

    fn next(&mut self) -> Option<Py<PyAny>> {
        self.stack.pop().map(|node| {
            // Push Arc to the node left of the current and Arcs to all nodes directly right of the left node
            let mut next = node.read().unwrap().left.clone(); // Clone Arc
            while let Some(node) = next {
                next = node.read().unwrap().right.clone(); // Will be pushed on in next iteration
                self.stack.push(node);
            }
            
            // Should always be the greatest
            Py::clone(&node.read().unwrap().item)
        })
    }
}
impl FusedIterator for IntoIter {}

#[pymethods]
impl IntoIter {
    fn __next__(&mut self) -> Option<<Self as Iterator>::Item> {
        self.next()
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
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
}

/// A Python module implemented in Rust.
#[pymodule]
fn python_extension(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PriorityQueue>()?;
    Ok(())
}
