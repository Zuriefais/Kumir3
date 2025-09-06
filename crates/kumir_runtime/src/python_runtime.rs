use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crate::{Lang, Runtime, RuntimeRequirements};
use log::{error, info};
use rustpython::vm::{PyResult, VirtualMachine};
use rustpython_vm::{
    AsObject,
    builtins::{PyBaseException, PyStr},
    compiler,
    object::{PyObjectRef, PyRef},
};

pub struct PythonRuntime {
    requirements: RuntimeRequirements,
    interpreter: rustpython_vm::Interpreter,
    code: String,
    kill_flag: Arc<AtomicBool>,
}

fn parse_rustpython_error(err: PyRef<PyBaseException>) -> String {
    let error_text = match err.get_arg(0) {
        Some(arg) => match arg.downcast::<PyStr>() {
            Ok(msg) => msg.to_string(),

            Err(_) => "Unknown error".to_string(),
        },

        None => String::from("Empty error message"),
    };

    let traceback = format!(
        "Traceback:\n\tLine number: {lineno}\n\tToken: {lasti}",
        lineno = err.traceback().unwrap().lineno,
        lasti = err.traceback().unwrap().lasti
    );

    format!("{error_text}\n{traceback}")
}

macro_rules! new_function {
    ($vm:expr, $requirements:expr, $name:ident) => {
        $vm.new_function(
            stringify!($name),
            move |vm: &VirtualMachine| -> PyResult<PyObjectRef> {
                match $requirements.$name() {
                    Err(arg) => Err(vm.new_runtime_error(arg)),
                    Ok(arg) => match arg {
                        Some(result) => Ok(vm.new_pyobj(result)),
                        None => Ok(vm.new_pyobj(vm.ctx.none())),
                    },
                }
            },
        )
    };
}

macro_rules! register_module {
    ($vm:expr, $scope:expr, $requirements:expr, $module_name:ident, $($name:ident),+) => {
        let module_dict = $vm.ctx.new_dict();

        $(
            let req = $requirements.clone();
            let func = new_function!($vm, req, $name);
            module_dict.set_item(stringify!($name), func.into(), &$vm).unwrap();
        )+

        let module = $vm.new_module(stringify!($module_name), module_dict, None);
        let sys_modules = $vm.sys_module.as_object().get_attr("modules", &$vm).unwrap();

        match sys_modules.set_item(stringify!($module_name), module.into(), &$vm) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Failed to register module {} with functions:",
                    stringify!($module_name),
                );

                $(
                    error!("\t{}", stringify!($name));
                )+

                error!("Error occured: {:?}", err);
            }
        }
    };
}

macro_rules! register_function {
    ($vm:expr, $scope:expr, $requirements:expr, $name:ident) => {
        let func = new_function!($vm, $requirements.clone(), $name);
        match $scope
            .globals
            .set_item(stringify!($name), func.into(), &$vm)
        {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Failed to register function {}: {:?}",
                    stringify!($name),
                    err
                );
            }
        }
    };
}

impl Runtime for PythonRuntime {
    fn init(
        requirements: RuntimeRequirements,
        _: Lang,
        code: String,
        kill_flag: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        info!("Initializing Python runtime");
        let kill_flag_clone = kill_flag.clone();
        let interpreter = rustpython::InterpreterConfig::new()
            .init_stdlib()
            .init_hook(Box::new(move |vm: &mut VirtualMachine| {
                let (signal_tx, signal_rx) = rustpython_vm::signal::user_signal_channel();
                let kill_flag = kill_flag_clone;
                wasm_thread::spawn(move || {
                    loop {
                        if kill_flag.load(Ordering::Relaxed) {
                            kill_flag.store(false, Ordering::Relaxed);
                            let _ =
                                signal_tx.send(Box::new(|vm: &VirtualMachine| -> PyResult<()> {
                                    Err(vm.new_exception_empty(
                                        vm.ctx.exceptions.keyboard_interrupt.to_owned(),
                                    ))
                                }));

                            break;
                        }

                        std::thread::sleep(Duration::from_millis(50));
                    }
                });
                vm.set_user_signal_channel(signal_rx);
            }))
            .interpreter();

        Ok(Self {
            requirements,
            interpreter,
            code,
            kill_flag,
        })
    }

    fn run(&mut self) -> Result<(), String> {
        self.interpreter.enter(|vm: &VirtualMachine| {
            let scope = vm.new_scope_with_builtins();

            register_module!(
                vm,
                &scope,
                self.requirements.clone(),
                robot,
                move_up,
                move_down,
                move_left,
                move_right,
                paint,
                free_right,
                free_left,
                free_above,
                free_below,
                wall_left,
                wall_right,
                wall_above,
                wall_below,
                colored,
                not_colored
            );

            let source = self.code.as_str();
            let code_obj = vm
                .compile(source, compiler::Mode::Exec, "<embedded>".to_owned())
                .map_err(|err| format!("{:?}", vm.new_syntax_error(&err, Some(source))))?;

            vm.run_code_obj(code_obj, scope).map_err(|err| {
                let mut output = String::new();

                match vm.write_exception_inner(&mut output, &err) {
                    Ok(_) => return output,
                    Err(args) => {
                        error!("Failed to use ready parser: {:?}", args);
                        return parse_rustpython_error(err);
                    }
                };
            })?;

            Ok(())
        })
    }
}
