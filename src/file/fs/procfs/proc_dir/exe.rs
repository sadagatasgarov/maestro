//! This module implements the `exe` node, which is a link to the executable file of the process.

use crate::errno::Errno;
use crate::file::fs::kernfs::node::KernFSNode;
use crate::file::FileContent;
use crate::file::Gid;
use crate::file::Mode;
use crate::file::Uid;
use crate::process::oom;
use crate::process::pid::Pid;
use crate::process::Process;
use crate::util::io::IO;
use crate::util::ptr::cow::Cow;

/// Struture representing the `exe` node.
pub struct Exe {
	/// The PID of the process.
	pub pid: Pid,
}

impl KernFSNode for Exe {
	fn get_mode(&self) -> Mode {
		0o777
	}

	fn get_uid(&self) -> Uid {
		let proc_mutex = Process::get_by_pid(self.pid).unwrap();
		let proc_guard = proc_mutex.lock();
		let proc = proc_guard.get();

		proc.get_euid()
	}

	fn get_gid(&self) -> Gid {
		let proc_mutex = Process::get_by_pid(self.pid).unwrap();
		let proc_guard = proc_mutex.lock();
		let proc = proc_guard.get();

		proc.get_egid()
	}

	fn get_content<'a>(&'a self) -> Cow<'a, FileContent> {
		let proc_mutex = Process::get_by_pid(self.pid).unwrap();
		let proc_guard = proc_mutex.lock();
		let proc = proc_guard.get();

		let s = oom::wrap(|| proc.get_exec_path().as_string());
		Cow::from(FileContent::Link(s))
	}
}

impl IO for Exe {
	fn get_size(&self) -> u64 {
		0
	}

	fn read(&mut self, _offset: u64, _buff: &mut [u8]) -> Result<(u64, bool), Errno> {
		Err(errno!(EINVAL))
	}

	fn write(&mut self, _offset: u64, _buff: &[u8]) -> Result<u64, Errno> {
		Err(errno!(EINVAL))
	}

	fn poll(&mut self, _mask: u32) -> Result<u32, Errno> {
		Err(errno!(EINVAL))
	}
}