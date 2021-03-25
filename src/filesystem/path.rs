/// This module handles files path.

use crate::util::FailableClone;
use crate::util::container::string::String;
use crate::util::container::vec::Vec;

/// A structure representing a path to a file.
pub struct Path {
	/// Tells whether the path is absolute or relative.
	absolute: bool,
	/// An array containing the different parts of the path which are separated with `/`.
	parts: Vec::<String>,
}

impl Path {
	/// Creates a new instance from string.
	pub fn from_string(path: &str) -> Self {
		Self {
			absolute: path.chars().next().unwrap() == '/',
			parts: Vec::new(), // TODO
		}
	}

	/// Tells whether the path is absolute or not.
	pub fn is_absolute(&self) -> bool {
		self.absolute
	}

	// TODO Use `push` on string for separator
	/// Converts the path into a String and returns it.
	pub fn as_string(&self) -> Result::<String, ()> {
		let separator = String::from("/")?;
		let mut s = String::new();
		if self.absolute {
			s.push_str(&separator)?;
		}
		for i in 0..self.parts.len() {
			s.push_str(&self.parts[i])?;
			if i + 1 < self.parts.len() {
				s.push_str(&separator)?;
			}
		}
		Ok(s)
	}

	/// Reduces the path, removing all useless `.` and `..`.
	pub fn reduce(&mut self) {
		let mut i = 0;
		while i < self.parts.len() {
			let part = &self.parts[i];
			if part == "." {
				self.parts.remove(i);
			} else if part == ".." && self.absolute && i == 0 {
				self.parts.remove(i);
			} else if part == ".." && i > 0 && self.parts[i - 1] != ".." {
				self.parts.remove(i);
				self.parts.remove(i - 1);
				i -= 1;
			} else {
				i += 1;
			}
		}
	}

	/// Concats the current path with another path `other` to create a new path. The path is not
	/// automaticaly reduced.
	pub fn concat(&self, other: &Self) -> Result::<Self, ()> {
		let mut self_parts = self.parts.failable_clone()?;
		let mut other_parts = other.parts.failable_clone()?;
		self_parts.append(&mut other_parts)?;
		Ok(Self {
			absolute: self.absolute,
			parts: self_parts,
		})
	}
}

impl FailableClone for Path {
	fn failable_clone(&self) -> Result::<Self, ()> {
		Ok(Self {
			absolute: self.absolute,
			parts: self.parts.failable_clone()?,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test_case]
	fn path_absolute0() {
		assert!(Path::from_string("/").is_absolute());
	}

	#[test_case]
	fn path_absolute1() {
		assert!(Path::from_string("/.").is_absolute());
	}

	#[test_case]
	fn path_absolute2() {
		assert!(!Path::from_string(".").is_absolute());
	}

	#[test_case]
	fn path_absolute3() {
		assert!(!Path::from_string("..").is_absolute());
	}

	#[test_case]
	fn path_absolute4() {
		assert!(!Path::from_string("./").is_absolute());
	}

	#[test_case]
	fn path_reduce0() {
		let mut path = Path::from_string("/.");
		path.reduce();
		assert_eq!(path.as_string().unwrap(), "/");
	}

	#[test_case]
	fn path_reduce1() {
		let mut path = Path::from_string("/..");
		path.reduce();
		assert_eq!(path.as_string().unwrap(), "/");
	}

	#[test_case]
	fn path_reduce2() {
		let mut path = Path::from_string("../");
		path.reduce();
		assert_eq!(path.as_string().unwrap(), "..");
	}

	#[test_case]
	fn path_reduce3() {
		let mut path = Path::from_string("../bleh");
		path.reduce();
		assert_eq!(path.as_string().unwrap(), "../bleh");
	}

	#[test_case]
	fn path_reduce4() {
		let mut path = Path::from_string("../bleh/..");
		path.reduce();
		assert_eq!(path.as_string().unwrap(), "..");
	}

	#[test_case]
	fn path_reduce5() {
		let mut path = Path::from_string("../bleh/../bluh");
		path.reduce();
		assert_eq!(path.as_string().unwrap(), "../bluh");
	}

	#[test_case]
	fn path_reduce6() {
		let mut path = Path::from_string("/bleh/../bluh");
		path.reduce();
		assert_eq!(path.as_string().unwrap(), "/bluh");
	}

	#[test_case]
	fn path_reduce6() {
		let mut path = Path::from_string("/bleh/../../bluh");
		path.reduce();
		assert_eq!(path.as_string().unwrap(), "/bluh");
	}

	// TODO test concat
}
