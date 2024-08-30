pub mod app_data;
pub mod helper;
pub mod screencopy;
pub mod shm;
pub mod virtual_keyboard;
pub mod virtual_pointer;

use anyhow::{Context, Result};
use app_data::AppData;
use helper::{connect_wayland, screenshot, setup_virtual_keyboard};
use ndarray::prelude::*;
use rand_distr::{Distribution, Normal};
use virtual_keyboard::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;
use virtual_pointer::zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1;
use wayland_client::{protocol::wl_pointer, EventQueue, QueueHandle};

pub enum UIntValue {
    UInt(u32),
    UIntRange(u32, u32),
}

impl UIntValue {
    pub fn get(&self) -> Result<u32> {
        match self {
            UIntValue::UInt(value) => Ok(*value),
            UIntValue::UIntRange(min, max) => {
                let mean = (min + max) / 2;
                let std_dev = (max - min) / 2;
                let normal = Normal::new(mean as f32, std_dev as f32)
                    .context("invalid normal distribution")?;
                let v = normal.sample(&mut rand::thread_rng()) as u32;
                Ok(v.max(*min).min(*max) as u32)
            }
        }
    }
}

pub struct Wdotool {
    app_data: AppData,
    event_queue: EventQueue<AppData>,
    queue_handle: QueueHandle<AppData>,
    keyboard: ZwpVirtualKeyboardV1,
    pointer: ZwlrVirtualPointerV1,
}

impl Wdotool {
    pub fn connect(wayland_display: Option<&str>) -> Result<Wdotool> {
        let connection = connect_wayland(wayland_display)?;
        let display = connection.display();
        let mut event_queue = connection.new_event_queue();
        let queue_handle = event_queue.handle();

        // Call the registry to get global objects
        display.get_registry(&queue_handle, ());
        let mut app_data = AppData::default();
        event_queue.roundtrip(&mut app_data).unwrap();

        let (mut app_data, keyboard) =
            setup_virtual_keyboard(app_data, &queue_handle, &mut event_queue);

        // Virtual pointer
        let pointer = app_data.vpm.as_ref().unwrap().create_virtual_pointer(
            app_data.seat.as_ref(),
            &queue_handle,
            (),
        );
        event_queue.roundtrip(&mut app_data).unwrap();

        Ok(Wdotool {
            app_data,
            event_queue,
            queue_handle,
            keyboard,
            pointer,
        })
    }

    pub fn wait_ouput_detected(&mut self) -> Result<()> {
        while !self.app_data.all_output_name_set() {
            self.event_queue.blocking_dispatch(&mut self.app_data)?;
        }
        Ok(())
    }

    pub fn screenshot(&mut self, screen_name: Option<&str>) -> Result<Array3<u8>> {
        let array = screenshot(
            &mut self.app_data,
            &self.queue_handle,
            &mut self.event_queue,
            screen_name,
        )?;
        Ok(array)
    }

    pub fn move_mouse(
        &mut self,
        x: UIntValue,
        y: UIntValue,
        x_extent: u32,
        y_extent: u32,
    ) -> Result<()> {
        let x = x.get()?;
        let y = y.get()?;
        self.pointer.motion_absolute(0, x, y, x_extent, y_extent);
        self.event_queue.roundtrip(&mut self.app_data)?;
        Ok(())
    }

    pub fn left_click(&mut self, duration_ms: UIntValue) -> Result<()> {
        let duration_ms = duration_ms.get()?;
        self.pointer
            .button(0, 272, wl_pointer::ButtonState::Pressed);
        self.event_queue.roundtrip(&mut self.app_data)?;
        std::thread::sleep(std::time::Duration::from_millis(duration_ms as u64));
        self.pointer
            .button(0, 272, wl_pointer::ButtonState::Released);
        self.event_queue.roundtrip(&mut self.app_data)?;
        Ok(())
    }

    pub fn right_click(&mut self, duration_ms: UIntValue) -> Result<()> {
        let duration_ms = duration_ms.get()?;
        self.pointer
            .button(0, 273, wl_pointer::ButtonState::Pressed);
        self.event_queue.roundtrip(&mut self.app_data)?;
        std::thread::sleep(std::time::Duration::from_millis(duration_ms as u64));
        self.pointer
            .button(0, 273, wl_pointer::ButtonState::Released);
        self.event_queue.roundtrip(&mut self.app_data)?;
        Ok(())
    }

    pub fn key_press(&mut self, key: u32, duration_ms: UIntValue) -> Result<()> {
        self.keyboard.key(0, key, 1);
        self.event_queue.roundtrip(&mut self.app_data)?;

        // sleep
        let duration_ms = duration_ms.get()?;
        std::thread::sleep(std::time::Duration::from_millis(duration_ms as u64));

        self.keyboard.key(0, key, 0);
        self.event_queue.roundtrip(&mut self.app_data)?;

        Ok(())
    }
}
