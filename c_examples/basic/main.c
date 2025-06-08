#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>

#include <scryer_prolog.h>

int main() {
    // For error handling
    scryer_Error error;

    char *program =
    "a(1).\n"
    "a(2).\n"
    "a(3).\n";

    // Create the machine
    scryer_MachineBuilder *machine_builder = scryer_machine_builder_new();
    scryer_Machine *machine = scryer_machine_builder_build(machine_builder);
    printf("Created the machine\n");

    // Consult a module
    error = scryer_machine_consult_module_string(machine, "test_module", program);
    if (error != SCRYER_ERROR_SUCCESS) exit(1);
    printf("Consulted the module\n");

    // Start a query
    scryer_QueryState *query_state = NULL;
    error = scryer_machine_run_query(machine, "a(A).", &query_state);
    if (error != SCRYER_ERROR_SUCCESS) exit(1);
    printf("Started the query\n");

    while (true) {
        // Get the next answer
        scryer_LeafAnswer *leaf_answer = NULL;
        error = scryer_query_state_next_answer(query_state, &leaf_answer);
        if (error != SCRYER_ERROR_SUCCESS) exit(1);

        // NULL indicates that we reached the end of the iterator of answers
        if (leaf_answer == NULL) break;

        printf("Got the leaf answer\n");
        
        // Get the bindings
        scryer_Bindings *bindings = NULL;
        error = scryer_leaf_answer_unwrap_bindings(leaf_answer, &bindings);
        if (error != SCRYER_ERROR_SUCCESS) exit(1);
        printf("Got the bindings\n");

        // Get the term bound to the A variable
        scryer_Term *term = NULL;
        error = scryer_bindings_get(bindings, "A", &term);
        if (error != SCRYER_ERROR_SUCCESS) exit(1);
        printf("Got the term\n");

        // Get the integer value of the term as a string
        char *big_integer = NULL;
        error = scryer_term_unwrap_integer(term, &big_integer);
        if (error != SCRYER_ERROR_SUCCESS) exit(1);
        printf("Got the big integer\n");

        // Prints the value
        printf("A = %s\n", big_integer);

        // Frees stuff done in this loop iteration
        scryer_string_drop(big_integer);
        scryer_term_drop(term);
        scryer_bindings_drop(bindings);
        scryer_leaf_answer_drop(leaf_answer);
    }

    // Frees the query state
    scryer_query_state_drop(query_state);

    // Frees the machine
    scryer_machine_drop(machine);
}
