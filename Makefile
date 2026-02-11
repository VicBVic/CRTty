PREFIX   ?= $(HOME)/.local
DESTDIR  ?=
LIB_DIR  := $(DESTDIR)$(PREFIX)/lib
BIN_DIR  := $(DESTDIR)$(PREFIX)/bin
CONF_DIR := $(HOME)/.config

SO       := target/release/libcrtty_crt.so
BIN      := target/release/crtty

.PHONY: build install uninstall clean

build:
	cargo build --release --workspace

install: build
	@mkdir -p $(LIB_DIR) $(BIN_DIR)
	@cp $(SO) $(LIB_DIR)/libcrtty_crt.so
	@cp $(BIN) $(BIN_DIR)/crtty
	@if [ -z "$(DESTDIR)" ] && [ ! -f $(CONF_DIR)/crtty.conf ]; then \
		mkdir -p $(CONF_DIR); \
		cp crtty.conf.example $(CONF_DIR)/crtty.conf; \
		echo "  Created $(CONF_DIR)/crtty.conf"; \
	fi
	@echo ""
	@echo "  Installed:"
	@echo "    $(LIB_DIR)/libcrtty_crt.so"
	@echo "    $(BIN_DIR)/crtty"
	@echo ""

uninstall:
	@rm -f $(LIB_DIR)/libcrtty_crt.so
	@rm -f $(BIN_DIR)/crtty
	@echo "  Removed library and CLI."
	@echo "  Config kept at $(CONF_DIR)/crtty.conf (delete manually if wanted)."

clean:
	cargo clean
