# Build the real extension with Cargo after the PHP build system creates the module directory.
.PHONY: rabbitmq_ext_cargo_build

all: rabbitmq_ext_cargo_build

rabbitmq_ext_cargo_build: $(phplibdir)/rabbitmq_ext.la
	cd $(srcdir) && cargo build --release
	cp $(srcdir)/target/release/librabbitmq_ext.so $(phplibdir)/rabbitmq_ext.so
