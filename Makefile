PROJECT_NAME = signals

# Sources
LIB_SRC = src/$(PROJECT_NAME).rs
DEPS_FILE = target/.$(PROJECT_NAME).deps
LIB_DEPS = $(shell head -n1 $(DEPS_FILE) 2> /dev/null)
EXAMPLES_SRC = $(wildcard examples/*.rs)

# Objects
LIB = target/$(shell rustc --crate-file-name ${LIB_SRC})
EXAMPLES_BIN = $(EXAMPLES_SRC:examples/%.rs=bin/%)

# CFG Directive Options
CFG_OPT ?= -O

all: ${LIB} ${EXAMPLES_BIN}

lib: ${LIB}

${LIB}: ${LIB_DEPS}
	@mkdir -p target
	rustc ${CFG_OPT} --out-dir target ${LIB_SRC}
	@rustc --no-trans --dep-info $(DEPS_FILE) ${LIB_SRC}
	@sed -i 's/.*: //' $(DEPS_FILE)

${EXAMPLES_BIN}: bin/%: examples/%.rs ${LIB}
	@mkdir -p bin
	rustc --out-dir bin -L target $<

doc: ${LIB_DEPS}
	rm -rf doc
	rustdoc ${LIB_SRC}

clean:
	rm -rf target bin doc

.PHONY: all clean
