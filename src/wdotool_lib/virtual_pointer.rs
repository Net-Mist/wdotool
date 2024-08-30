use wayland_client;
use wayland_client::protocol::*;

pub mod __interfaces {
    use wayland_client::protocol::__interfaces::*;
    wayland_scanner::generate_interfaces!("./protocols/wlr-virtual-pointer-unstable-v1.xml");
}
use self::__interfaces::*;

wayland_scanner::generate_client_code!("./protocols/wlr-virtual-pointer-unstable-v1.xml");
