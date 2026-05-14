ROOT_DIR := $(abspath ..)
PRIV_DIR := $(MIX_APP_PATH)/priv
NIF := $(PRIV_DIR)/jailguard_nif.so

UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
	JAILGUARD_LIB := libjailguard.dylib
	NIF_LDFLAGS := -dynamiclib -undefined dynamic_lookup -Wl,-rpath,@loader_path
else
	JAILGUARD_LIB := libjailguard.so
	NIF_LDFLAGS := -shared -Wl,-rpath,'$$ORIGIN'
endif

CC ?= cc
CFLAGS ?=
CFLAGS += -O3 -Wall -Wextra -Werror -fPIC

.PHONY: all clean rust-lib

all: $(NIF)

rust-lib:
	cargo build --manifest-path "$(ROOT_DIR)/Cargo.toml" --release --locked --lib
	mkdir -p "$(PRIV_DIR)"
	cp "$(ROOT_DIR)/target/release/$(JAILGUARD_LIB)" "$(PRIV_DIR)/$(JAILGUARD_LIB)"

$(NIF): c_src/jailguard_nif.c rust-lib
	mkdir -p "$(PRIV_DIR)"
	$(CC) $(CFLAGS) \
	    -I"$(ERTS_INCLUDE_DIR)" \
	    -I"$(ROOT_DIR)/include" \
	    c_src/jailguard_nif.c \
	    $(NIF_LDFLAGS) \
	    -L"$(PRIV_DIR)" -ljailguard \
	    -o "$(NIF)"

clean:
	rm -rf "$(PRIV_DIR)"
