use pyo3::intern;
use pyo3::{prelude::*, types::PyFunction};
#[allow(unused_imports)]
use pyo3::exceptions::PyIndexError;
#[allow(unused_imports)]
use std::cmp::Ordering::*;
use std::sync::{Arc, RwLock, Weak};


struct Node {
    /// This should NOT be used for comparisions, use value instead
    item: Py<PyAny>,
    /// This should be used for any comparisions
    value: Py<PyAny>,
    left: Option<Arc<RwLock<Node>>>,
    right: Option<Arc<RwLock<Node>>>,
}

/// PriorityQueue implmented with an explicit binary search tree
#[pyclass(sequence)]
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
    /// This method guarantees that the item will be pushed onto the tree will actually be pushed on
    /// (and not deallocated as soon as it returns).
    fn push(&mut self, item: &PyAny) -> PyResult<()>{
        let py = item.py();
        let new_node = Node {
            item: item.into_py(py),
            value: self.get_comparison_value_for(item)?,
            left: None,
            right: None,
        };

        match &self.root {
            Some(root_node) => {
                let mut next = Some(Arc::as_ptr(root_node));
                while let Some(node) = next {
                    // SAFETY: We never decrease any of the Arcs strong counts to zero and since we take '&mut self'
                    // nothing else can decrease them to zero either (or do anything with the tree at all for that matter).
                    unsafe {
                        let node_readlock = (*node).read().unwrap();

                        match new_node.value.as_ref(py).compare(&node_readlock.value)? {
                            Less => match &node_readlock.left {
                                Some(left_node) => next = Some(Arc::as_ptr(left_node)),
                                None => {
                                    drop(node_readlock); // Otherwise it would deadlock
                                    (*node).write().unwrap().left = Some(Arc::new(RwLock::new(new_node)));
                                    break;
                                },
                            }
                            Equal | Greater => match &node_readlock.right {
                                Some(right_node) => next = Some(Arc::as_ptr(right_node)),
                                None => {
                                    drop(node_readlock); // Otherwise it would deadlock
                                    (*node).write().unwrap().right = Some(Arc::new(RwLock::new(new_node)));
                                    break;
                                },
                            }
                        }
                    };
                }
            },
            None => self.root = Some(Arc::new(RwLock::new(new_node))),
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
        self.root = None;
        self.length = 0;
     }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Access the next item without removing it from the queue, this method will return None
    /// only if the queue is completely empty.
    /// It is a logical error to modify the item returned by this method in such a way that its
    /// comparison value would change.
    fn peek(&self) -> Option<Py<PyAny>> {
        fn rightmost_item_from(node: &Node) -> Py<PyAny> {
            match &node.right {
                Some(right_node) => {
                    let readlock = right_node.read().unwrap();
                    rightmost_item_from(&*readlock)
                },
                None => Py::clone(&node.item),
            }
        }

        self.root.as_ref().map(|root_node| {
            let node_readlock = root_node.read().unwrap();
            // I am making the same assumption here as I am for __contains__
            rightmost_item_from(&*node_readlock)
        })
    }

    /// This function should return true or false only when it is absolutely certain that the item does or does
    /// not exist within the tree respectively (an item existing in the tree is defined as it being reachable
    /// from the root).
    fn __contains__(&self, py: Python<'_>, item: &PyAny) -> PyResult<bool> {
        let value = self.get_comparison_value_for(item)?;

        fn rsearch(value: &PyAny, node: &Node) -> PyResult<bool> {
            match value.compare(&node.value)? {
                Less => match &node.left {
                    Some(left_node) => {
                        let readlock = left_node.read().unwrap();
                        rsearch(value, &*readlock)
                    },
                    None => Ok(false),
                },
                Equal => Ok(true),
                Greater => match &node.right {
                    Some(right_node) => {
                        let readlock = right_node.read().unwrap();
                        rsearch(value, &*readlock)
                    },
                    None => Ok(false),
                },
            }
        }
        match &self.root {
            Some(root_node) => {
                let node_readlock = root_node.read().unwrap();
                // Im using the call stack here to avoid allocating a bunch of readlocks on the heap,
                // instead they are being allocated on the stack. I am assuming that if the tree
                // is height balanced the call stack will seldom grow large enough to crash the program.
                // If this assumption proves to be false in practise I will use a heap allocation instead.
                rsearch(value.as_ref(py), &*node_readlock)
            },
            None => Ok(false),
        }
    }

    /// Ez impl by calling __delitem__ when it is impled
    fn remove(&mut self) { todo!() }

    fn __len__(&self) -> usize {
        self.length
    }

    fn __iter__(&self) -> IntoIter {
        self.into_iter()
    }

    fn __getitem__(&self, index: usize) -> PyResult<Py<PyAny>> {
        self.into_iter().nth(index).ok_or_else(|| PyIndexError::new_err(
            format!("Index: {index} out of range!")
        ))
    }

    // fn __str__(&self, py: Python<'_>) -> PyResult<String> {
    //     let mut string = String::with_capacity(2);
    //     string.push('[');
    //     let mut iter = self.into_iter();
    //     while let Some(item) = iter.next() {
    //         string.push_str(item.as_ref(py).str()?.to_str()?);
    //         if iter.len() != 0 {
    //             string.push_str(", ");
    //         }
    //     }
    //     string.push(']');
    //     Ok(string)
    // }
}

impl IntoIterator for &PriorityQueue {
    type Item = Py<PyAny>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        // If the tree was height-balanced this capacity could be smaller
        let mut stack = Vec::with_capacity(self.length);

        let mut next = self.root.clone();
        while let Some(node) = next {
            next = node.read().unwrap().right.clone(); // Will be pushed on in next iteration
            stack.push(node);
        }

        IntoIter { stack/*, length: self.length*/ }
    }
}

#[pyclass]
struct IntoIter {
    stack: Vec<Arc/*Week*/<RwLock<Node>>>,
    // length: usize,
}

impl Iterator for IntoIter {
    type Item = Py<PyAny>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

// impl Iterator for IntoIter {
//     type Item = Py<PyAny>;

//     fn next(&mut self) -> Option<Py<PyAny>> {
//         self.stack.pop().map(|node| {
//             let mut next = node.read().unwrap().left.clone();
//             while let Some(node) = next {
//                 next = node.read().unwrap().right.clone(); // Will be pushed on in next iteration
//                 self.stack.push(node);
//             }
            
//             self.length -= 1;
//             // Should always be the greatest
//             Py::clone(&node.read().unwrap().item)
//         })
//     }

//     // fn size_hint(&self) -> (usize, Option<usize>) {
//     //     (self.length, Some(self.length))
//     // }
// }
// impl std::iter::FusedIterator for IntoIter {}

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
