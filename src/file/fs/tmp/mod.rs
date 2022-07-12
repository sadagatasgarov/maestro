//! Tmpfs (Temporary file system) is, as its name states a temporary filesystem. The files are
//! stored on the kernel's memory and thus are removed when the filesystem is unmounted.

use core::mem::size_of;
use crate::errno;
use crate::file::Errno;
use crate::file::File;
use crate::file::FileContent;
use crate::file::Gid;
use crate::file::INode;
use crate::file::Mode;
use crate::file::Uid;
use crate::file::fs::Statfs;
use crate::file::path::Path;
use crate::util::IO;
use crate::util::container::hashmap::HashMap;
use crate::util::container::string::String;
use crate::util::ptr::SharedPtr;
use super::Filesystem;
use super::FilesystemType;
use super::kernfs::KernFS;
use super::kernfs::node::KernFSNode;

/// The default maximum amount of memory the filesystem can use in bytes.
const DEFAULT_MAX_SIZE: usize = 512 * 1024 * 1024;

/// Returns the size in bytes used by the given node `node`.
fn get_used_size(node: &KernFSNode) -> usize {
	size_of::<KernFSNode>() + node.get_size() as usize
}

/// Structure representing the temporary file system.
/// On the inside, the tmpfs works using a kernfs.
pub struct TmpFS {
	/// The maximum amount of memory in bytes the filesystem can use.
	max_size: usize,
	/// The currently used amount of memory in bytes.
	size: usize,

	/// The kernfs.
	fs: KernFS,
}

impl TmpFS {
	/// Creates a new instance.
	/// `max_size` is the maximum amount of memory the filesystem can use in bytes.
	/// `fs_id` is the ID of the mounted filesystem.
	/// `readonly` tells whether the filesystem is readonly.
	/// `mountpath` is the path at which the filesystem is mounted.
	pub fn new(max_size: usize, fs_id: u32, readonly: bool, mountpath: Path) -> Result<Self, Errno> {
		let mut fs = Self {
			max_size,
			size: 0,

			fs: KernFS::new(String::from(b"tmpfs")?, fs_id, readonly, mountpath)?,
		};

		// Adding the root node
		let root_node = KernFSNode::new(0o777, 0, 0, FileContent::Directory(HashMap::new()), None);
		fs.update_size(get_used_size(&root_node) as _, | fs | {
			fs.fs.set_root(root_node)?;
			Ok(())
		})?;

		Ok(fs)
	}

	/// Executes the given function `f`. On success, the function adds `s` to the total size of the
	/// filesystem.
	/// If `f` fails, the function doesn't change the total size and returns the error.
	/// If the new total size is too large, `f` is not executed and the function returns an error.
	fn update_size<F: FnOnce(&mut Self) -> Result<(), Errno>>(&mut self, s: isize, f: F)
		-> Result<(), Errno> {
		if s < 0 {
			f(self)?;

			if self.size < (-s as usize) {
				// If the result would underflow, set the total to zero
				self.size = 0;
			} else {
				self.size -= -s as usize;
			}

			Ok(())
		} else if self.size + (s as usize) < self.max_size {
			f(self)?;

			self.size += s as usize;
			Ok(())
		} else {
			Err(errno!(ENOSPC))
		}
	}
}

impl Filesystem for TmpFS {
	fn get_name(&self) -> &[u8] {
		self.fs.get_name()
	}

	fn get_id(&self) -> u32 {
		self.fs.get_id()
	}

	fn is_readonly(&self) -> bool {
		self.fs.is_readonly()
	}

	fn must_cache(&self) -> bool {
		self.fs.must_cache()
	}

	fn get_stat(&self, io: &mut dyn IO) -> Result<Statfs, Errno> {
		self.fs.get_stat(io)
	}

	fn get_root_inode(&self, io: &mut dyn IO) -> Result<INode, Errno> {
		self.fs.get_root_inode(io)
	}

	fn get_inode(&mut self, io: &mut dyn IO, parent: Option<INode>, name: &String)
		-> Result<INode, Errno> {
		self.fs.get_inode(io, parent, name)
	}

	fn load_file(&mut self, io: &mut dyn IO, inode: INode, name: String)
		-> Result<File, Errno> {
		self.fs.load_file(io, inode, name)
	}

	fn add_file(&mut self, io: &mut dyn IO, parent_inode: INode, name: String,
		uid: Uid, gid: Gid, mode: Mode, content: FileContent) -> Result<File, Errno> {
		// TODO Update fs's size
		self.fs.add_file(io, parent_inode, name, uid, gid, mode, content)
	}

	fn add_link(&mut self, io: &mut dyn IO, parent_inode: INode, name: &String,
		inode: INode) -> Result<(), Errno> {
		// TODO Update fs's size
		self.fs.add_link(io, parent_inode, name, inode)
	}

	fn update_inode(&mut self, io: &mut dyn IO, file: &File) -> Result<(), Errno> {
		// TODO Update fs's size
		self.fs.update_inode(io, file)
	}

	fn remove_file(&mut self, io: &mut dyn IO, parent_inode: INode, name: &String)
		-> Result<(), Errno> {
		// TODO Update fs's size
		self.fs.remove_file(io, parent_inode, name)
	}

	fn read_node(&mut self, io: &mut dyn IO, inode: INode, off: u64, buf: &mut [u8])
		-> Result<u64, Errno> {
		self.fs.read_node(io, inode, off, buf)
	}

	fn write_node(&mut self, io: &mut dyn IO, inode: INode, off: u64, buf: &[u8])
		-> Result<(), Errno> {
		// TODO Update fs's size
		self.fs.write_node(io, inode, off, buf)
	}
}

/// Structure representing the tmpfs file system type.
pub struct TmpFsType {}

impl FilesystemType for TmpFsType {
	fn get_name(&self) -> &[u8] {
		b"tmpfs"
	}

	fn detect(&self, _io: &mut dyn IO) -> Result<bool, Errno> {
		Ok(false)
	}

	fn create_filesystem(&self, _io: &mut dyn IO, fs_id: u32)
		-> Result<SharedPtr<dyn Filesystem>, Errno> {
		Ok(SharedPtr::new(TmpFS::new(DEFAULT_MAX_SIZE, fs_id, false, Path::root())?)?)
	}

	fn load_filesystem(&self, _io: &mut dyn IO, fs_id: u32, mountpath: Path, readonly: bool)
		-> Result<SharedPtr<dyn Filesystem>, Errno> {
		Ok(SharedPtr::new(TmpFS::new(DEFAULT_MAX_SIZE, fs_id, readonly, mountpath)?)?)
	}
}
