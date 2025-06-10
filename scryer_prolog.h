#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum scryer_Error {
  SCRYER_ERROR_SUCCESS,
  SCRYER_ERROR_ERROR,
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

/**
 * A dictionary of bindings in a leaf answer.
 */
typedef struct scryer_Bindings scryer_Bindings;

/**
 * A leaf answer.
 */
typedef struct scryer_LeafAnswer scryer_LeafAnswer;

/**
 * A Scryer Prolog instance.
 */
typedef struct scryer_Machine scryer_Machine;

/**
 * A builder for a [`Machine`].
 */
typedef struct scryer_MachineBuilder scryer_MachineBuilder;

/**
 * A handler for an in-progress query.
 *
 * It's parent [`Machine`] shouldn't be accessed while this isn't dropped.
 */
typedef struct scryer_QueryState scryer_QueryState;

/**
 * A Prolog Term.
 */
typedef struct scryer_Term scryer_Term;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Creates a [`MachineBuilder`] with the default options.
 */
struct scryer_MachineBuilder *scryer_machine_builder_new(void);

/**
 * Drops a [`MachineBuilder`].
 *
 * Notice that this shouldn't be called if [`scryer_machine_builder_build`] is
 * called, because the [`MachineBuilder`] gets consumed in that case.
 *
 * # Safety
 *
 * `machine_builder` should point to a [`MachineBuilder`] previously created
 * with [`scryer_machine_builder_new`].
 */
void scryer_machine_builder_drop(struct scryer_MachineBuilder *machine_builder);

/**
 * Creates a [`Machine`] from a [`MachineBuilder`].
 *
 * This consumes the [`MachineBuilder`], so you shouldn't call
 * [`scryer_machine_builder_drop`] after.
 *
 * # Safety
 *
 * `machine_builder` should point to a [`MachineBuilder`] previously created
 * with [`scryer_machine_builder_new`].
 */
struct scryer_Machine *scryer_machine_builder_build(struct scryer_MachineBuilder *machine_builder);

/**
 * Drops a [`Machine`].
 *
 * # Safety
 *
 * `machine` should point to a [`Machine`] previously created with
 * [`scryer_machine_builder_build`].
 */
void scryer_machine_drop(struct scryer_Machine *machine);

/**
 * Run a query from a string.
 *
 * If no error occurs, `query_state` will be updated with a pointer to a
 * [`QueryState`]. This [`Machine`] shoudn't be accessed again until that
 * [`QueryState`] is dropped with `scryer_query_state_drop`.
 *
 * # Errors
 *
 * Currently this function can't error, but this will probably change.
 *
 * # Safety
 *
 * - `machine` should point to a [`Machine`] previously created with
 * [`scryer_machine_builder_build`]. - `query` should be a null-terminated
 * UTF-8 encoded string.
 */
enum scryer_Error scryer_machine_run_query(struct scryer_Machine *machine,
                                           const char *query,
                                           struct scryer_QueryState **query_state);

/**
 * Consults a module from a string.
 *
 * # Errors
 *
 * Currently this function can't error, but this will probably change.
 *
 * # Safety
 *
 * - `machine` should point to a [`Machine`] previously created with
 * [`scryer_machine_builder_build`]. - `module` and `program` should both be
 * null-terminated UTF-8 encoded strings.
 */
enum scryer_Error scryer_machine_consult_module_string(struct scryer_Machine *machine,
                                                       const char *module,
                                                       const char *program);

/**
 * Drops a [`QueryState`].
 *
 * # Safety
 *
 * `query_state` should point to a [`QueryState`] previously created with
 * [`scryer_machine_run_query`].
 */
void scryer_query_state_drop(struct scryer_QueryState *query_state);

/**
 * Get the next leaf answer from the query.
 *
 * If no error occurs, `leaf_answer` will be updated with a pointer to a
 * [`LeafAnswer`].
 *
 * # Errors
 *
 * If an error occurs, then `leaf_answer` will be updated with a pointer to
 * a [`LeafAnswer`] that contains the error term. It can be unwrapped with
 * [`scryer_leaf_answer_unwrap_exception`].
 *
 * # Safety
 *
 * `query_state` should point to a [`QueryState`] previously created with
 *   [`scryer_machine_run_query`].
 */
enum scryer_Error scryer_query_state_next_answer(struct scryer_QueryState *query_state,
                                                 struct scryer_LeafAnswer **leaf_answer);

/**
 * Drops a [`LeafAnswer`].
 *
 * # Safety
 *
 * `leaf_answer` should point to a [`LeafAnswer`] previously created with
 * [`scryer_query_state_next_answer`].
 */
void scryer_leaf_answer_drop(struct scryer_LeafAnswer *leaf_answer);

/**
 * Gets the kind of the [`LeafAnswer`].
 *
 * # Safety
 *
 * `leaf_answer` should point to a [`LeafAnswer`] previously created with
 * [`scryer_query_state_next_answer`].
 */
enum scryer_LeafAnswerKind scryer_leaf_answer_kind(const struct scryer_LeafAnswer *leaf_answer);

/**
 * Unwraps an exception term from a [`LeafAnswer`].
 *
 * On success updates `term` with a pointer to a [`Term`].
 *
 * # Errors
 *
 * If the `LeafAnswer` is not an exception, this returns [`Error::Error`] and
 * updates `term` to a null pointer.
 *
 * # Safety
 *
 * `leaf_answer` should point to a [`LeafAnswer`] previously created with
 * [`scryer_query_state_next_answer`].
 */
enum scryer_Error scryer_leaf_answer_unwrap_exception(const struct scryer_LeafAnswer *leaf_answer,
                                                      struct scryer_Term **term);

/**
 * Unwraps the bindings from a [`LeafAnswer`].
 *
 * On success updates `bindings` with a pointer to a [`Bindings`].
 *
 * # Errors
 *
 * If the `LeafAnswer` is not a leaf answer (aka, it's an exception, true
 * or false), this returns [`Error::Error`] and updates `bindings` to a null
 * pointer.
 *
 * # Safety
 *
 * `leaf_answer` should point to a [`LeafAnswer`] previously created with
 * [`scryer_query_state_next_answer`].
 */
enum scryer_Error scryer_leaf_answer_unwrap_bindings(const struct scryer_LeafAnswer *leaf_answer,
                                                     struct scryer_Bindings **bindings);

/**
 * Drops a [`Bindings`].
 *
 * # Safety
 *
 * `bindings` should point to a [`bindings`] previously created with
 * [`scryer_leaf_answer_unwrap_bindings`].
 */
void scryer_bindings_drop(struct scryer_Bindings *bindings);

/**
 * Get the term bound to a variable in [`Bindings`].
 *
 * If the variable specified by `variable` exists in the bindings, succeeds and
 * updates `term` with a pointer to a [`Term`].
 *
 * # Errors
 *
 * If the variable specified doesn't exist in the bindings, this returns
 * [`Error::Error`] and updates `term` to a null pointer.
 *
 * # Safety
 *
 * - `variable` should be a null-terminated UTF-8 encoded string. -
 * `bindings` should point to a [`bindings`] previously created with
 * [`scryer_leaf_answer_unwrap_bindings`].
 */
enum scryer_Error scryer_bindings_get(const struct scryer_Bindings *bindings,
                                      const char *variable,
                                      struct scryer_Term **term);

/**
 * Drops a [`Term`].
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
void scryer_term_drop(struct scryer_Term *term);

/**
 * Gets the kind of a [`Term`].
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_TermKind scryer_term_kind(const struct scryer_Term *term);

/**
 * Unwraps a big integer from a [`Term`].
 *
 * If `term` is an integer, succeeds and updates `big_integer` with  a
 * null-terminated string representing that integer. This is so that arbitrary
 * precision can be supported. If you need an actual integer you should parse
 * this string.
 *
 * # Errors
 *
 * If `term` is not an integer, returns [`Error::Error`] and updates
 * `big_integer` to a null pointer.
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_Error scryer_term_unwrap_integer(const struct scryer_Term *term, char **big_integer);

/**
 * Unwraps a float from a [`Term`].
 *
 * If `term` is a float, succeeds and updates `scryer_float` with it's value.
 *
 * # Errors
 *
 * If `term` is not a float, returns [`Error::Error`] and updates
 * `scryer_float` to `0.0`.
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_Error scryer_term_unwrap_float(const struct scryer_Term *term, double *scryer_float);

/**
 * Unwraps a rational from a [`Term`].
 *
 * If `term` is a rational, succeeds and updates `numerator` and
 * `denominator` with a strings representing their values, like in
 * [`scryer_term_unwrap_integer`].
 *
 * # Errors
 *
 * If `term` is not a rational, returns [`Error::Error`] and updates
 * `numerator` and `denominator` to a null pointers.
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_Error scryer_term_unwrap_rational(const struct scryer_Term *term,
                                              char **numerator,
                                              char **denominator);

/**
 * Unwraps an atom from a [`Term`].
 *
 * If `term` is an atom, succeeds and updates `atom` with a null terminated
 * string with it's contents.
 *
 * # Errors
 *
 * If `term` is not an atom, returns [`Error::Error`] and updates `atom` to a
 * null pointer.
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_Error scryer_term_unwrap_atom(const struct scryer_Term *term, char **atom);

/**
 * Unwraps a string from a [`Term`].
 *
 * If `term` is a string, succeeds and updates `string` with a null terminated
 * string with it's contents.
 *
 * # Errors
 *
 * If `term` is not a string, returns [`Error::Error`] and updates `string` to
 * a null pointer.
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_Error scryer_term_unwrap_string(const struct scryer_Term *term, char **string);

/**
 * Unwraps a list from a [`Term`].
 *
 * If `term` is a list, succeeds and updates `term_list` with a pointer to a
 * buffer containing pointers to terms, and `len` to the number of terms in
 * that buffer.
 *
 * This buffer needs to be dropped with `scryer_list_drop`.
 *
 * # Errors
 *
 * If `term` is not a list, returns [`Error::Error`], updates `term_list` to a
 * null pointer and `len` to 0.
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_Error scryer_term_unwrap_list(const struct scryer_Term *term,
                                          struct scryer_Term ***term_list,
                                          uintptr_t *len);

/**
 * Unwraps a compound from a [`Term`].
 *
 * If `term` is a compound, succeeds and updates `functor` with a null
 * terminated string with the contents of the functor, `args` with a buffer
 * with containing pointers to terms, and `len` to the number of terms in that
 * buffer.
 *
 * `args` needs to be dropped with `scryer_list_drop`.
 *
 * # Errors
 *
 * If `term` is not a compound, returns [`Error::Error`], updates `functor` and
 * `args` to a null pointers, and `len` to 0.
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_Error scryer_term_unwrap_compound(const struct scryer_Term *term,
                                              char **functor,
                                              struct scryer_Term ***args,
                                              uintptr_t *len);

/**
 * Unwraps a variable from a [`Term`].
 *
 * If `term` is a variable, succeeds and updates `variable` with a null
 * terminated variable with it's name.
 *
 * # Errors
 *
 * If `term` is not a variable, returns [`Error::Error`] and updates `variable`
 * to a null pointer.
 *
 * # Safety
 *
 * `term` should point to a [`Term`] previously created by Scryer Prolog.
 */
enum scryer_Error scryer_term_unwrap_variable(const struct scryer_Term *term, char **variable);

/**
 * Drop a previously allocated string.
 *
 * # Safety
 *
 * `string` should be a string previously allocated by Scryer Prolog.
 */
void scryer_string_drop(char *string);

/**
 * Drop a previously allocated list.
 *
 * This only frees the memory for the list itself. The elements it contains
 * should be dropped first separatelly.
 *
 * # Safety
 *
 * `list` should be a list previously created with [`scryer_term_unwrap_list`],
 * and `len` should be it's length.
 */
void scryer_list_drop(struct scryer_Term **list, uintptr_t len);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
