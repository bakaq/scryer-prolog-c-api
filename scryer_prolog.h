#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum scryer_Error {
  SCRYER_ERROR_SUCCESS,
  SCRYER_ERROR_ERROR,
  SCRYER_ERROR_PANIC,
} scryer_Error;

typedef enum scryer_LeafAnswerKind {
  SCRYER_LEAF_ANSWER_KIND_TRUE,
  SCRYER_LEAF_ANSWER_KIND_FALSE,
  SCRYER_LEAF_ANSWER_KIND_LEAF_ANSWER,
  SCRYER_LEAF_ANSWER_KIND_EXCEPTION,
} scryer_LeafAnswerKind;

typedef enum scryer_TermKind {
  SCRYER_TERM_KIND_INTEGER,
  SCRYER_TERM_KIND_RATIONAL,
  SCRYER_TERM_KIND_FLOAT,
  SCRYER_TERM_KIND_ATOM,
  SCRYER_TERM_KIND_STRING,
  SCRYER_TERM_KIND_LIST,
  SCRYER_TERM_KIND_COMPOUND,
  SCRYER_TERM_KIND_VARIABLE,
} scryer_TermKind;

typedef struct scryer_Bindings scryer_Bindings;

typedef struct scryer_LeafAnswer scryer_LeafAnswer;

typedef struct scryer_Machine scryer_Machine;

typedef struct scryer_MachineBuilder scryer_MachineBuilder;

typedef struct scryer_QueryState scryer_QueryState;

typedef struct scryer_Term scryer_Term;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct scryer_MachineBuilder *scryer_machine_builder_new(void);

void scryer_machine_builder_drop(struct scryer_MachineBuilder *machine_builder);

struct scryer_Machine *scryer_machine_builder_build(struct scryer_MachineBuilder *machine_builder);

void scryer_machine_drop(struct scryer_Machine *machine);

enum scryer_Error scryer_machine_run_query(struct scryer_Machine *machine,
                                           const char *query,
                                           struct scryer_QueryState **query_state);

enum scryer_Error scryer_machine_consult_module_string(struct scryer_Machine *machine,
                                                       const char *module,
                                                       const char *program);

void scryer_query_state_drop(struct scryer_QueryState *query_state);

enum scryer_Error scryer_query_state_next_answer(struct scryer_QueryState *query_state,
                                                 struct scryer_LeafAnswer **leaf_answer);

void scryer_leaf_answer_drop(struct scryer_LeafAnswer *leaf_answer);

enum scryer_LeafAnswerKind scryer_leaf_answer_kind(const struct scryer_LeafAnswer *leaf_answer);

enum scryer_Error scryer_leaf_answer_unwrap_exception(const struct scryer_LeafAnswer *leaf_answer,
                                                      struct scryer_Term **term);

enum scryer_Error scryer_leaf_answer_unwrap_bindings(const struct scryer_LeafAnswer *leaf_answer,
                                                     struct scryer_Bindings **bindings);

void scryer_bindings_drop(struct scryer_Bindings *bindings);

enum scryer_Error scryer_bindings_get(const struct scryer_Bindings *bindings,
                                      const char *variable,
                                      struct scryer_Term **term);

void scryer_term_drop(struct scryer_Term *term);

enum scryer_TermKind scryer_term_kind(const struct scryer_Term *term);

enum scryer_Error scryer_term_unwrap_integer(const struct scryer_Term *term, char **big_integer);

enum scryer_Error scryer_term_unwrap_float(const struct scryer_Term *term, double *scryer_float);

enum scryer_Error scryer_term_unwrap_rational(const struct scryer_Term *term,
                                              char **numerator,
                                              char **denominator);

enum scryer_Error scryer_term_unwrap_atom(const struct scryer_Term *term, char **atom);

enum scryer_Error scryer_term_unwrap_string(const struct scryer_Term *term, char **string);

enum scryer_Error scryer_term_unwrap_list(const struct scryer_Term *term,
                                          struct scryer_Term ***term_list,
                                          uintptr_t *len);

enum scryer_Error scryer_term_unwrap_compound(const struct scryer_Term *term,
                                              char **functor,
                                              struct scryer_Term ***args,
                                              uintptr_t *len);

enum scryer_Error scryer_term_unwrap_variable(const struct scryer_Term *term, char **variable);

void scryer_string_drop(char *string);

void scryer_list_drop(struct scryer_Term **list, uintptr_t len);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
