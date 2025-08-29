use crate::{Lang, Runtime, RuntimeRequirements};
use log::{error, info};
use rustpython::vm::VirtualMachine;
use rustpython_vm::{AsObject, compiler};

pub struct PythonRuntime {
    requirements: RuntimeRequirements,
    interpreter: rustpython_vm::Interpreter,
    code: String,
}

macro_rules! register_module {
    ($vm:expr, $scope:expr, $requirements:expr, $module_name:ident, $($name:ident),+) => {
        let module_dict = $vm.ctx.new_dict();
        $(
            let req = $requirements.clone();
            let func = $vm.new_function(stringify!($name), move || req.$name());
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

impl Runtime for PythonRuntime {
    fn init(requirements: RuntimeRequirements, _: Lang, code: String) -> Result<Self, String> {
        info!("Initializing Python runtime");
        let interpreter = rustpython::InterpreterConfig::new()
            .init_stdlib()
            .interpreter();

        Ok(Self {
            requirements,
            interpreter,
            code,
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

            vm.run_code_obj(code_obj, scope)
                .map_err(|err| format!("{:?}", err.traceback()))?;

            Ok(())
        })
    }
}
