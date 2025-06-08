#![allow(dead_code)]

use std::ffi::{CStr, CString, c_char, c_double};

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

// === Term methods ===

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_drop(term: Box<Term>) {
    drop(term)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_kind(term: &Term) -> TermKind {
    match term.0 {
        scryer_prolog::Term::Integer(_) => TermKind::Integer,
        scryer_prolog::Term::Rational(_) => TermKind::Rational,
        scryer_prolog::Term::Float(_) => TermKind::Float,
        scryer_prolog::Term::Atom(_) => TermKind::Atom,
        scryer_prolog::Term::String(_) => TermKind::String,
        scryer_prolog::Term::List(_) => TermKind::List,
        scryer_prolog::Term::Compound(_, _) => TermKind::Compound,
        scryer_prolog::Term::Var(_) => TermKind::Variable,
        _ => unreachable!(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_unwrap_integer(
    term: &Term,
    big_integer: *mut *mut c_char,
) -> Error {
    let (error, big_integer_ptr) = if let scryer_prolog::Term::Integer(big_int) = &term.0 {
        (
            Error::Success,
            CString::new(big_int.to_string()).unwrap().into_raw(),
        )
    } else {
        (Error::Error, std::ptr::null_mut())
    };

    unsafe { *big_integer = big_integer_ptr };

    error
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_unwrap_float(
    term: &Term,
    scryer_float: *mut c_double,
) -> Error {
    let (error, scryer_float_val) = if let scryer_prolog::Term::Float(float) = &term.0 {
        (Error::Success, *float)
    } else {
        (Error::Error, 0.0)
    };

    unsafe { *scryer_float = scryer_float_val };

    error
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_unwrap_rational(
    term: &Term,
    numerator: *mut *mut c_char,
    denominator: *mut *mut c_char,
) -> Error {
    let (error, numerator_ptr, denominator_ptr) =
        if let scryer_prolog::Term::Rational(rational) = &term.0 {
            let (num, den) = rational.clone().into_parts();
            (
                Error::Success,
                CString::new(num.to_string()).unwrap().into_raw(),
                CString::new(den.to_string()).unwrap().into_raw(),
            )
        } else {
            (Error::Error, std::ptr::null_mut(), std::ptr::null_mut())
        };

    unsafe { *numerator = numerator_ptr };
    unsafe { *denominator = denominator_ptr };

    error
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_unwrap_atom(term: &Term, atom: *mut *mut c_char) -> Error {
    let (error, atom_ptr) = if let scryer_prolog::Term::Atom(a) = &term.0 {
        (Error::Success, CString::new(a.clone()).unwrap().into_raw())
    } else {
        (Error::Error, std::ptr::null_mut())
    };

    unsafe { *atom = atom_ptr };

    error
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_unwrap_string(term: &Term, string: *mut *mut c_char) -> Error {
    let (error, string_ptr) = if let scryer_prolog::Term::String(s) = &term.0 {
        (Error::Success, CString::new(s.clone()).unwrap().into_raw())
    } else {
        (Error::Error, std::ptr::null_mut())
    };

    unsafe { *string = string_ptr };

    error
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_unwrap_list(
    term: &Term,
    term_list: *mut *mut *mut Term,
    len: *mut usize,
) -> Error {
    let (error, term_list_ptr, term_list_len) = if let scryer_prolog::Term::List(l) = &term.0 {
        let mut term_list_vec: Vec<*mut Term> = l
            .iter()
            .map(|t| Box::into_raw(Box::new(Term(t.clone()))))
            .collect();

        term_list_vec.shrink_to_fit();
        assert_eq!(term_list_vec.len(), term_list_vec.capacity());

        let list_ptr = term_list_vec.as_mut_ptr();
        let len = term_list_vec.len();

        std::mem::forget(term_list_vec);

        (Error::Success, list_ptr, len)
    } else {
        (Error::Error, std::ptr::null_mut(), 0)
    };

    unsafe { *term_list = term_list_ptr };
    unsafe { *len = term_list_len };

    error
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_unwrap_compound(
    term: &Term,
    functor: *mut *mut c_char,
    args: *mut *mut *mut Term,
    len: *mut usize,
) -> Error {
    let (error, functor_ptr, args_ptr, args_len) =
        if let scryer_prolog::Term::Compound(f, args) = &term.0 {
            let functor_ptr = CString::new(f.clone()).unwrap().into_raw();

            let mut args_vec: Vec<*mut Term> = args
                .iter()
                .map(|t| Box::into_raw(Box::new(Term(t.clone()))))
                .collect();

            args_vec.shrink_to_fit();
            assert_eq!(args_vec.len(), args_vec.capacity());

            let list_ptr = args_vec.as_mut_ptr();
            let len = args_vec.len();

            std::mem::forget(args_vec);

            (Error::Success, functor_ptr, list_ptr, len)
        } else {
            (Error::Error, std::ptr::null_mut(), std::ptr::null_mut(), 0)
        };

    unsafe { *functor = functor_ptr };
    unsafe { *args = args_ptr };
    unsafe { *len = args_len };

    error
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_unwrap_variable(
    term: &Term,
    variable: *mut *mut c_char,
) -> Error {
    let (error, variable_ptr) = if let scryer_prolog::Term::Var(v) = &term.0 {
        (Error::Success, CString::new(v.clone()).unwrap().into_raw())
    } else {
        (Error::Error, std::ptr::null_mut())
    };

    unsafe { *variable = variable_ptr };

    error
}

// === Memory management ===

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_string_drop(string: *mut c_char) {
    drop(unsafe { CString::from_raw(string) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_list_drop(list: *mut *mut Term, len: usize) {
    drop(unsafe { Vec::from_raw_parts(list, len, len) })
}
