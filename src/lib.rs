#![allow(dead_code)]

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
