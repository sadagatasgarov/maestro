//! This module implements the status file, which allows to retrieve the current status of the
//! process.

use crate::errno::Errno;
use crate::file::fs::kernfs::node::KernFSNode;
use crate::file::FileContent;
use crate::file::Gid;
use crate::file::Mode;
use crate::file::Uid;
use crate::process::pid::Pid;
use crate::process::Process;
use crate::util::io::IO;
use crate::util::ptr::cow::Cow;
use core::cmp::min;

/// Structure representing the status node of the procfs.
pub struct Status {
	/// The PID of the process.
	pub pid: Pid,
}

impl KernFSNode for Status {
	fn get_mode(&self) -> Mode {
		0o444
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
		Cow::from(FileContent::Regular)
	}
}

impl IO for Status {
	fn get_size(&self) -> u64 {
		0
	}

	fn read(&mut self, offset: u64, buff: &mut [u8]) -> Result<(u64, bool), Errno> {
		if buff.is_empty() {
			return Ok((0, false));
		}

		let proc_mutex = Process::get_current().unwrap();
		let proc_guard = proc_mutex.lock();
		let proc = proc_guard.get();

		let name = proc.get_argv()
			.iter()
			.map(| name | unsafe { name.as_str_unchecked() })
			.next()
			.unwrap_or("?");

		let umask = proc.get_umask();

		let state = proc.get_state();
		let state_char = state.get_char();
		let state_name = state.as_str();

		let pid = proc.get_pid();
		let ppid = proc.get_parent_pid();

		let uid = proc.get_uid();
		let euid = proc.get_euid();
		let suid = proc.get_suid();

		let gid = proc.get_gid();
		let egid = proc.get_egid();
		let sgid = proc.get_sgid();

		// TODO Fill every fields with process's data
		// Generating content
		let content = crate::format!("Name: {name}
Umask: {umask:4o}
State: {state_char} ({state_name})
Tgid: 0
Ngid: 0
Pid: {pid}
PPid: {ppid}
TracerPid: 0
Uid: {uid} {euid} {suid} TODO
Gid: {gid} {egid} {sgid} TODO
FDSize: TODO
Groups: TODO
NStgid: TODO
NSpid: TODO
NSpgid: TODO
NSsid: TODO
VmPeak: TODO kB
VmSize: TODO kB
VmLck: TODO kB
VmPin: TODO kB
VmHWM: TODO kB
VmRSS: TODO kB
RssAnon: TODO kB
RssFile: TODO kB
RssShmem: TODO kB
VmData: TODO kB
VmStk: TODO kB
VmExe: TODO kB
VmLib: TODO kB
VmPTE: TODO kB
VmSwap: TODO kB
HugetlbPages: TODO kB
CoreDumping: TODO
THP_enabled: TODO
Threads: TODO
SigQ: TODO/TODO
SigPnd: 0000000000000000
ShdPnd: 0000000000000000
SigBlk: 0000000000000000
SigIgn: 0000000000000000
SigCgt: 0000000000000000
CapInh: 0000000000000000
CapPrm: 0000000000000000
CapEff: 0000000000000000
CapBnd: 000001ffffffffff
CapAmb: 0000000000000000
NoNewPrivs: 0
Seccomp: 0
Seccomp_filters: 0
Speculation_Store_Bypass: thread vulnerable
SpeculationIndirectBranch: conditional enabled
Cpus_allowed: ff
Cpus_allowed_list: 0-7
Mems_allowed: 00000001
Mems_allowed_list: 0
voluntary_ctxt_switches: 0
nonvoluntary_ctxt_switches: 0
")?;

		// Copying content to userspace buffer
		let content_bytes = content.as_bytes();
		let len = min((content_bytes.len() as u64 - offset) as usize, buff.len());
		buff[..len].copy_from_slice(&content_bytes[(offset as usize)..(offset as usize + len)]);

		let eof = (offset + len as u64) >= content_bytes.len() as u64;
		Ok((len as _, eof))
	}

	fn write(&mut self, _offset: u64, _buff: &[u8]) -> Result<u64, Errno> {
		Err(errno!(EINVAL))
	}

	fn poll(&mut self, _mask: u32) -> Result<u32, Errno> {
		// TODO
		todo!();
	}
}