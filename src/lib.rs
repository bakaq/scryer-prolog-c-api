#![allow(dead_code)]

use std::ffi::{CStr, CString, c_char, c_double};

#[repr(C)]
pub enum Error {
    Success,
    Error,
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

/// A builder for a [`Machine`].
pub struct MachineBuilder(scryer_prolog::MachineBuilder);

/// A Scryer Prolog instance.
pub struct Machine(scryer_prolog::Machine);

/// A handler for an in-progress query.
///
/// It's parent [`Machine`] shouldn't be accessed while this isn't dropped.
pub struct QueryState<'a>(scryer_prolog::QueryState<'a>);

enum LeafAnswerInner {
    Success(scryer_prolog::LeafAnswer),
    Error(scryer_prolog::Term),
}

/// A leaf answer.
pub struct LeafAnswer(LeafAnswerInner);

/// A dictionary of bindings in a leaf answer.
pub struct Bindings(std::collections::BTreeMap<String, scryer_prolog::Term>);

/// A Prolog Term.
pub struct Term(scryer_prolog::Term);

// === MachineBuilder methods ===

/// Creates a [`MachineBuilder`] with the default options.
#[unsafe(no_mangle)]
pub extern "C" fn scryer_machine_builder_new() -> Box<MachineBuilder> {
    Box::new(MachineBuilder(scryer_prolog::MachineBuilder::new()))
}

/// Drops a [`MachineBuilder`].
///
/// Notice that this shouldn't be called if [`scryer_machine_builder_build`] is
/// called, because the [`MachineBuilder`] gets consumed in that case.
///
/// # Safety
///
/// `machine_builder` should point to a [`MachineBuilder`] previously created
/// with [`scryer_machine_builder_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_machine_builder_drop(machine_builder: Box<MachineBuilder>) {
    drop(machine_builder)
}

/// Creates a [`Machine`] from a [`MachineBuilder`].
///
/// This consumes the [`MachineBuilder`], so you shouldn't call
/// [`scryer_machine_builder_drop`] after.
///
/// # Safety
///
/// `machine_builder` should point to a [`MachineBuilder`] previously created
/// with [`scryer_machine_builder_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_machine_builder_build(
    machine_builder: Box<MachineBuilder>,
) -> Box<Machine> {
    Box::new(Machine(machine_builder.0.build()))
}

// === Machine methods ===

/// Drops a [`Machine`].
///
/// # Safety
///
/// `machine` should point to a [`Machine`] previously created with
/// [`scryer_machine_builder_build`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_machine_drop(machine: Box<Machine>) {
    drop(machine)
}

/// Run a query from a string.
///
/// If no error occurs, `query_state` will be updated with a pointer to a
/// [`QueryState`]. This [`Machine`] shoudn't be accessed again until that
/// [`QueryState`] is dropped with `scryer_query_state_drop`.
///
/// # Errors
///
/// Currently this function can't error, but this will probably change.
///
/// # Safety
///
/// - `machine` should point to a [`Machine`] previously created with
/// [`scryer_machine_builder_build`]. - `query` should be a null-terminated
/// UTF-8 encoded string.
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

/// Consults a module from a string.
///
/// # Errors
///
/// Currently this function can't error, but this will probably change.
///
/// # Safety
///
/// - `machine` should point to a [`Machine`] previously created with
/// [`scryer_machine_builder_build`]. - `module` and `program` should both be
/// null-terminated UTF-8 encoded strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_machine_consult_module_string(
    machine: &mut Machine,
    module: *const c_char,
    program: *const c_char,
) -> Error {
    let module = unsafe { CStr::from_ptr(module) }
        .to_str()
        .expect("UTF-8 encoding");
    let program = unsafe { CStr::from_ptr(program) }
        .to_str()
        .expect("UTF-8 encoding");

    machine.0.consult_module_string(module, program);
    Error::Success
}

// === QueryState methods ===

/// Drops a [`QueryState`].
///
/// # Safety
///
/// `query_state` should point to a [`QueryState`] previously created with
/// [`scryer_machine_run_query`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_query_state_drop(query_state: Box<QueryState>) {
    drop(query_state)
}

/// Get the next leaf answer from the query.
///
/// If no error occurs, `leaf_answer` will be updated with a pointer to a
/// [`LeafAnswer`].
///
/// # Errors
///
/// If an error occurs, then `leaf_answer` will be updated with a pointer to
/// a [`LeafAnswer`] that contains the error term. It can be unwrapped with
/// [`scryer_leaf_answer_unwrap_exception`].
///
/// # Safety
///
/// `query_state` should point to a [`QueryState`] previously created with
///   [`scryer_machine_run_query`].
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

/// Drops a [`LeafAnswer`].
///
/// # Safety
///
/// `leaf_answer` should point to a [`LeafAnswer`] previously created with
/// [`scryer_query_state_next_answer`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_leaf_answer_drop(leaf_answer: Box<LeafAnswer>) {
    drop(leaf_answer)
}

/// Gets the kind of the [`LeafAnswer`].
///
/// # Safety
///
/// `leaf_answer` should point to a [`LeafAnswer`] previously created with
/// [`scryer_query_state_next_answer`].
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

/// Unwraps an exception term from a [`LeafAnswer`].
///
/// On success updates `term` with a pointer to a [`Term`].
///
/// # Errors
///
/// If the `LeafAnswer` is not an exception, this returns [`Error::Error`] and
/// updates `term` to a null pointer.
///
/// # Safety
///
/// `leaf_answer` should point to a [`LeafAnswer`] previously created with
/// [`scryer_query_state_next_answer`].
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

/// Unwraps the bindings from a [`LeafAnswer`].
///
/// On success updates `bindings` with a pointer to a [`Bindings`].
///
/// # Errors
///
/// If the `LeafAnswer` is not a leaf answer (aka, it's an exception, true
/// or false), this returns [`Error::Error`] and updates `bindings` to a null
/// pointer.
///
/// # Safety
///
/// `leaf_answer` should point to a [`LeafAnswer`] previously created with
/// [`scryer_query_state_next_answer`].
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

/// Drops a [`Bindings`].
///
/// # Safety
///
/// `bindings` should point to a [`bindings`] previously created with
/// [`scryer_leaf_answer_unwrap_bindings`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_bindings_drop(bindings: Box<Bindings>) {
    drop(bindings)
}

/// Get the term bound to a variable in [`Bindings`].
///
/// If the variable specified by `variable` exists in the bindings, succeeds and
/// updates `term` with a pointer to a [`Term`].
///
/// # Errors
///
/// If the variable specified doesn't exist in the bindings, this returns
/// [`Error::Error`] and updates `term` to a null pointer.
///
/// # Safety
///
/// - `variable` should be a null-terminated UTF-8 encoded string. -
/// `bindings` should point to a [`bindings`] previously created with
/// [`scryer_leaf_answer_unwrap_bindings`].
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

/// Drops a [`Term`].
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_term_drop(term: Box<Term>) {
    drop(term)
}

/// Gets the kind of a [`Term`].
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Unwraps a big integer from a [`Term`].
///
/// If `term` is an integer, succeeds and updates `big_integer` with  a
/// null-terminated string representing that integer. This is so that arbitrary
/// precision can be supported. If you need an actual integer you should parse
/// this string.
///
/// # Errors
///
/// If `term` is not an integer, returns [`Error::Error`] and updates
/// `big_integer` to a null pointer.
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Unwraps a float from a [`Term`].
///
/// If `term` is a float, succeeds and updates `scryer_float` with it's value.
///
/// # Errors
///
/// If `term` is not a float, returns [`Error::Error`] and updates
/// `scryer_float` to `0.0`.
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Unwraps a rational from a [`Term`].
///
/// If `term` is a rational, succeeds and updates `numerator` and
/// `denominator` with a strings representing their values, like in
/// [`scryer_term_unwrap_integer`].
///
/// # Errors
///
/// If `term` is not a rational, returns [`Error::Error`] and updates
/// `numerator` and `denominator` to a null pointers.
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Unwraps an atom from a [`Term`].
///
/// If `term` is an atom, succeeds and updates `atom` with a null terminated
/// string with it's contents.
///
/// # Errors
///
/// If `term` is not an atom, returns [`Error::Error`] and updates `atom` to a
/// null pointer.
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Unwraps a string from a [`Term`].
///
/// If `term` is a string, succeeds and updates `string` with a null terminated
/// string with it's contents.
///
/// # Errors
///
/// If `term` is not a string, returns [`Error::Error`] and updates `string` to
/// a null pointer.
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Unwraps a list from a [`Term`].
///
/// If `term` is a list, succeeds and updates `term_list` with a pointer to a
/// buffer containing pointers to terms, and `len` to the number of terms in
/// that buffer.
///
/// This buffer needs to be dropped with `scryer_list_drop`.
///
/// # Errors
///
/// If `term` is not a list, returns [`Error::Error`], updates `term_list` to a
/// null pointer and `len` to 0.
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Unwraps a compound from a [`Term`].
///
/// If `term` is a compound, succeeds and updates `functor` with a null
/// terminated string with the contents of the functor, `args` with a buffer
/// with containing pointers to terms, and `len` to the number of terms in that
/// buffer.
///
/// `args` needs to be dropped with `scryer_list_drop`.
///
/// # Errors
///
/// If `term` is not a compound, returns [`Error::Error`], updates `functor` and
/// `args` to a null pointers, and `len` to 0.
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Unwraps a variable from a [`Term`].
///
/// If `term` is a variable, succeeds and updates `variable` with a null
/// terminated variable with it's name.
///
/// # Errors
///
/// If `term` is not a variable, returns [`Error::Error`] and updates `variable`
/// to a null pointer.
///
/// # Safety
///
/// `term` should point to a [`Term`] previously created by Scryer Prolog.
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

/// Drop a previously allocated string.
///
/// # Safety
///
/// `string` should be a string previously allocated by Scryer Prolog.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_string_drop(string: *mut c_char) {
    drop(unsafe { CString::from_raw(string) })
}

/// Drop a previously allocated list.
///
/// This only frees the memory for the list itself. The elements it contains
/// should be dropped first separatelly.
///
/// # Safety
///
/// `list` should be a list previously created with [`scryer_term_unwrap_list`],
/// and `len` should be it's length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scryer_list_drop(list: *mut *mut Term, len: usize) {
    drop(unsafe { Vec::from_raw_parts(list, len, len) })
}
