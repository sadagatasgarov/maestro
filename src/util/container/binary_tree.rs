/// This module implements a binary tree container.

use core::cmp::Ordering;
use core::cmp::max;
use core::ptr::NonNull;

/// The color of a binary tree node.
enum NodeColor {
	Red,
	Black,
}

/// TODO doc
struct BinaryTreeNode<T> {
	/// Pointer to the parent node
	parent: Option::<NonNull<Self>>,
	/// Pointer to the left child
	left: Option::<NonNull<Self>>,
	/// Pointer to the right child
	right: Option::<NonNull<Self>>,
	/// The color of the node
	color: NodeColor,

	value: T,
}

impl<T: 'static> BinaryTreeNode<T> {
	/// Creates a new node with the given `value`. The node is colored Red by default.
	pub fn new(value: T) -> Self {
		Self {
			parent: None,
			left: None,
			right: None,
			color: NodeColor::Red,

			value: value,
		}
	}

	/// Unwraps the given pointer option into a reference option.
	fn unwrap_pointer(ptr: &Option::<NonNull::<Self>>) -> Option::<&'static Self> {
		if let Some(p) = ptr {
			unsafe { // Dereference of raw pointer
				Some(&*p.as_ptr())
			}
		} else {
			None
		}
	}

	/// Same as `unwrap_pointer` but returns a mutable reference.
	fn unwrap_pointer_mut(ptr: &mut Option::<NonNull::<Self>>) -> Option::<&'static mut Self> {
		if let Some(p) = ptr {
			unsafe { // Call to unsafe function
				Some(&mut *(p.as_ptr() as *mut _))
			}
		} else {
			None
		}
	}

	/// Returns a reference to the left child node.
	pub fn get_parent(&self) -> Option::<&'static Self> {
		Self::unwrap_pointer(&self.parent)
	}

	/// Returns a reference to the parent child node.
	pub fn get_parent_mut(&mut self) -> Option::<&'static mut Self> {
		Self::unwrap_pointer_mut(&mut self.parent)
	}

	/// Returns a mutable reference to the parent child node.
	pub fn get_left(&self) -> Option::<&'static Self> {
		Self::unwrap_pointer(&self.left)
	}

	/// Returns a reference to the left child node.
	pub fn get_left_mut(&mut self) -> Option::<&'static mut Self> {
		Self::unwrap_pointer_mut(&mut self.left)
	}

	/// Returns a reference to the left child node.
	pub fn get_right(&self) -> Option::<&'static Self> {
		Self::unwrap_pointer(&self.right)
	}

	/// Returns a reference to the left child node.
	pub fn get_right_mut(&mut self) -> Option::<&'static mut Self> {
		Self::unwrap_pointer_mut(&mut self.right)
	}

	/// Returns a reference to the grandparent node.
	pub fn get_grandparent(&self) -> Option::<&'static Self> {
		if let Some(p) = self.get_parent() {
			p.get_parent()
		} else {
			None
		}
	}

	/// Returns a mutable reference to the grandparent node.
	pub fn get_grandparent_mut(&mut self) -> Option::<&'static mut Self> {
		if let Some(p) = self.get_parent_mut() {
			p.get_parent_mut()
		} else {
			None
		}
	}

	/// Returns a reference to the sibling node.
	pub fn get_sibling(&self) -> Option::<&'static Self> {
		let self_ptr = self as *const _;
		let p = self.get_parent();
		if p.is_none() {
			return None;
		}

		let parent = p.unwrap();
		let left = parent.get_left();
		if left.is_some() && left.unwrap() as *const _ == self_ptr {
			parent.get_right()
		} else {
			parent.get_left()
		}
	}

	/// Returns a mutable reference to the sibling node.
	pub fn get_sibling_mut(&mut self) -> Option::<&'static mut Self> {
		let self_ptr = self as *const _;
		let p = self.get_parent_mut();
		if p.is_none() {
			return None;
		}

		let parent = p.unwrap();
		let left = parent.get_left_mut();
		if left.is_some() && left.unwrap() as *const _ == self_ptr {
			parent.get_right_mut()
		} else {
			parent.get_left_mut()
		}
	}

	/// Returns a reference to the uncle node.
	pub fn get_uncle(&mut self) -> Option::<&'static Self> {
		if let Some(parent) = self.get_parent() {
			parent.get_sibling()
		} else {
			None
		}
	}

	/// Returns a mutable reference to the uncle node.
	pub fn get_uncle_mut(&mut self) -> Option::<&'static mut Self> {
		if let Some(parent) = self.get_parent_mut() {
			parent.get_sibling_mut()
		} else {
			None
		}
	}

	/// Applies a left tree rotation with the current node as pivot.
	pub fn left_rotate(&mut self) {
		let root = self.parent;
		let root_ptr = unsafe { // Dereference of raw pointer
			&mut *(root.unwrap().as_ptr() as *mut Self)
		};
		let left = self.left;

		self.left = root;
		root_ptr.parent = NonNull::new(self);

		root_ptr.right = left;
		if left.is_some() {
			unsafe { // Dereference of raw pointer
				&mut *(left.unwrap().as_ptr() as *mut Self)
			}.parent = root;
		}
	}

	/// Applies a right tree rotation with the current node as pivot.
	pub fn right_rotate(&mut self) {
		let root = self.parent;
		let root_ptr = unsafe { // Dereference of raw pointer
			&mut *(root.unwrap().as_ptr() as *mut Self)
		};
		let right = self.right;

		self.right = root;
		root_ptr.parent = NonNull::new(self);

		root_ptr.left = right;
		if right.is_some() {
			unsafe { // Dereference of raw pointer
				&mut *(right.unwrap().as_ptr() as *mut Self)
			}.parent = root;
		}
	}

	/// Inserts the given node `node` to left of the current node.
	pub fn insert_left(&mut self, node: &mut BinaryTreeNode::<T>) {
		if let Some(mut n) = self.left {
			node.insert_left(unsafe { // Call to unsafe function
				n.as_mut()
			});
		}
		self.left = NonNull::new(node);
		node.parent = NonNull::new(self);
	}

	/// Inserts the given node `node` to right of the current node.
	pub fn insert_right(&mut self, node: &mut BinaryTreeNode::<T>) {
		if let Some(mut n) = self.right {
			node.insert_right(unsafe { // Call to unsafe function
				n.as_mut()
			});
		}
		self.right = NonNull::new(node);
		node.parent = NonNull::new(self);
	}

	/// Returns the number of nodes in the subtree.
	pub fn nodes_count(&self) -> usize {
		let left_count = if let Some(l) = self.get_left() {
			l.nodes_count()
		} else {
			0
		};
		let right_count = if let Some(r) = self.get_right() {
			r.nodes_count()
		} else {
			0
		};
		1 + left_count + right_count
	}

	/// Returns the depth of the subtree.
	pub fn get_depth(&self) -> usize {
		let left_count = if let Some(l) = self.get_left() {
			l.nodes_count()
		} else {
			0
		};
		let right_count = if let Some(r) = self.get_right() {
			r.nodes_count()
		} else {
			0
		};
		1 + max(left_count, right_count)
	}
}

/// TODO doc
pub struct BinaryTree<T: 'static> {
	/// The root node of the binary tree.
	root: Option::<NonNull<BinaryTreeNode::<T>>>,
}

impl<T: 'static> BinaryTree<T> {
	/// Creates a new binary tree.
	pub fn new() -> Self {
		Self {
			root: None,
		}
	}

	/// Returns a reference to the root node.
	fn get_root(&self) -> Option::<&BinaryTreeNode::<T>> {
		if let Some(r) = self.root.as_ref() {
			unsafe { // Call to unsafe function
				Some(r.as_ref())
			}
		} else {
			None
		}
	}

	/// Returns a mutable reference to the root node.
	fn get_root_mut(&mut self) -> Option::<&mut BinaryTreeNode::<T>> {
		if let Some(r) = self.root.as_mut() {
			unsafe { // Call to unsafe function
				Some(r.as_mut())
			}
		} else {
			None
		}
	}

	/// Returns the number of nodes in the tree.
	pub fn nodes_count(&self) -> usize {
		if let Some(r) = self.get_root() {
			r.nodes_count()
		} else {
			0
		}
	}

	/// Returns the depth of the tree.
	pub fn get_depth(&self) -> usize {
		if let Some(r) = self.get_root() {
			r.get_depth()
		} else {
			0
		}
	}

	/// Updates the root of the tree.
	/// `node` is a node of the tree.
	fn update_node(&mut self, node: &mut BinaryTreeNode::<T>) {
		let mut root = NonNull::new(node as *mut BinaryTreeNode::<T>);
		while root.is_some() {
			root = unsafe { // Call to unsafe function
				root.unwrap().as_mut()
			}.parent;
		}
		self.root = root;
	}

	/// Returns the node with the closest value. Returns None if the tree is empty.
	/// `cmp` is the comparison function.
	fn get_closest_node<F: Fn(&T) -> Ordering>(&mut self, cmp: F)
		-> Option::<&mut BinaryTreeNode::<T>> {
		let mut node = self.get_root_mut();

		while node.is_some() {
			let n = node.unwrap();
			let ord = cmp(&n.value);
			if ord == Ordering::Less {
				node = n.get_left_mut();
			} else if ord == Ordering::Greater {
				node = n.get_right_mut();
			} else {
				return Some(n);
			}
		}

		node
	}

	/// Searches for a value with the given closure for comparison.
	/// `cmp` is the comparison function.
	pub fn get<F: Fn(&T) -> Ordering>(&mut self, cmp: F) -> Option::<&mut T> {
		let mut node = self.get_root_mut();

		while node.is_some() {
			let n = node.unwrap();
			let ord = cmp(&n.value);
			if ord == Ordering::Less {
				node = n.get_left_mut();
			} else if ord == Ordering::Greater {
				node = n.get_right_mut();
			} else {
				return Some(&mut n.value);
			}
		}

		None
	}

	/// Inserts a node in the tree.
	/// `value` is the node to insert.
	/// `cmp` is the comparison function.
	pub fn insert<F: Fn(&T, &T) -> Ordering>(&mut self, value: T, cmp: F) {
		let mut node = BinaryTreeNode::new(value);
		let closest = self.get_closest_node(| val | {
			cmp(val, &node.value) // TODO Check order
		});

		if let Some(c) = closest {
			let order = cmp(&c.value, &node.value); // TODO Check order
			if order == Ordering::Less {
				c.insert_left(&mut node);
			} else {
				c.insert_right(&mut node);
			}

			// TODO Equilibrate
			self.update_node(&mut node);
		} else {
			debug_assert!(self.root.is_none());
			self.root = NonNull::new(&mut node);
		}
	}

	/// Removes a node from the tree.
	/// `value` is the node to remove.
	/// `cmp` is the comparison function.
	pub fn remove<F: Fn(&T) -> Ordering>(&mut self, _value: T, _cmp: F) {
		// TODO
	}
}

#[cfg(test)]
mod test {
	//use super::*;

	#[test_case]
	fn binary_tree_node_rotate0() {
		// TODO
	}

	// TODO
}
