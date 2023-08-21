use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use self::env::WasmEnvironment;

mod env;

#[wasm_bindgen]
pub struct FiftState {
    #[wasm_bindgen(skip)]
    pub context: fift::Context<'static>,
    #[wasm_bindgen(skip)]
    pub env: EnvHandle,
    #[wasm_bindgen(skip)]
    pub output: OutputHandle,

    #[wasm_bindgen(skip)]
    pub _env_writer: Box<EnvWriter>,
    #[wasm_bindgen(skip)]
    pub _output_writer: Box<OutputWriter>,
}

#[wasm_bindgen]
impl FiftState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<FiftState, JsValue> {
        let mut env_writer = Box::new(EnvWriter::default());
        let env = env_writer.make_handle();

        let mut output_writer = Box::new(OutputWriter::default());
        let output = output_writer.make_handle();

        let env_writer_ref =
            unsafe { std::mem::transmute::<_, &'static mut EnvWriter>(env_writer.as_mut()) };
        let output_writer_ref =
            unsafe { std::mem::transmute::<_, &'static mut OutputWriter>(output_writer.as_mut()) };

        let context = fift::Context::new(env_writer_ref, output_writer_ref)
            .with_basic_modules()
            .handle_error()?;

        Ok(FiftState {
            context,
            env,
            output,
            _env_writer: env_writer,
            _output_writer: output_writer,
        })
    }

    #[wasm_bindgen]
    pub fn run(&mut self, input: String) -> Result<String, JsValue> {
        self.context.add_source_block(fift::core::SourceBlock::new(
            "<stdin>",
            std::io::Cursor::new(input),
        ));
        self.context.run().handle_error()?;

        String::from_utf8(self.output.take_all()).handle_error()
    }
}

#[derive(Default)]
pub struct OutputWriter(Rc<RefCell<Vec<u8>>>);

impl OutputWriter {
    fn make_handle(&self) -> OutputHandle {
        OutputHandle(self.0.clone())
    }
}

impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = buf.len();
        self.0.borrow_mut().write_all(buf).map(|_| len)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.borrow_mut().write_all(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct OutputHandle(Rc<RefCell<Vec<u8>>>);

impl OutputHandle {
    fn take_all(&self) -> Vec<u8> {
        std::mem::take(&mut *self.0.borrow_mut())
    }
}

#[derive(Default)]
pub struct EnvWriter(Rc<RefCell<WasmEnvironment>>);

impl EnvWriter {
    fn make_handle(&self) -> EnvHandle {
        EnvHandle(self.0.clone())
    }
}

impl fift::core::Environment for EnvWriter {
    fn now_ms(&self) -> u64 {
        self.0.borrow().now_ms()
    }

    fn get_env(&self, name: &str) -> Option<String> {
        self.0.borrow().get_env(name)
    }

    fn file_exists(&self, name: &str) -> bool {
        self.0.borrow().file_exists(name)
    }

    fn write_file(&mut self, name: &str, contents: &[u8]) -> std::io::Result<()> {
        self.0.borrow_mut().write_file(name, contents)
    }

    fn read_file(&mut self, name: &str) -> std::io::Result<Vec<u8>> {
        self.0.borrow_mut().read_file(name)
    }

    fn read_file_part(&mut self, name: &str, offset: u64, len: u64) -> std::io::Result<Vec<u8>> {
        self.0.borrow_mut().read_file_part(name, offset, len)
    }

    fn include(&self, name: &str) -> std::io::Result<fift::core::SourceBlock> {
        self.0.borrow().include(name)
    }
}

pub struct EnvHandle(Rc<RefCell<WasmEnvironment>>);

impl EnvHandle {
    fn borrow(&self) -> std::cell::Ref<'_, WasmEnvironment> {
        self.0.borrow()
    }

    fn borrow_mut(&self) -> std::cell::RefMut<'_, WasmEnvironment> {
        self.0.borrow_mut()
    }
}

impl<T, E> HandleError for Result<T, E>
where
    E: ToString,
{
    type Output = T;

    fn handle_error(self) -> Result<Self::Output, JsValue> {
        self.map_err(|e| js_sys::Error::new(&e.to_string()).into())
    }
}

pub trait HandleError {
    type Output;

    fn handle_error(self) -> Result<Self::Output, JsValue>;
}
