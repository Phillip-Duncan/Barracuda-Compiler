#include <cstdlib>
#include <stdio.h>
#include <string.h>
#include <barracuda_compiler.h>

const char* test_code = "fn fib(n) {\nlet a = 0;\nlet b = 1;\nfor (let i = 0; i < n; i = i + 1) {\nlet temp = a + b;\na = b;\nb = temp;\nprint a;\n}\n}\nextern count;\nfib(count);";
const char* identifier = "count";

int main(int argc, char *argv[]) {
    printf("Testing calling barracuda compiler from a c file.\n");

    // Create Request
    CompilerRequest request;
    request.code_text = strdup(test_code);

    request.env_vars.ptr = (EnvironmentVariable_t*)malloc(sizeof(EnvironmentVariable_t));
    request.env_vars.len = 1;
    request.env_vars.cap = 1;

    request.env_vars.ptr->identifier = strdup(identifier);
    request.env_vars.ptr->ptr_offset = 0;


    {
        // Send Request
        CompilerResponse response = compile(&request);

        // Process Response
        printf("Code:\n%s\n\n", test_code);
        printf("Compiled:\n%s\n", response.code_text);
        for(int i = 0; i < response.values_list.len; i++) {
            printf("%f\n", response.values_list.ptr[i]);
        }
        // ...

        // Don't forget to free the response
        free_compile_response(response);
    }
    free(request.env_vars.ptr);
}