//! The `setuid32` syscall sets the UID of the process's owner.

use crate::errno::Errno;
use crate::file::Uid;
use crate::file::ROOT_UID;
use crate::process::Process;
use macros::syscall;

/// The implementation of the `setuid32` syscall.
#[syscall]
pub fn setuid32(uid: Uid) -> Result<i32, Errno> {
	let mutex = Process::get_current().unwrap();
	let guard = mutex.lock();
	let proc = guard.get_mut();

	// TODO Implement correctly
	if proc.get_uid() == ROOT_UID && proc.get_euid() == ROOT_UID {
		proc.set_uid(uid);
		proc.set_euid(uid);
		proc.set_suid(uid);

		Ok(0)
	} else {
		Err(errno!(EPERM))
	}
}
