use anyhow::Result;
use libc::{ftruncate, shm_open};
use libc::{shm_unlink, O_EXCL};
use libc::{O_CREAT, O_RDWR, S_IRUSR, S_IWUSR};
use log::info;
use rand::distributions::{Alphanumeric, DistString};
use std::ffi::CString;
use std::fs::File;
use std::os::fd::FromRawFd;

// idea of the code comes from https://wayland-book.com/surfaces/shared-memory.html

fn randname() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 6)
}

pub fn create_shm_file(size: usize) -> Result<File> {
    let mut retries = 100;
    let mut name = "/wl_shm-".to_string();
    while retries > 0 {
        name.push_str(&randname());
        info!("Trying to create shm file: {}", name);
        let cname = CString::new(&name[..]).unwrap();
        unsafe {
            let fd = shm_open(cname.as_ptr(), O_RDWR | O_CREAT | O_EXCL, S_IRUSR | S_IWUSR);
            if fd >= 0 {
                shm_unlink(cname.as_ptr());
                let ret = ftruncate(fd, size as libc::off_t);
                if ret == 0 {
                    return Ok(File::from_raw_fd(fd));
                } else {
                    break;
                }
            }
        }
        retries -= 1;
    }
    let err = std::io::Error::last_os_error();
    if err.raw_os_error() != Some(libc::EEXIST) {
        anyhow::bail!("Failed to create shm file: {}", err);
    }
    anyhow::bail!("Failed to create shm file")
}
