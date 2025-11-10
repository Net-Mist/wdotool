from typing import Optional, Tuple
import numpy as np

__all__ = ["Wdotool"]

class Wdotool:
    def __init__(self, wayland_display: Optional[str] = None) -> None: ...
    
    def move_mouse(
        self,
        x_extent: int,
        y_extent: int,
        x: int,
        y: int,
        x_max: Optional[int] = None,
        y_max: Optional[int] = None,
    ) -> None: ...
    
    def left_click(
        self,
        duration_ms: int,
        duration_ms_max: Optional[int] = None,
    ) -> None: ...
    
    def left_down(self) -> None: ...
    
    def left_up(self) -> None: ...
    
    def right_click(
        self,
        duration_ms: int,
        duration_ms_max: Optional[int] = None,
    ) -> None: ...
    
    def right_down(self) -> None: ...
    
    def right_up(self) -> None: ...
    
    def key_press(
        self,
        key: int,
        duration_ms: int,
        duration_ms_max: Optional[int] = None,
    ) -> None: ...
    
    def key_down(self, key: int) -> None: ...
    
    def key_up(self, key: int) -> None: ...
    
    def screenshot(self, screen_name: Optional[str] = None) -> np.ndarray: ...
