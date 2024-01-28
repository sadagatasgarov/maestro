//! Utility functions for system calls.

pub mod at;

use crate::errno;
use crate::errno::EResult;
use crate::process::mem_space::ptr::SyscallString;
use crate::process::regs::Regs;
use crate::process::scheduler;
use crate::process::Process;
use crate::process::State;
use crate::util::container::string::String;
use crate::util::container::vec::Vec;
use core::mem::size_of;

// TODO Find a safer and cleaner solution
/// Checks that the given array of strings at pointer `ptr` is accessible to
/// process `proc`, then returns its content.
///
/// If the array or its content strings are not accessible by the process, the
/// function returns an error.
pub unsafe fn get_str_array(process: &Process, ptr: *const *const u8) -> EResult<Vec<String>> {
	let mem_space = process.get_mem_space().unwrap();
	let mem_space_guard = mem_space.lock();

	// Checking every elements of the array and counting the number of elements
	let mut len = 0;
	loop {
		let elem_ptr = ptr.add(len);

		// Checking access on elem_ptr
		if !mem_space_guard.can_access(elem_ptr as _, size_of::<*const u8>(), true, false) {
			return Err(errno!(EFAULT));
		}

		// Safe because the access is checked before
		let elem = *elem_ptr;
		if elem.is_null() {
			break;
		}

		len += 1;
	}

	// Filling the array
	// TODO collect
	let mut arr = Vec::with_capacity(len)?;
	for i in 0..len {
		let elem = *ptr.add(i);
		let s: SyscallString = (elem as usize).into();

		arr.push(String::try_from(s.get(&mem_space_guard)?.unwrap())?)?;
	}

	Ok(arr)
}

/// Updates the execution flow of the current process according to its state.
///
/// When the state of the current process has been changed, execution may not
/// resume. In which case, the current function handles the execution flow
/// accordingly.
///
/// The function locks the mutex of the current process. Thus, the caller must
/// ensure the mutex isn't already locked to prevent a deadlock.
///
/// If returning, the function returns the mutex lock of the current process.
pub fn handle_proc_state() {
	let proc_mutex = Process::current_assert();
	let proc = proc_mutex.lock();

	match proc.get_state() {
		// The process is executing a signal handler. Make the scheduler jump to it
		State::Running => {
			if proc.is_handling_signal() {
				let regs = proc.regs.clone();
				drop(proc);
				drop(proc_mutex);

				unsafe {
					regs.switch(true);
				}
			}
		}

		// The process is sleeping or has been stopped. Waiting until wakeup
		State::Sleeping | State::Stopped => {
			drop(proc);
			drop(proc_mutex);

			scheduler::end_tick();
		}

		// The process has been killed. Stopping execution and waiting for the next tick
		State::Zombie => {
			drop(proc);
			drop(proc_mutex);

			scheduler::end_tick();
		}
	}
}

/// Checks whether the current syscall must be interrupted to execute a signal.
///
/// If interrupted, the function doesn't return and the control flow jumps
/// directly to handling the signal.
///
/// The function locks the mutex of the current process. Thus, the caller must
/// ensure the mutex isn't already locked to prevent a deadlock.
///
/// `regs` is the registers state passed to the current syscall.
pub fn signal_check(regs: &Regs) {
	let proc_mutex = Process::current_assert();
	let mut proc = proc_mutex.lock();

	if proc.get_next_signal().is_some() {
		// Returning the system call early to resume it later
		let mut r = regs.clone();
		// TODO Clean
		r.eip -= 2; // TODO Handle the case where the instruction isn't two bytes long (sysenter)
		proc.regs = r;
		proc.syscalling = false;

		// Switching to handle the signal
		proc.prepare_switch();

		drop(proc);
		drop(proc_mutex);

		handle_proc_state();
	}
}