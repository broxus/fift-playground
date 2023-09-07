use std::io::Write;

use fift::core::Environment;
use wasm_bindgen::prelude::*;

use self::env::{IFileProvider, WasmEnvironment};
use self::util::*;

mod env;
mod util;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &str = r#"
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

#[wasm_bindgen(js_name = "interpret")]
pub fn interpret(
    files: IFileProvider,
    input: String,
    with_stdlib: bool,
) -> Result<ExecutionOutput, JsValue> {
    let mut env_writer = WasmEnvironment::new(files);
    let mut output_writer = OutputWriter::default();

    let stdlib = with_stdlib
        .then(|| env_writer.include(fift_libs::base_lib().name))
        .transpose()
        .handle_error()?;

    let mut context = fift::Context::new(&mut env_writer, &mut output_writer)
        .with_basic_modules()
        .handle_error()?
        .with_limits(fift::core::ExecutionLimits {
            max_steps: Some(10_000_000),
            max_include_depth: Some(100),
        });

    context.add_source_block(fift::core::SourceBlock::new(
        "<stdin>",
        std::io::Cursor::new(input),
    ));

    if let Some(stdlib) = stdlib {
        context.add_source_block(stdlib);
    }

    let res = context.run();

    let mut output = ObjectBuilder::new();
    match res {
        Ok(exit_code) => output = output.set("success", true).set("exitCode", !exit_code),
        Err(e) => {
            output = output.set("success", false).set("stderr", format!("{e:?}"));

            if let Some(pos) = context.input.get_position() {
                let word_start = pos.line[..pos.word_start].chars().count();
                let word_end = word_start + pos.line[pos.word_start..pos.word_end].chars().count();

                output = output.set(
                    "errorPosition",
                    ObjectBuilder::new()
                        .set("depth", pos.offset)
                        .set("blockName", pos.source_block_name)
                        .set("line", pos.line)
                        .set("lineNumber", pos.line_number)
                        .set("wordStart", word_start)
                        .set("wordEnd", word_end)
                        .build(),
                );
            };

            if let Some(next) = context.next.take() {
                let d = &context.dicts.current;
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

    Ok(output
        .set(
            "stdout",
            String::from_utf8(std::mem::take(&mut output_writer.0)).handle_error()?,
        )
        .build()
        .unchecked_into())
}

#[derive(Default)]
pub struct OutputWriter(Vec<u8>);

impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = buf.len();
        self.0.extend_from_slice(buf);
        Ok(len)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.extend_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
