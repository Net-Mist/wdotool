use wayland_client;
use wayland_client::protocol::*;

pub mod __interfaces {
    use wayland_client::protocol::__interfaces::*;
    wayland_scanner::generate_interfaces!("./protocols/virtual-keyboard-unstable-v1.xml");
}
use self::__interfaces::*;

wayland_scanner::generate_client_code!("./protocols/virtual-keyboard-unstable-v1.xml");
