use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use fift::core::Environment;
use wasm_bindgen::prelude::*;

use self::env::WasmEnvironment;
use self::util::*;

mod env;
mod util;

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
    pub fn run(&mut self, input: String, with_stdlib: bool) -> Result<ExecutionOutput, JsValue> {
        self.context.add_source_block(fift::core::SourceBlock::new(
            "<stdin>",
            std::io::Cursor::new(input),
        ));

        if with_stdlib {
            self.context
                .add_source_block(self.env.borrow().include("Fift.fif").handle_error()?);
        }

        let res = self.context.run();

        let mut output = ObjectBuilder::new().set(
            "stdout",
            String::from_utf8(self.output.take_all()).handle_error()?,
        );

        match res {
            Ok(exit_code) => output = output.set("success", true).set("exitCode", !exit_code),
            Err(e) => {
                output = output.set("success", false).set("stderr", format!("{e:?}"));

                if let Some(pos) = self.context.input.get_position() {
                    output = output.set(
                        "errorPosition",
                        ObjectBuilder::new()
                            .set("depth", pos.offset)
                            .set("blockName", pos.source_block_name)
                            .set("line", pos.line)
                            .set("lineNumber", pos.line_number)
                            .set("wordStart", pos.word_start)
                            .set("wordEnd", pos.word_end)
                            .build(),
                    );
                };

                if let Some(next) = self.context.next.take() {
                    let d = &self.context.dicts.current;
                    let mut buffer = String::new();

                    let backtrace: js_sys::Array = js_sys::Array::new();
                    let mut cont = &next;
                    loop {
                        buffer.clear();
                        std::fmt::write(&mut buffer, format_args!("{}", cont.display_dump(d)))
                            .expect("Should not fail");
                        backtrace.push(&JsValue::from_str(&buffer));

                        let Some(next) = cont.up() else {
                            break;
                        };
                        cont = next;
                    }

                    output = output.set("backtrace", backtrace);
                }
            }
        }

        Ok(output.build().unchecked_into())
    }
}

#[wasm_bindgen(typescript_custom_section)]
const EXECUTION_OUTPUT: &str = r#"
export type ExecutionOutput = { stdout: string } & (
    | {
        success: true;
        exitCode: number;
    }
    | {
        success: false;
        stderr: string;
        errorPosition?: {
            depth: number;
            blockName: string;
            line: string;
            lineNumber: number;
            wordStart: number;
            wordEnd: number;
        },
        backtrace?: string[];
    }
);
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ExecutionOutput")]
    pub type ExecutionOutput;
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
}
