#include <stdio.h>
#include <barracuda_compiler.h>

int main(int argc, char *argv[]) {
    printf("Testing calling barracuda compiler from a c file.\n");

    // From barracuda_compiler.h
    hello_world();
}