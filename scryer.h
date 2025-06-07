#ifndef SCRYER_H
#define SCRYER_H

#include <stddef.h>

// === Types ===

typedef enum {
    SCRYER_SUCCESS,
    SCRYER_ERROR,
} scryer_Error;

typedef enum {
    SCRYER_TRUE,
    SCRYER_FALSE,
    SCRYER_LEAF_ANSWER,
    SCRYER_EXCEPTION,
} scryer_LeafAnswerKind;

typedef enum {
    SCRYER_INTEGER,
    SCRYER_RATIONAL,
    SCRYER_FLOAT,
    SCRYER_ATOM,
    SCRYER_STRING,
    SCRYER_LIST,
    SCRYER_COMPOUND,
    SCRYER_VARIABLE,
} scryer_TermKind;

typedef struct scryer_MachineBuilder scryer_MachineBuilder;
typedef struct scryer_Machine scryer_Machine;
typedef struct scryer_QueryState scryer_QueryState;

typedef struct scryer_LeafAnswer scryer_LeafAnswer;
typedef struct scryer_Bindings scryer_Bindings;
typedef struct scryer_Term scryer_Term;

// === MachineBuilder methods ===
scryer_MachineBuilder *scryer_machine_builder_new();
void scryer_machine_builder_drop(scryer_MachineBuilder *machine_builder);
scryer_Machine *
scryer_machine_builder_build(scryer_MachineBuilder *machine_builder);

// For now this is kind of useless, but it leaves space to add more methods to
// configure the Machine in the future.

// === Machine methods ===
void scryer_machine_drop(scryer_Machine *machine);
scryer_Error scryer_machine_run_query(
    scryer_Machine *machine, char *query, scryer_QueryState **query_state
);
scryer_Error scryer_machine_consult_module_string(
    scryer_Machine *machine, char *module, char *program
);

// === QueryState methods ===
void scryer_query_state_drop(scryer_QueryState *query_state);
scryer_Error scryer_query_state_next_answer(
    scryer_QueryState *query_state, scryer_LeafAnswer *leaf_answer
);

// === LeafAnswer methods ===
void scryer_leaf_answer_drop(scryer_LeafAnswer *leaf_answer);
scryer_LeafAnswerKind scryer_leaf_answer_kind(scryer_LeafAnswer *leaf_answer);
scryer_Error scryer_leaf_answer_unwrap_exception(
    scryer_LeafAnswer *leaf_answer, scryer_Term **term
);
scryer_Error scryer_leaf_answer_unwrap_bindings(
    scryer_LeafAnswer *leaf_answer, scryer_Bindings **bindings
);

// === Bindings methods ===
void scryer_bindings_drop(scryer_Bindings *bindings);
// TODO: This will be basically a reimplementation of the interface of a
// HashMap<String, Term>

// === Term methods ===
void scryer_term_drop(scryer_Term *term);
scryer_TermKind scryer_term_kind(scryer_Term *term);
scryer_Error scryer_term_unwrap_integer(scryer_Term *term, char **big_integer);
scryer_Error scryer_term_unwrap_float(scryer_Term *term, double *scryer_float);
scryer_Error scryer_term_unwrap_rational(
    scryer_Term *term, char **numerator, char **denominator
);
scryer_Error scryer_term_unwrap_atom(scryer_Term *term, char **atom);
scryer_Error scryer_term_unwrap_string(scryer_Term *term, char **string);
scryer_Error scryer_term_unwrap_list(
    scryer_Term *term, scryer_Term **term_list, size_t *len
);
scryer_Error scryer_term_unwrap_compound(
    scryer_Term *term, char **functor, scryer_Term **args, size_t *len
);
scryer_Error scryer_term_unwrap_variable(scryer_Term *term, char **variable);

#endif
