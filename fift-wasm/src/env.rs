use std::borrow::Cow;
use std::collections::HashMap;

pub struct WasmEnvironment {
    files: HashMap<String, Cow<'static, [u8]>>,
}

impl Default for WasmEnvironment {
    fn default() -> Self {
        let mut files = HashMap::new();
        for &(name, content) in LIBRARIES {
            files.insert(name.to_owned(), Cow::Borrowed(content));
        }
        Self { files }
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
        self.files.contains_key(name)
    }

    fn write_file(&mut self, name: &str, contents: &[u8]) -> std::io::Result<()> {
        self.files
            .insert(name.to_owned(), Cow::Owned(contents.to_owned()));
        Ok(())
    }

    fn read_file(&mut self, name: &str) -> std::io::Result<Vec<u8>> {
        match self.files.get(name) {
            Some(data) => Ok(data.to_vec()),
            None => Err(not_found(name)),
        }
    }

    fn read_file_part(&mut self, name: &str, offset: u64, len: u64) -> std::io::Result<Vec<u8>> {
        match self.files.get(name) {
            Some(data) => {
                if offset >= data.len() as u64 || offset + len > data.len() as u64 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        format!("Offset out of range"),
                    ));
                }

                let offset = offset as usize;
                let len = len as usize;
                Ok(data[offset..offset + len].to_vec())
            }
            None => Err(not_found(name)),
        }
    }

    fn include(&self, name: &str) -> std::io::Result<fift::core::SourceBlock> {
        let Some(data) = self.files.get(name) else {
            return Err(not_found(name));
        };

        Ok(match data {
            Cow::Owned(data) => {
                fift::core::SourceBlock::new(name, std::io::Cursor::new(data.clone()))
            }
            Cow::Borrowed(data) => fift::core::SourceBlock::new(name, std::io::Cursor::new(*data)),
        })
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

macro_rules! define_libs {
    ($($name:literal => $path:literal),*$(,)?) => {&[
        $(($name, include_bytes!($path))),*
    ]};
}

const LIBRARIES: &'static [(&'static str, &'static [u8])] = define_libs! {
    "Asm.fif" => "lib/Asm.fif",
    "Color.fif" => "lib/Color.fif",
    "Fift.fif" => "lib/Fift.fif",
    "FiftExt.fif" => "lib/FiftExt.fif",
    "Lisp.fif" => "lib/Lisp.fif",
    "Lists.fif" => "lib/Lists.fif",
    "Stack.fif" => "lib/Stack.fif",
    "TonUtil.fif" => "lib/TonUtil.fif",
};
