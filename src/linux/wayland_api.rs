//! Wayland API implementation for Linux platform.
//!
//! The wayland API builds around the dbus protocol, to grab the screenshot.
//!
//! ## Resources
//!
//! - https://forum.qt.io/topic/136838/how-to-take-screenshot-via-dbus-org-freedesktop-portal
use crate::*;

use wayland_client::{protocol::wl_registry, Connection, Dispatch, QueueHandle};

pub struct WaylandApi {
    // connection: Connection,
    // display: WlDisplay,
}

// Implement `Dispatch<WlRegistry, ()> for our state. This provides the logic
// to be able to process events for the wl_registry interface.
//
// The second type parameter is the user-data of our implementation. It is a
// mechanism that allows you to associate a value to each particular Wayland
// object, and allow different dispatching logic depending on the type of the
// associated value.
//
// In this example, we just use () as we don't have any value to associate. See
// the `Dispatch` documentation for more details about this.
impl Dispatch<wl_registry::WlRegistry, ()> for WaylandApi {
    fn event(
        _state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // When receiving events from the wl_registry, we are only interested in the
        // `global` event, which signals a new available global.
        // When receiving this event, we just print its characteristics in this example.
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} (v{})", name, interface, version);
        }
    }
}

impl WaylandApi {
    pub fn new() -> Result<Self> {
        // let connection = Connection::connect_to_env().unwrap();
        // let display = connection.display();
        //
        // let mut event_queue = connection.new_event_queue();
        // let qhandle = event_queue.handle();
        //
        // display.get_registry(&qhandle, ());
        //

        Ok(Self {})
    }
}

impl PlatformApi for WaylandApi {
    fn calibrate(&mut self, window_id: WindowId) -> Result<()> {
        unimplemented!()
    }
    fn window_list(&self) -> Result<WindowList> {
        unimplemented!()
    }
    fn capture_window_screenshot(&self, window_id: WindowId) -> Result<ImageOnHeap> {
        unimplemented!()
    }
    fn get_active_window(&self) -> Result<WindowId> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wayland_client::{protocol::wl_registry, Connection, Dispatch, QueueHandle};

    #[test]
    fn test_wayland_api() {
        let connection = Connection::connect_to_env().unwrap();
        let display = connection.display();

        // Create an event queue for our event processing
        let mut event_queue = connection.new_event_queue();
        // And get its handle to associate new objects to it
        let qh = event_queue.handle();

        // Create a wl_registry object by sending the wl_display.get_registry request.
        // This method takes two arguments: a handle to the queue that the newly created
        // wl_registry will be assigned to, and the user-data that should be associated
        // with this registry (here it is () as we don't need user-data).
        let _registry = display.get_registry(&qh, ());

        let mut api = WaylandApi::new().unwrap();

        // At this point everything is ready, and we just need to wait to receive the events
        // from the wl_registry. Our callback will print the advertised globals.
        println!("Advertised globals:");

        // To actually receive the events, we invoke the `roundtrip` method. This method
        // is special and you will generally only invoke it during the setup of your program:
        // it will block until the server has received and processed all the messages you've
        // sent up to now.
        //
        // In our case, that means it'll block until the server has received our
        // wl_display.get_registry request, and as a reaction has sent us a batch of
        // wl_registry.global events.
        //
        // `roundtrip` will then empty the internal buffer of the queue it has been invoked
        // on, and thus invoke our `Dispatch` implementation that prints the list of advertised
        // globals.
        event_queue.roundtrip(&mut api).unwrap();
    }
}
