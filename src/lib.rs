#![allow(dead_code)]

use std::ffi::{CStr, CString, c_char};

#[repr(C)]
pub enum Error {
    Success,
    Error,
    Panic,
}

#[repr(C)]
pub enum LeafAnswerKind {
    True,
    False,
    LeafAnswer,
    Exception,
}

#[repr(C)]
pub enum TermKind {
    Integer,
    Rational,
    Float,
    Atom,
    String,
    List,
    Compound,
    Variable,
}

pub struct MachineBuilder(scryer_prolog::MachineBuilder);
pub struct Machine(scryer_prolog::Machine);
pub struct QueryState<'a>(scryer_prolog::QueryState<'a>);

pub struct LeafAnswer(scryer_prolog::LeafAnswer);
pub struct Bindings {}
pub struct Term(scryer_prolog::Term);

// === MachineBuilder methods ===

#[unsafe(no_mangle)]
pub extern "C" fn scryer_machine_builder_new() -> Box<MachineBuilder> {
    Box::new(MachineBuilder(scryer_prolog::MachineBuilder::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn scryer_machine_builder_drop(machine_builder: Box<MachineBuilder>) {
    drop(machine_builder)
}

#[unsafe(no_mangle)]
pub extern "C" fn scryer_machine_builder_build(
    machine_builder: Box<MachineBuilder>,
) -> Box<Machine> {
    Box::new(Machine(machine_builder.0.build()))
}

// === Machine methods ===

#[unsafe(no_mangle)]
pub extern "C" fn scryer_machine_drop(machine: Box<Machine>) {
    drop(machine)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_machine_run_query<'a>(
    machine: &'a mut Machine,
    query: *const c_char,
    query_state: &mut *mut QueryState<'a>,
) -> Error {
    let query = unsafe { CStr::from_ptr(query) }.to_str().unwrap();

    let query_state_box = Box::new(QueryState(machine.0.run_query(query)));
    *query_state = Box::into_raw(query_state_box);
    Error::Success
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_machine_consult_module_string(
    machine: &mut Machine,
    module: *const c_char,
    program: *const c_char,
) -> Error {
    let module = unsafe { CStr::from_ptr(module) }.to_str().unwrap();
    let program = unsafe { CStr::from_ptr(program) }.to_str().unwrap();

    machine.0.consult_module_string(module, program);
    Error::Success
}

// === QueryState methods ===

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_query_state_drop(query_state: Box<QueryState>) {
    drop(query_state)
}
