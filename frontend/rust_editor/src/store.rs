use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsValue;

pub struct Store {
    local_storage: web_sys::Storage,
    name: String,
}

impl Store {
    pub fn new(name: &str) -> Option<Store> {
        let window = web_sys::window()?;
        if let Ok(Some(local_storage)) = window.local_storage() {
            let store = Store {
                local_storage,
                name: String::from(name),
            };

            Some(store)
        } else {
            None
        }
    }

    /// Read
    pub fn fetch_local_storage<T: DeserializeOwned>(&self) -> Option<T> {
        // If we have an existing cached value, return early.
        if let Ok(Some(value)) = self.local_storage.get_item(&self.name) {
            match serde_json::from_str::<T>(&value) {
                Ok(value) => return Some(value),
                Err(_) => {}
            }
        }

        None
    }

    /// Write
    pub fn sync_local_storage<T: Serialize>(&self, data: &T) -> Result<(), JsValue> {
        match serde_json::to_string(&data) {
            Ok(s) => {
                return self.local_storage.set_item(&self.name, &s);
            }
            Err(_) => {}
        }

        Ok(())
    }
}
