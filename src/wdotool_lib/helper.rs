use std::{
    env,
    io::Read,
    os::{
        fd::{AsFd, BorrowedFd, IntoRawFd},
        unix::net::UnixStream,
    },
    path::PathBuf,
};

use anyhow::{Context, Result};
use ndarray::{Array, Array3};
use wayland_client::{
    protocol::{wl_keyboard, wl_shm},
    Connection, EventQueue, QueueHandle,
};

use crate::wdotool_lib::app_data::Screencopy;

use super::{app_data::AppData, shm::create_shm_file, virtual_keyboard::zwp_virtual_keyboard_v1};

/// Connect to the wayland compositor
///
/// This function will try to connect to the wayland compositor using the
/// user-provided display, or the WAYLAND_DISPLAY environment variable, or the
/// default wayland-0 socket.
///
/// args:
///    display: Option<&str> - The name of the display to connect to. If not a full path, it will be
///                           looked up in the XDG_RUNTIME_DIR directory.
pub fn connect_wayland(display: Option<&str>) -> Result<Connection> {
    let socket_file: PathBuf = match display {
        Some(display) => display.into(),
        None => env::var("WAYLAND_DISPLAY")
            .unwrap_or_else(|_| "wayland-0".into())
            .into(),
    };

    let socket_path = if socket_file.is_absolute() {
        socket_file
    } else {
        let mut socket_path: PathBuf = env::var("XDG_RUNTIME_DIR")
            .context("no XDG_RUNTIME_DIR set")?
            .into();
        socket_path.push(socket_file);
        socket_path
    };

    let socket = UnixStream::connect(socket_path.clone())
        .context(format!("failed to connect to unix stream {socket_path:?}"))?;
    Connection::from_socket(socket).context("failed to connect to wayland compositor")
}

pub fn setup_virtual_keyboard(
    mut app_data: AppData,
    qh: &QueueHandle<AppData>,
    event_queue: &mut EventQueue<AppData>,
) -> (AppData, zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1) {
    // get keymap from current keyboard
    app_data.seat.as_ref().unwrap().get_keyboard(qh, ());
    event_queue.roundtrip(&mut app_data).unwrap();

    let virtual_keyboard = app_data.vkm.as_ref().unwrap().create_virtual_keyboard(
        app_data.seat.as_ref().unwrap(),
        qh,
        (),
    );
    // upload_keymap we got from the current keyboard
    let keymap = app_data.keymap.unwrap();
    app_data.keymap = None;

    let fd = keymap.fd;
    let fd = fd.into_raw_fd();
    let fd = unsafe { BorrowedFd::borrow_raw(fd) };
    virtual_keyboard.keymap(wl_keyboard::KeymapFormat::XkbV1.into(), fd, keymap.size);
    event_queue.roundtrip(&mut app_data).unwrap();

    (app_data, virtual_keyboard)
}

pub fn screenshot(
    app_data: &mut AppData,
    qh: &QueueHandle<AppData>,
    event_queue: &mut EventQueue<AppData>,
    output_name: Option<&str>,
) -> Result<Array3<u8>> {
    let output = match output_name {
        Some(name) => app_data
            .get_output_by_name(name)
            .context(format!("no WLOutput with name {name}"))?,
        None => {
            if app_data.outputs.len() > 1 {
                anyhow::bail!(
                    "more that one WLOuput set. Please specify the name of the one to use"
                )
            }

            let k_v = app_data
                .outputs
                .iter()
                .next()
                .context("at least one display need to be set")?;
            &k_v.1.output
        }
    };

    let screencopy_frame = app_data
        .screencopy_manager
        .as_ref()
        .context("no screencopy manager")?
        .capture_output(0, output, qh, ());
    app_data.screencopy = Some(Screencopy::new(screencopy_frame));
    event_queue.roundtrip(app_data)?;

    while !app_data.screencopy_buffer_set() {
        event_queue.blocking_dispatch(app_data)?;
    }

    let buffer_param = app_data
        .screencopy
        .as_ref()
        .unwrap()
        .buffer
        .as_ref()
        .unwrap();

    let width = buffer_param.width as i32;
    let height = buffer_param.height as i32;
    let stride = buffer_param.stride as i32;

    let mut file = create_shm_file(buffer_param.size())?;
    let fd = file.as_fd();

    let wl_shm_pool = app_data
        .shm
        .as_ref()
        .context("no shared memory")?
        .create_pool(fd, buffer_param.size() as i32, qh, ());

    event_queue.roundtrip(app_data)?;

    let buffer =
        wl_shm_pool.create_buffer(0, width, height, stride, wl_shm::Format::Xrgb8888, qh, ());
    event_queue.roundtrip(app_data)?;

    app_data.screencopy.as_ref().unwrap().frame.copy(&buffer);
    app_data.screencopy_in_progress = true;

    while app_data.screencopy_in_progress {
        event_queue.blocking_dispatch(app_data)?;
    }

    app_data.screencopy.as_ref().unwrap().frame.destroy();
    app_data.screencopy = None;

    let mut buf = vec![0u8; height as usize * width as usize * 4];
    file.read_exact(&mut buf[..])?;
    let array = Array::from_vec(buf)
        .to_shape((height as usize, width as usize, 4))?
        .to_owned();
    Ok(array)
}
