.DEFAULT_GOAL := c_lib_test

PROJECT_ROOT=../../..
INCLUDE_DIR=$(PROJECT_ROOT)/compiler/include
LIB=$(PROJECT_ROOT)/target/debug
OUT=c_lib_test

barracuda_header:
	cd $(PROJECT_ROOT) && mkdir -p include && cargo test --features c-headers -- generate_headers

barracuda_library: barracuda_header
	cd $(PROJECT_ROOT) && cargo build --lib

c_lib_test: main.c barracuda_library
	g++ main.c -o $(OUT) -l barracuda_compiler -I $(INCLUDE_DIR) -L $(LIB) -Wl,-rpath='$(LIB)'

.PHONY: clean
clean:
	rm $(OUT)
	cd $(PROJECT_ROOT) && rm -rf target/ && rm include/*.h -f
