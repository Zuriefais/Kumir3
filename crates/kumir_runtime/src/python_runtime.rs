use crate::{Lang, Runtime, RuntimeRequirements};
use log::info;
use rustpython::vm::{PyObject, PyResult, TryFromBorrowedObject, VirtualMachine};
use rustpython_vm::{AsObject, compiler, scope::Scope};

pub struct PythonRuntime {
    requirements: RuntimeRequirements,
    interpreter: rustpython_vm::Interpreter,
    code: String,
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

            register_move_up(vm, &scope, self.requirements.clone());
            register_move_down(vm, &scope, self.requirements.clone());
            register_move_right(vm, &scope, self.requirements.clone());
            register_move_left(vm, &scope, self.requirements.clone());
            register_paint(vm, &scope, self.requirements.clone());

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

fn register_move_up(vm: &VirtualMachine, scope: &Scope, requirements: RuntimeRequirements) {
    let move_up = vm.new_function("move_up", move || requirements.move_up());
    scope
        .globals
        .set_item("move_up", move_up.into(), &vm)
        .unwrap();
}

fn register_move_down(vm: &VirtualMachine, scope: &Scope, requirements: RuntimeRequirements) {
    let move_down = vm.new_function("move_down", move || requirements.move_down());
    scope
        .globals
        .set_item("move_down", move_down.into(), &vm)
        .unwrap();
}

fn register_move_right(vm: &VirtualMachine, scope: &Scope, requirements: RuntimeRequirements) {
    let move_right = vm.new_function("move_right", move || requirements.move_right());
    scope
        .globals
        .set_item("move_right", move_right.into(), &vm)
        .unwrap();
}

fn register_move_left(vm: &VirtualMachine, scope: &Scope, requirements: RuntimeRequirements) {
    let move_left = vm.new_function("move_left", move || requirements.move_left());
    scope
        .globals
        .set_item("move_left", move_left.into(), &vm)
        .unwrap();
}

fn register_paint(vm: &VirtualMachine, scope: &Scope, requirements: RuntimeRequirements) {
    let paint = vm.new_function("paint", move || requirements.paint());
    scope.globals.set_item("paint", paint.into(), &vm).unwrap();
}
