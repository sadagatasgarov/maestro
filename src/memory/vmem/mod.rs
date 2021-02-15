/// The virtual memory makes the kernel able to isolate processes, which is essential for modern
/// systems.

// TODO Make this file fully cross-platform

// TODO Only if on the corresponding architecture
pub mod x86;

use core::ffi::c_void;
use crate::util::boxed::Box;

/// Trait representing virtual memory context handler. This trait is the interface to manipulate
/// virtual memory on any architecture. Each architecture has its own structure implementing this
/// trait.
pub trait VMem {
	/// Translates the given virtual address `ptr` to the corresponding physical address. If the
	/// address is not mapped, the function returns None.
	fn translate(&self, ptr: *const c_void) -> Option<*const c_void>;

	/// Tells whether the given pointer `ptr` is mapped or not.
	fn is_mapped(&self, ptr: *const c_void) -> bool {
		self.translate(ptr) != None
	}

	/// Maps the the given physical address `physaddr` to the given virtual address `virtaddr` with
	/// the given flags.
	fn map(&mut self, physaddr: *const c_void, virtaddr: *const c_void, flags: u32)
		-> Result<(), ()>;
	/// Maps the given range of physical address `physaddr` to the given range of virtual address
	/// `virtaddr`. The range is `pages` pages large.
	fn map_range(&mut self, physaddr: *const c_void, virtaddr: *const c_void, pages: usize,
		flags: u32) -> Result<(), ()>;

	/// Maps the physical address `ptr` to the same address in virtual memory with the given flags
	/// `flags`.
	fn identity(&mut self, ptr: *const c_void, flags: u32) -> Result<(), ()> {
		self.map(ptr, ptr, flags)
	}
	/// Identity maps a range beginning at physical address `from` with pages `pages` and flags
	/// `flags`.
	fn identity_range(&mut self, ptr: *const c_void, pages: usize, flags: u32) -> Result<(), ()> {
		self.map_range(ptr, ptr, pages, flags)
	}

	/// Unmaps the page at virtual address `virtaddr`.
	fn unmap(&mut self, virtaddr: *const c_void) -> Result<(), ()>;
	/// Unmaps the given range beginning at virtual address `virtaddr` with size of `pages` pages.
	fn unmap_range(&mut self, virtaddr: *const c_void, pages: usize) -> Result<(), ()>;

	/// Clones the context, creating a new one pointing towards the same physical pages.
	fn clone(&self) -> Result::<Self, ()> where Self: Sized;

	/// Binds the virtual memory context handler.
	fn bind(&self);
	/// Tells whether the handler is bound or not.
	fn is_bound(&self) -> bool;
	/// Flushes the modifications of the context if bound. This function should be called after
	/// applying modifications to the context.
	fn flush(&self);
}

/// Creates a new virtual memory context handler for the current architecture.
pub fn new() -> Result::<Box::<dyn VMem>, ()> {
	Ok(Box::new(x86::X86VMem::new()?)? as Box::<dyn VMem>)
}

// TODO Handle leak
/// Creates and loads the kernel's memory protection, protecting its code from writing.
pub fn kernel() {
	if let Ok(kernel_vmem) = new() {
		kernel_vmem.bind();
	} else {
		crate::kernel_panic!("Cannot initialize kernel virtual memory!", 0);
	}
}

/// Tells whether the read-only pages protection is enabled.
pub fn is_write_lock() -> bool {
	unsafe {
		(x86::cr0_get() & (1 << 16)) != 0
	}
}

/// Sets whether the kernel can write to read-only pages.
pub fn set_write_lock(lock: bool) {
	if lock {
		unsafe {
			x86::cr0_set(1 << 16);
		}
	} else {
		unsafe {
			x86::cr0_clear(1 << 16);
		}
	}
}

/// Executes the closure given as parameter. During execution, the kernel can write on read-only
/// pages. The state of the write lock is restored after the closure's execution.
pub unsafe fn write_lock_wrap<T: Fn()>(f: T) {
	let lock = is_write_lock();
	set_write_lock(false);

	f();

	set_write_lock(lock);
}
