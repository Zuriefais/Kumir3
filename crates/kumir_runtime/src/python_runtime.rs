use crate::{Lang, Runtime, RuntimeRequirements};
use log::{error, info};
use rustpython::vm::{PyObject, PyResult, TryFromBorrowedObject, VirtualMachine};
use rustpython_vm::{AsObject, compiler, scope::Scope};

pub struct PythonRuntime {
    requirements: RuntimeRequirements,
    interpreter: rustpython_vm::Interpreter,
    code: String,
}

macro_rules! register {
    ($vm:expr, $scope:expr, $requirements:expr, $name:ident) => {
        let req = $requirements.clone();
        let func = $vm.new_function(stringify!($name), move || req.$name());
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

            register!(vm, &scope, self.requirements.clone(), move_up);
            register!(vm, &scope, self.requirements.clone(), move_down);
            register!(vm, &scope, self.requirements.clone(), move_left);
            register!(vm, &scope, self.requirements.clone(), move_right);
            register!(vm, &scope, self.requirements.clone(), paint);
            register!(vm, &scope, self.requirements.clone(), free_right);
            register!(vm, &scope, self.requirements.clone(), free_left);
            register!(vm, &scope, self.requirements.clone(), free_above);
            register!(vm, &scope, self.requirements.clone(), free_below);
            register!(vm, &scope, self.requirements.clone(), wall_left);
            register!(vm, &scope, self.requirements.clone(), wall_right);
            register!(vm, &scope, self.requirements.clone(), wall_above);
            register!(vm, &scope, self.requirements.clone(), wall_below);
            register!(vm, &scope, self.requirements.clone(), colored);

            let source = self.code.as_str();
            let code_obj = vm
                .compile(source, compiler::Mode::Exec, "<embedded>".to_owned())
                .map_err(|err| format!("{:?}", vm.new_syntax_error(&err, Some(source))))?;

            vm.run_code_obj(code_obj, scope)
                .map_err(|err| format!("{:?}", err))?;

            Ok(())
        })
    }
}
