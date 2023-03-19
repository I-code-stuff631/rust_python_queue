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
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
}

/// PriorityQueue implmented with an explicit binary search tree
#[pyclass(sequence, unsendable)]
struct PriorityQueue {
    root: Option<Rc<RefCell<Node>>>,
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

        match &self.root {
            Some(root_node) => {
                let mut next: *const Rc<RefCell<Node>> = root_node as *const _;
                loop {
                    let mut node = unsafe {
                        // SAFETY: We do not decrease the strong count of any Rcs in the tree...
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

    // /// Pops the next item off the queue, will return None if the queue is empty
    // fn pop(&mut self) -> Option<Py<PyAny>> {
    //     // Getting the parent node will prob be the hardest part of impling this and I dont think that will actually
    //     // be hard
    //     todo!()
    // }
    
    fn clear(&mut self) { 
        self.root = None;
        self.length = 0;
     }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    // /// Access the next item without removing it from the queue, this method will return None if the queue 
    // /// is empty.
    // /// It is a logical error to modify the item returned by this method in such a way that its
    // /// comparison value would change.
    // fn peek(&self) -> Option<Py<PyAny>> {
    //     self.root.as_ref().map(|root_node| {
    //         unsafe {
    //             let mut next = root_node.try_borrow_unguarded().expect("Nothing should have a RefMut to this node");
    //             let root_node = (); // Could be used to deallocate parts of the tree that are being looped over

    //             while let Some(right_node) = &next.right {
    //                 next = right_node.try_borrow_unguarded().expect("Nothing should have a RefMut to this node");
    //                 // root_node.borrow_mut().right = None; // SAFETY: The ref just died (except this causes UB)
    //             }
    //             Py::clone(&next.item)
    //         }
    //     })
    // }

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

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        let mut string = String::with_capacity(2);
        string.push('[');

        let mut length_remaining = self.length;
        let mut iter = self.into_iter();
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
    type Item = Py<PyAny>;
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

#[pyclass(unsendable)]
struct IntoIter {
    stack: Vec<Rc<RefCell<Node>>>,
}

impl Iterator for IntoIter {
    type Item = Py<PyAny>;

    fn next(&mut self) -> Option<Py<PyAny>> {
        self.stack.pop().map(|node| {
            let node = (*node).borrow();

            let mut next = node.left.clone();
            while let Some(node) = next {
                next = (*node).borrow().right.clone(); // Will be pushed on in next iteration
                self.stack.push(node);
            }
            
            // Should always be the greatest
            Py::clone(&node.item)
        })
    }
}
impl std::iter::FusedIterator for IntoIter {}

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
