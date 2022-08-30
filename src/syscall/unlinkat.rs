//! The `unlinkat` syscall allows to unlink a file.

use super::util;
use crate::errno::Errno;
use crate::file::fcache;
use crate::process::mem_space::ptr::SyscallString;
use crate::process::regs::Regs;
use crate::process::Process;

/// The implementation of the `unlinkat` syscall.
pub fn unlinkat(regs: &Regs) -> Result<i32, Errno> {
	let dirfd = regs.ebx as i32;
	let pathname: SyscallString = (regs.ecx as usize).into();
	let flags = regs.edx as i32;

	let (file_mutex, uid, gid) = {
		let mutex = Process::get_current().unwrap();
		let guard = mutex.lock();
		let proc = guard.get_mut();

		let uid = proc.get_euid();
		let gid = proc.get_egid();

		let mem_space = proc.get_mem_space().unwrap();
		let mem_space_guard = mem_space.lock();
		let pathname = pathname
			.get(&mem_space_guard)?
			.ok_or_else(|| errno!(EFAULT))?;

		let file = util::get_file_at(guard, false, dirfd, pathname, flags)?;

		(file, uid, gid)
	};
	let file_guard = file_mutex.lock();
	let file = file_guard.get_mut();

	let fcache_mutex = fcache::get();
	let fcache_guard = fcache_mutex.lock();
	let files_cache = fcache_guard.get_mut().as_mut().unwrap();

	files_cache.remove_file(file, uid, gid)?;

	Ok(0)
}