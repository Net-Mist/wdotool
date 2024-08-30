# Wdotool

A very light Python package to send mouse and keyboard actions to wayland and get screenshots as numpy arrays, throught Rust.

It supports deterministic actions:

```python
import logging

from wdotool import Wdotool

logging.basicConfig(level=logging.INFO)

w = Wdotool()
w.move_mouse(x_extent=2560, y_extent=1440, x=100, y=100)
w.right_click(duration_ms=10)
screen_image = w.screenshot()

assert screen_image.shape == (1440, 2560, 4)
```

Or statistical actions:

```python
import logging

from wdotool import Wdotool

logging.basicConfig(level=logging.INFO)

w = Wdotool("wayland-2")
w.move_mouse(x_extent=2560, y_extent=1440, x=100, y=100, x_max=120, y_max=120)
w.right_click(duration_ms=10, duration_ms_max=20)
screen_image = w.screenshot("HDMI-A-1")

assert screen_image.shape == (1440, 2560, 4)
```


If parameters `{p_name}` and `{p_name}_max` are defined, it will draw a random value in the range [`{p_name}`, `{p_name}_max`], following a normal distribution of mean `({p_name} + {p_name}_max)/2` and standard variation `({p_name}_max - {p_name})/2`.

The python package doesn't have any dependencies, except numpy, if you wish to do screenshots.

Your Wayland compositor need to support 3 protocols:
- [virtual-keyboard-unstable-v1](https://wayland.app/protocols/virtual-keyboard-unstable-v1), version 1
- [wlr-screencopy-unstable-v1](https://wayland.app/protocols/wlr-screencopy-unstable-v1), version 3
- [wlr-virtual-pointer-unstable-v1](https://wayland.app/protocols/wlr-virtual-pointer-unstable-v1) version 2

This solution has been developped under `Hyprland`, but according to the compatibility lists, should also work under `Sway` and `Mir`

## Special Thanks
- [wtype](https://github.com/atx/wtype) for showing how to do stuff in C
- [wev](https://github.com/jwrdegoede/wev) to have a look at client-side wayland events
