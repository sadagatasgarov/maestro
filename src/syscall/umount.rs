//! The `umount` system call allows to unmount a filesystem previously mounted
//! with `mount`.

use crate::errno;
use crate::errno::Errno;
use crate::file::mountpoint;
use crate::file::path::Path;
use crate::process::mem_space::ptr::SyscallString;
use crate::process::Process;
use macros::syscall;

/// The implementation of the `umount` syscall.
#[syscall]
pub fn umount(target: SyscallString) -> Result<i32, Errno> {
	let mutex = Process::get_current().unwrap();
	let guard = mutex.lock();
	let proc = guard.get();

	// Getting a slice to the string
	let mem_space = proc.get_mem_space().unwrap();
	let mem_space_guard = mem_space.lock();
	let target_slice = target.get(&mem_space_guard)?.ok_or(errno!(EFAULT))?;

	// Getting the mountpoint
	let target_path = Path::from_str(target_slice, true)?;
	let _mountpoint = mountpoint::from_path(&target_path).ok_or(errno!(EINVAL))?;

	// TODO Check if busy (EBUSY)
	// TODO If not, sync and unmount

	Ok(0)
}
