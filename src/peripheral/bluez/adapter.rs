use dbus::{stdintf::org_freedesktop_dbus::ObjectManager, MessageItem, Path, Props};
use futures::{future::poll_fn, prelude::*, Async};
use std::sync::Arc;

use super::{
    connection::Connection,
    constants::{ADAPTER_IFACE, BLUEZ_SERVICE_NAME, LE_ADVERTISING_MANAGER_IFACE},
};
use crate::Error;

#[derive(Debug, Clone)]
pub struct Adapter {
    pub object_path: Path<'static>,
    connection: Arc<Connection>,
}

impl Adapter {
    fn find_adapter(connection: &Arc<Connection>) -> Result<Path<'static>, dbus::Error> {
        let connection_path = connection.fallback.with_path(BLUEZ_SERVICE_NAME, "/", 5000);
        let managed_objects = connection_path.get_managed_objects()?;
        for (path, props) in managed_objects.iter() {
            if props.contains_key(LE_ADVERTISING_MANAGER_IFACE) {
                return Ok(path.clone());
            }
        }

        panic!("LEAdvertisingManager1 interface not found");
    }

    pub fn new(connection: Arc<Connection>) -> Result<Self, dbus::Error> {
        let object_path = Adapter::find_adapter(&connection)?;
        Ok(Adapter {
            object_path,
            connection,
        })
    }

    pub fn powered_on(self: &Self, on: bool) -> Result<(), dbus::Error> {
        Props::new(
            &self.connection.fallback,
            BLUEZ_SERVICE_NAME,
            &self.object_path,
            ADAPTER_IFACE,
            1000,
        )
        .set("Powered", MessageItem::Bool(on))
    }

    pub fn is_powered_on(self: &Self) -> Box<impl Future<Item = (), Error = Error>> {
        let connection = self.connection.clone();
        let object_path = self.object_path.clone();

        let powered_on = poll_fn(move || -> Result<Async<()>, Error> {
            let props = Props::new(
                &connection.fallback,
                BLUEZ_SERVICE_NAME,
                &object_path,
                ADAPTER_IFACE,
                1000,
            );

            let message_item = props.get("Powered")?;
            let powered = message_item.inner::<bool>();
            if powered.is_ok() && powered.unwrap() {
                Ok(Async::Ready(()))
            } else {
                Ok(Async::NotReady)
            }
        });

        Box::new(powered_on)
    }
}