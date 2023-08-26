use std::collections::HashMap;

use gloo_utils::errors::JsError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &str = r#"
export interface IFileProvider {
    fileExists: (name: string) => boolean;
    readFile: (name: string) => ArrayBuffer;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IFileProvider")]
    pub type IFileProvider;

    #[wasm_bindgen(method, js_name = "fileExists")]
    pub fn file_exists(this: &IFileProvider, name: &str) -> bool;

    #[wasm_bindgen(method, js_name = "readFile", catch)]
    pub fn read_file(this: &IFileProvider, name: &str) -> Result<Vec<u8>, JsValue>;
}

pub struct WasmEnvironment {
    temp_files: HashMap<String, Vec<u8>>,
    external_files: IFileProvider,
}

impl WasmEnvironment {
    pub fn new(files: IFileProvider) -> Self {
        Self {
            temp_files: Default::default(),
            external_files: files,
        }
    }
}

impl fift::core::Environment for WasmEnvironment {
    fn now_ms(&self) -> u64 {
        now_sec_u64()
    }

    fn get_env(&self, _: &str) -> Option<String> {
        None
    }

    fn file_exists(&self, name: &str) -> bool {
        self.temp_files.contains_key(name)
            || fift_libs::all().contains_key(name)
            || self.external_files.file_exists(name)
    }

    fn write_file(&mut self, name: &str, contents: &[u8]) -> std::io::Result<()> {
        self.temp_files.insert(name.to_owned(), contents.to_owned());
        Ok(())
    }

    fn read_file(&mut self, name: &str) -> std::io::Result<Vec<u8>> {
        if let Some(data) = self.temp_files.get(name) {
            Ok(data.to_vec())
        } else if self.external_files.file_exists(name) {
            self.external_files.read_file(name).map_err(map_js_err)
        } else if let Some(data) = fift_libs::all().get(name) {
            Ok(data.as_bytes().to_vec())
        } else {
            Err(not_found(name))
        }
    }

    fn read_file_part(&mut self, name: &str, offset: u64, len: u64) -> std::io::Result<Vec<u8>> {
        use std::borrow::Cow;

        fn out_of_range() -> std::io::Error {
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Offset out of range")
        }

        fn check_range(data: &[u8], offset: u64, len: u64) -> std::io::Result<()> {
            if offset >= data.len() as u64 || offset + len > data.len() as u64 {
                return Err(out_of_range());
            }
            Ok(())
        }

        let data = if let Some(data) = self.temp_files.get(name) {
            Cow::Borrowed(data.as_slice())
        } else if self.external_files.file_exists(name) {
            self.external_files
                .read_file(name)
                .map(Cow::Owned)
                .map_err(map_js_err)?
        } else if let Some(data) = fift_libs::all().get(name) {
            Cow::Borrowed(data.as_bytes())
        } else {
            return Err(not_found(name));
        };

        check_range(&data, offset, len)?;
        let offset = offset as usize;
        let len = len as usize;
        Ok(data[offset..offset + len].to_vec())
    }

    fn include(&self, name: &str) -> std::io::Result<fift::core::SourceBlock> {
        Ok(if self.external_files.file_exists(name) {
            let data = self.external_files.read_file(name).map_err(map_js_err)?;
            fift::core::SourceBlock::new(name, std::io::Cursor::new(data))
        } else {
            let Some(data) = fift_libs::all().get(name) else {
                return Err(not_found(name));
            };
            fift::core::SourceBlock::new(name, std::io::Cursor::new(*data))
        })
    }
}

fn map_js_err(error: JsValue) -> std::io::Error {
    #[derive(Debug)]
    struct SomeJsError;

    impl std::fmt::Display for SomeJsError {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(self, f)
        }
    }

    impl std::error::Error for SomeJsError {}

    if let Ok(error) = error.dyn_into::<js_sys::Error>() {
        std::io::Error::new(std::io::ErrorKind::Other, JsError::from(error))
    } else {
        std::io::Error::new(std::io::ErrorKind::Other, "JS error")
    }
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
fn now_sec_u64() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
fn now_sec_u64() -> u64 {
    use std::time::SystemTime;

    (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
        .expect("shouldn't fail")
        .as_secs()
}

fn not_found(name: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("`{name}` file not found"),
    )
}
