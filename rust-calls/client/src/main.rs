use nix::sys::memfd::*;
use nix::sys::mman::*;
use nix::unistd::ftruncate;
use std::fs::File;
use std::num::*;
use std::os::*;
use std::process::*;
use wayland_client::protocol::wl_compositor;
use wayland_client::protocol::{
    wl_buffer::WlBuffer,
    wl_compositor::WlCompositor,
    wl_registry::{self, WlRegistry},
    wl_shm::{Format, WlShm},
    wl_shm_pool::WlShmPool,
    wl_surface::WlSurface,
};
use wayland_client::{Connection, Dispatch, QueueHandle};




// init the toolbox with all the imports...
#[derive(Default)]
struct ClientState {
    compositor: Option<WlCompositor>, // every state that can be nulled has to be saved as Option
    shm: Option<WlShm>,
    surface: Option<WlSurface>,
}

// =========================================================================
// HELPER FUNCTIONS


// creating the anonymous file on RAM...
fn create_shm(size: usize) -> Result<File, Box<dyn std::error::Error>> {
    let owned_fd = memfd_create(c"shm_pixel_biffer", MemFdCreateFlag::MFD_CLOEXEC)?;
    ftruncate(&owned_fd, size as i64)?;
    let file = File::from(owned_fd);
    Ok(file)
}

// the registry listener whenever the server announces an event over a socket
// this has to run...
impl Dispatch<WlRegistry, ()> for ClientState {
    
}




fn main() {
    println!("Hello, world!");
}
