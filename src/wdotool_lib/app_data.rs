use std::{collections::HashMap, os::fd::OwnedFd};

use log::info;
use wayland_client::{
    protocol::{
        wl_buffer,
        wl_keyboard::{self, KeymapFormat},
        wl_output, wl_registry, wl_seat,
        wl_shm::{self, Format},
        wl_shm_pool,
    },
    Connection, Dispatch, QueueHandle, WEnum,
};

use super::{
    screencopy::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1},
    virtual_keyboard::{zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1},
    virtual_pointer::{zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1},
};

pub struct Keymap {
    pub format: WEnum<KeymapFormat>,
    pub fd: OwnedFd,
    pub size: u32,
}

pub struct Buffer {
    pub format: WEnum<Format>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

impl Buffer {
    pub fn size(&self) -> usize {
        // 4 because R, G, B, A
        4 * self.height as usize * self.width as usize
    }
}

pub struct Output {
    pub output: wl_output::WlOutput,
    pub name: Option<String>,
}

pub struct Screencopy {
    pub frame: zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
    pub buffer: Option<Buffer>,
}

impl Screencopy {
    pub fn new(frame: zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1) -> Self {
        Screencopy {
            frame,
            buffer: None,
        }
    }
}

#[derive(Default)]
pub struct AppData {
    pub seat: Option<wl_seat::WlSeat>,
    pub vkm: Option<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1>,
    pub vpm: Option<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1>,
    pub keymap: Option<Keymap>,
    pub screencopy_manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    pub outputs: HashMap<u32, Output>,
    pub shm: Option<wl_shm::WlShm>,
    pub screencopy: Option<Screencopy>,
    pub screencopy_in_progress: bool,
}

impl AppData {
    pub fn get_output_by_name(&self, name: &str) -> Option<&wl_output::WlOutput> {
        for output in self.outputs.values() {
            if let Some(output_name) = &output.name {
                if output_name == name {
                    return Some(&output.output);
                }
            }
        }
        None
    }

    pub fn all_output_name_set(&self) -> bool {
        for output in self.outputs.values() {
            if output.name.is_none() {
                return false;
            }
        }
        true
    }

    pub fn screencopy_buffer_set(&self) -> bool {
        self.screencopy.as_ref().unwrap().buffer.is_some()
    }
}

// note that most wayland objects never send a signal (as the app doesn't have a display)
// but the displatch need to be implemented for the object to be usable

impl Dispatch<wl_seat::WlSeat, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Seat event: {:?}", event);
    }
}

impl Dispatch<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
        event: zwp_virtual_keyboard_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Virtual keyboard manager event: {:?}", event);
    }
}

impl Dispatch<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
        event: zwlr_virtual_pointer_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        // ZwlrVirtualPointerManagerV1 doesn't have event for now, but just in case...
        info!("Virtual pointer manager event: {:?}", event);
    }
}

impl Dispatch<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
        event: zwlr_screencopy_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Screencopy manager event: {:?}", event);
    }
}

impl Dispatch<wl_output::WlOutput, u32> for AppData {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        name: &u32,
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("WlOutput event for {name}: {:?}", event);
        if let wl_output::Event::Name { name: output_name } = event {
            state.outputs.get_mut(name).unwrap().name = Some(output_name);
        }
    }
}

impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Screencopy event: {:?}", event);
        if let zwlr_screencopy_frame_v1::Event::Buffer {
            format,
            width,
            height,
            stride,
        } = event
        {
            state.screencopy.as_mut().unwrap().buffer = Some(Buffer {
                format,
                width,
                height,
                stride,
            });
        } else if let zwlr_screencopy_frame_v1::Event::Ready {
            tv_sec_hi: _,
            tv_sec_lo: _,
            tv_nsec: _,
        } = event
        {
            // screencopy is ready
            state.screencopy_in_progress = false;
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        event: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Shm event: {:?}", event);
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        event: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Shm event: {:?}", event);
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Shm event: {:?}", event);
    }
}

impl Dispatch<zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
        event: zwp_virtual_keyboard_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Virtual keyboard event: {:?}", event);
    }
}

impl Dispatch<zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
        event: zwlr_virtual_pointer_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Virtual pointer event: {:?}", event);
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        info!("Keyboard event: {:?}", event);
        if let wl_keyboard::Event::Keymap { format, fd, size } = event {
            state.keymap = Some(Keymap { format, fd, size });
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        info!("Registry event: {:?}", event);
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == *"wl_seat" {
                state.seat = Some(registry.bind(name, version, qh, ()));
            } else if interface == *"zwp_virtual_keyboard_manager_v1" {
                state.vkm = Some(registry.bind(name, version, qh, ()));
            } else if interface == *"zwlr_virtual_pointer_manager_v1" {
                state.vpm = Some(registry.bind(name, version, qh, ()));
            } else if interface == *"zwlr_screencopy_manager_v1" {
                state.screencopy_manager = Some(registry.bind(name, version, qh, ()));
            } else if interface == *"wl_output" {
                state.outputs.insert(
                    name,
                    Output {
                        output: registry.bind(name, version, qh, name),
                        name: None,
                    },
                );
            } else if interface == *"wl_shm" {
                state.shm = Some(registry.bind(name, version, qh, ()));
            }
        }
    }
}
