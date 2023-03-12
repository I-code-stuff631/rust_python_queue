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
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

/// PriorityQueue implmented with an explicit binary search tree
#[pyclass]
struct PriorityQueue {
    root: Option<Arc<RwLock<Node>>>, // Tree has a read-mode and a write-mode
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

        match &self.root {
            Some(root) => {
                let mut root = root.write().unwrap();

                let mut next = match new_node.value.as_ref(py).compare(&root.value)? {
                    Less => &mut root.left,
                    Equal | Greater => &mut root.right,
                };
                while let Some(node) = next {
                    match new_node.value.as_ref(py).compare(&node.value)? {
                        Less => next = &mut node.left,
                        Equal | Greater => next = &mut node.right,
                    }
                }
                debug_assert!(next.is_none());

                *next = Some(Box::new(new_node)); // Put node
            },
            None => self.root = Some(Arc::new(RwLock::new(new_node))),
        }

        self.length += 1;
        Ok(())
    }

    // /// Pops the next item off the queue
    // fn pop(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> { todo!() }
    
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
        self.root.as_deref().map(|root| {
            let root = root.read().unwrap();

            let mut next = &*root;
            while let Some(right_node) = next.right.as_deref() {
                next = right_node;
            }

            Py::clone(&next.item)
        })
    }

    // fn __contains__(&self, py: Python<'_>, item: &PyAny) -> PyResult<bool> {
    //     let value = self.get_comparison_value_for(item)?;
    //     let mut next = self.root.as_ref().map(|root| root.read().unwrap());
    //     let root = 
    //     while let Some(node) = next {
    //         let node = node.read().unwrap();
    //         match value.as_ref(py).compare(&node.value)? {
    //             Less => next = &node.left,
    //             Greater => next = &node.right,
    //             Equal => return Ok(true), // We found a match
    //         };
    //     };
    //     Ok(false)
    // }

    /// Ez impl by calling __delitem__ when it is impled
    fn remove(&mut self) { todo!() }

    // fn __getitem__(&self, index: usize) -> PyResult<Py<PyAny>> {
    //     self.into_iter().nth(index).ok_or(PyIndexError::new_err(
    //         format!("Index: {index} out of range!")
    //     ))
    // }

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
        match &self.root {
            Some(root) => {
                let mut stack: Vec<*const Node> = Vec::with_capacity(self.length);

                let mut next = &*root.read().unwrap();
                stack.push(next);
                while let Some(right_node) = next.right.as_deref() {
                    stack.push(right_node);
                    next = right_node;
                }

                IntoIter {
                    lock: Some(Arc::clone(&root)),
                    stack,
                }
            },
            None => IntoIter { lock: None, stack: Vec::new() }
        }
    }
}

#[pyclass]
struct IntoIter {
    lock: Option<Arc<RwLock<Node>>>,
    stack: Vec<*const Node>,
}
unsafe impl Send for IntoIter {}

impl Iterator for IntoIter {
    type Item = Py<PyAny>;

    fn next(&mut self) -> Option<Py<PyAny>> {
        if let Some(node) = self.stack.pop() {
            let _lock = unsafe {
                // SAFETY: We popped a node which, due to the into iter code, means lock is not None
                self.lock.as_ref().unwrap_unchecked().read().unwrap()
            };
            // From here until the lock is dropped the tree should be safely readable
            
            // Find the next greatest node
            unsafe {
                let mut next = (*node).left.as_deref();

                // Explore right
                while let Some(right_node) = next {
                    self.stack.push(right_node);
                    next = right_node.right.as_deref();
                }
            }

            unsafe {
                Some(Py::clone(&(*node).item)) // Should always be the greatest
            } 
        } else {
            // The iteration is over
            self.lock = None; // Release the lock, allowing the tree to be cleaned up if it is the last Arc
            None
        }
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

#[cfg(test)]
mod test {
    use super::PriorityQueue;
    use pyo3::conversion::ToPyObject;
    use pyo3::Python;

    #[test]
    fn iter() {
        let mut queue = PriorityQueue::new(None);
        
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            queue.push(5.to_object(py).as_ref(py)).unwrap();
            queue.push(0.to_object(py).as_ref(py)).unwrap();
            queue.push(10.to_object(py).as_ref(py)).unwrap();
            queue.push(10.to_object(py).as_ref(py)).unwrap();
            queue.push(10.5.to_object(py).as_ref(py)).unwrap();
            queue.push(5.to_object(py).as_ref(py)).unwrap();
            queue.push(3.to_object(py).as_ref(py)).unwrap();
            queue.push(6.to_object(py).as_ref(py)).unwrap();
            queue.push(2.to_object(py).as_ref(py)).unwrap();

            let mut queue_iter = queue.__iter__();
            assert_eq!(10.5, queue_iter.next().unwrap().extract(py).unwrap());
            assert_eq!(10, queue_iter.next().unwrap().extract(py).unwrap());
            assert_eq!(10, queue_iter.next().unwrap().extract(py).unwrap());
            assert_eq!(6, queue_iter.next().unwrap().extract(py).unwrap());
            assert_eq!(5, queue_iter.next().unwrap().extract(py).unwrap());
            assert_eq!(5, queue_iter.next().unwrap().extract(py).unwrap());
            assert_eq!(3, queue_iter.next().unwrap().extract(py).unwrap());
            assert_eq!(2, queue_iter.next().unwrap().extract(py).unwrap());
            assert_eq!(0, queue_iter.next().unwrap().extract(py).unwrap());
        });
    }
}
