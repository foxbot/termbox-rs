lib=libtermbox.rlib
main=lib.rs
sources=lib.rs

all: ${lib}

${lib}: ${sources}
	rustc ${main}
