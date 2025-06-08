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

enum LeafAnswerInner {
    Success(scryer_prolog::LeafAnswer),
    Error(scryer_prolog::Term),
}
pub struct LeafAnswer(LeafAnswerInner);
pub struct Bindings(std::collections::BTreeMap<String, scryer_prolog::Term>);
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
    query_state: *mut *mut QueryState<'a>,
) -> Error {
    let query = unsafe { CStr::from_ptr(query) }.to_str().unwrap();

    let query_state_box = Box::new(QueryState(machine.0.run_query(query)));
    let query_state_ptr = Box::into_raw(query_state_box);
    unsafe { *query_state = query_state_ptr };
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_query_state_next_answer(
    query_state: &mut QueryState,
    leaf_answer: *mut *mut LeafAnswer,
) -> Error {
    let (error, leaf_answer_ptr) = query_state
        .0
        .next()
        .map(|l| match l {
            Ok(la) => (
                Error::Success,
                Box::into_raw(Box::new(LeafAnswer(LeafAnswerInner::Success(la)))),
            ),
            Err(error) => (
                Error::Error,
                Box::into_raw(Box::new(LeafAnswer(LeafAnswerInner::Error(error)))),
            ),
        })
        .unwrap_or((Error::Success, std::ptr::null_mut()));

    unsafe { *leaf_answer = leaf_answer_ptr };

    error
}

// === LeafAnswer methods ===

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_leaf_answer_drop(leaf_answer: Box<LeafAnswer>) {
    drop(leaf_answer)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_leaf_answer_kind(leaf_answer: &LeafAnswer) -> LeafAnswerKind {
    match &leaf_answer.0 {
        LeafAnswerInner::Success(inner) => match inner {
            scryer_prolog::LeafAnswer::True => LeafAnswerKind::True,
            scryer_prolog::LeafAnswer::False => LeafAnswerKind::False,
            scryer_prolog::LeafAnswer::LeafAnswer { .. } => LeafAnswerKind::LeafAnswer,
            scryer_prolog::LeafAnswer::Exception(_) => LeafAnswerKind::Exception,
        },
        LeafAnswerInner::Error(_) => LeafAnswerKind::Exception,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_leaf_answer_unwrap_exception(
    leaf_answer: &LeafAnswer,
    term: *mut *mut Term,
) -> Error {
    let (error, term_ptr) = match &leaf_answer.0 {
        LeafAnswerInner::Success(inner) => match inner {
            scryer_prolog::LeafAnswer::Exception(e) => {
                (Error::Success, Box::into_raw(Box::new(Term(e.clone()))))
            }
            _ => (Error::Error, std::ptr::null_mut()),
        },
        LeafAnswerInner::Error(error) => {
            (Error::Success, Box::into_raw(Box::new(Term(error.clone()))))
        }
    };

    unsafe { *term = term_ptr };

    error
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_leaf_answer_unwrap_bindings(
    leaf_answer: &LeafAnswer,
    bindings: *mut *mut Bindings,
) -> Error {
    let (error, bindings_ptr) = match &leaf_answer.0 {
        LeafAnswerInner::Success(inner) => match inner {
            scryer_prolog::LeafAnswer::LeafAnswer { bindings, .. } => (
                Error::Success,
                Box::into_raw(Box::new(Bindings(bindings.clone()))),
            ),
            _ => (Error::Error, std::ptr::null_mut()),
        },
        LeafAnswerInner::Error(_) => (Error::Error, std::ptr::null_mut()),
    };

    unsafe { *bindings = bindings_ptr };

    error
}

// === Bindings methods ===

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_bindings_drop(bindings: Box<Bindings>) {
    drop(bindings)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_bindings_get(
    bindings: &Bindings,
    variable: *const c_char,
    term: *mut *mut Term,
) -> Error {
    let variable_str = unsafe { CStr::from_ptr(variable) }.to_str().unwrap();
    let (error, term_ptr) = bindings
        .0
        .get(variable_str)
        .map(|term| (Error::Success, Box::into_raw(Box::new(Term(term.clone())))))
        .unwrap_or((Error::Error, std::ptr::null_mut()));

    unsafe { *term = term_ptr };

    error
}

// TODO: Iterator?
