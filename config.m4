PHP_ARG_ENABLE([rabbitmq_ext],
  [whether to enable rabbitmq_ext support],
  [AS_HELP_STRING([--enable-rabbitmq_ext], [Enable RabbitMQ extension (requires Rust/Cargo)])])

if test "$PHP_RABBITMQ_EXT" != "no"; then
  AC_PATH_PROG([CARGO], [cargo], [no])
  if test "x$CARGO" = "xno"; then
    AC_MSG_ERROR([cargo is required to build this extension. Install Rust from https://rustup.rs])
  fi

  AC_PATH_PROG([RUSTC], [rustc], [no])
  if test "x$RUSTC" = "xno"; then
    AC_MSG_ERROR([rustc is required to build this extension. Install Rust from https://rustup.rs])
  fi

  PHP_NEW_EXTENSION([rabbitmq_ext], [stub.c], [$ext_shared])
  PHP_ADD_MAKEFILE_FRAGMENT
fi
