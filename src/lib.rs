pub mod wdotool_lib;

use numpy::PyArray3;
use pyo3::prelude::*;
use wdotool_lib::UIntValue;

#[pyclass]
struct Wdotool {
    internal: wdotool_lib::Wdotool,
}

#[pymethods]
impl Wdotool {
    #[new]
    #[pyo3(signature = (wayland_display=None))]
    pub fn new(wayland_display: Option<&str>) -> anyhow::Result<Self> {
        let mut internal = wdotool_lib::Wdotool::connect(wayland_display)?;
        internal.wait_ouput_detected()?;
        Ok(Wdotool { internal })
    }

    #[pyo3(signature = (x_extent, y_extent, x, y, x_max=None, y_max=None))]
    pub fn move_mouse(
        &mut self,
        x_extent: u32,
        y_extent: u32,
        x: u32,
        y: u32,
        x_max: Option<u32>,
        y_max: Option<u32>,
    ) -> anyhow::Result<()> {
        let x = match x_max {
            Some(x_max) => UIntValue::UIntRange(x, x_max),
            None => UIntValue::UInt(x),
        };
        let y = match y_max {
            Some(y_max) => UIntValue::UIntRange(y, y_max),
            None => UIntValue::UInt(y),
        };

        self.internal.move_mouse(x, y, x_extent, y_extent)?;
        Ok(())
    }

    #[pyo3(signature = (duration_ms, duration_ms_max=None))]
    pub fn left_click(
        &mut self,
        duration_ms: u32,
        duration_ms_max: Option<u32>,
    ) -> anyhow::Result<()> {
        let duration_ms = match duration_ms_max {
            Some(duration_ms_max) => UIntValue::UIntRange(duration_ms, duration_ms_max),
            None => UIntValue::UInt(duration_ms),
        };

        self.internal.left_click(duration_ms)?;
        Ok(())
    }

    #[pyo3(signature = (duration_ms, duration_ms_max=None))]
    pub fn right_click(
        &mut self,
        duration_ms: u32,
        duration_ms_max: Option<u32>,
    ) -> anyhow::Result<()> {
        let duration_ms = match duration_ms_max {
            Some(duration_ms_max) => UIntValue::UIntRange(duration_ms, duration_ms_max),
            None => UIntValue::UInt(duration_ms),
        };

        self.internal.right_click(duration_ms)?;
        Ok(())
    }

    #[pyo3(signature = (key, duration_ms, duration_ms_max=None))]
    pub fn key_press(
        &mut self,
        key: u32,
        duration_ms: u32,
        duration_ms_max: Option<u32>,
    ) -> anyhow::Result<()> {
        let duration_ms = match duration_ms_max {
            Some(duration_ms_max) => UIntValue::UIntRange(duration_ms, duration_ms_max),
            None => UIntValue::UInt(duration_ms),
        };

        self.internal.key_press(key, duration_ms)?;
        Ok(())
    }

    #[pyo3(signature = (screen_name=None))]
    pub fn screenshot(&mut self, screen_name: Option<&str>) -> anyhow::Result<Py<PyArray3<u8>>> {
        let screenshot = self.internal.screenshot(screen_name)?;

        Python::with_gil(|py| {
            let a = PyArray3::from_owned_array_bound(py, screenshot).unbind();
            Ok(a)
        })
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn wdotool(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<Wdotool>()?;
    Ok(())
}
