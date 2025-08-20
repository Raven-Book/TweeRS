use crate::error::{JSResult, ScriptError};
use std::sync::Once;
use tracing::debug;

static V8_INIT: Once = Once::new();

pub struct ScriptEngine {
    isolate: v8::OwnedIsolate,
}

impl ScriptEngine {
    fn setup_console(scope: &mut v8::HandleScope, global: &v8::Object) {
        let console_key = v8::String::new(scope, "console").unwrap();
        let console_obj = v8::Object::new(scope);

        let log_key = v8::String::new(scope, "log").unwrap();
        let log_fn = v8::Function::new(
            scope,
            |scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             _rv: v8::ReturnValue| {
                let mut output = Vec::new();
                for i in 0..args.length() {
                    if i > 0 {
                        output.push(" ".to_string());
                    }
                    let arg = args.get(i);
                    if let Some(str_arg) = arg.to_string(scope) {
                        output.push(str_arg.to_rust_string_lossy(scope));
                    } else {
                        output.push("[object]".to_string());
                    }
                }
                println!("[Script Console] {}", output.join(""));
            },
        )
        .unwrap();

        console_obj.set(scope, log_key.into(), log_fn.into());
        global.set(scope, console_key.into(), console_obj.into());
    }

    pub fn new() -> JSResult<Self> {
        V8_INIT.call_once(|| {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        let isolate = v8::Isolate::new(v8::CreateParams::default());

        Ok(Self { isolate })
    }

    pub fn execute_data_processor(
        &mut self,
        data_json: &str,
        format_json: &str,
        script: &str,
    ) -> JSResult<String> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Context::new(scope, v8::ContextOptions::default());
        let scope = &mut v8::ContextScope::new(scope, context);

        let input_key = v8::String::new(scope, "input").unwrap();
        let input_value = v8::String::new(scope, data_json).unwrap();
        let input_obj = v8::json::parse(scope, input_value)
            .ok_or_else(|| ScriptError::InvalidOutput("Failed to parse input JSON".to_string()))?;

        let global = context.global(scope);
        global.set(scope, input_key.into(), input_obj);

        let format_key = v8::String::new(scope, "format").unwrap();
        let format_value = v8::String::new(scope, format_json).unwrap();
        let format_obj = v8::json::parse(scope, format_value)
            .ok_or_else(|| ScriptError::InvalidOutput("Failed to parse format JSON".to_string()))?;
        global.set(scope, format_key.into(), format_obj);

        Self::setup_console(scope, &global);

        let wrapped_script = format!("(function() {{ {script} }})()");
        let code = v8::String::new(scope, &wrapped_script).unwrap();
        let script_obj = v8::Script::compile(scope, code, None)
            .ok_or_else(|| ScriptError::CompilationError("Failed to compile script".to_string()))?;

        let result = script_obj
            .run(scope)
            .ok_or_else(|| ScriptError::ExecutionError("Script execution failed".to_string()))?;
        let result_json = v8::json::stringify(scope, result)
            .ok_or_else(|| ScriptError::InvalidOutput("Failed to stringify result".to_string()))?;

        Ok(result_json.to_rust_string_lossy(scope))
    }

    pub fn execute_html_processor(
        &mut self,
        html: &str,
        passages_json: &str,
        format_json: &str,
        script: &str,
    ) -> JSResult<String> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Context::new(scope, v8::ContextOptions::default());
        let scope = &mut v8::ContextScope::new(scope, context);

        let global = context.global(scope);
        Self::setup_console(scope, &global);
        let input_key = v8::String::new(scope, "input").unwrap();
        let input_value = v8::String::new(scope, html).unwrap();
        global.set(scope, input_key.into(), input_value.into());

        let passages_key = v8::String::new(scope, "passages").unwrap();
        let passages_value = v8::String::new(scope, passages_json).unwrap();
        let passages_obj = v8::json::parse(scope, passages_value).ok_or_else(|| {
            ScriptError::InvalidOutput("Failed to parse passages JSON".to_string())
        })?;
        global.set(scope, passages_key.into(), passages_obj);

        let format_key = v8::String::new(scope, "format").unwrap();
        let format_value = v8::String::new(scope, format_json).unwrap();
        let format_obj = v8::json::parse(scope, format_value)
            .ok_or_else(|| ScriptError::InvalidOutput("Failed to parse format JSON".to_string()))?;
        global.set(scope, format_key.into(), format_obj);

        let wrapped_script = format!("(function() {{ {script} }})()");
        let code = v8::String::new(scope, &wrapped_script).unwrap();
        let script_obj = v8::Script::compile(scope, code, None)
            .ok_or_else(|| ScriptError::CompilationError("Failed to compile script".to_string()))?;

        let result = script_obj
            .run(scope)
            .ok_or_else(|| ScriptError::ExecutionError("Script execution failed".to_string()))?;
        if result.is_string() {
            Ok(result.to_string(scope).unwrap().to_rust_string_lossy(scope))
        } else {
            Err(ScriptError::InvalidOutput(
                "Script must return a string".to_string(),
            ))
        }
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new().expect("Failed to initialize V8 engine")
    }
}

impl Drop for ScriptEngine {
    fn drop(&mut self) {
        debug!("ScriptEngine dropped");
    }
}
