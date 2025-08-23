use crate::executors::robot::robot_module;
use log::info;
use rustpython_vm as vm;

pub fn run_code_from_string(source: &str) {
    let interp = rustpython::InterpreterConfig::new()
        .init_stdlib()
        .init_hook(Box::new(|vm| {
            vm.add_native_module("robot".to_owned(), Box::new(robot_module::make_module));
        }))
        .interpreter();

    interp.enter(|vm| {
        let scope = vm.new_scope_with_builtins();

        match vm
            .compile(source, vm::compiler::Mode::Exec, "<embedded>".to_owned())
            .map_err(|err| vm.new_syntax_error(&err, Some(source)))
            .and_then(|code_obj| vm.run_code_obj(code_obj, scope))
        {
            Ok(_) => (),
            Err(exception) => vm.print_exception(exception),
        }
    })
}
