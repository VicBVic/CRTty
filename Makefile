PREFIX   ?= $(HOME)/.local
DESTDIR  ?=
LIB_DIR  := $(DESTDIR)$(PREFIX)/lib
BIN_DIR  := $(DESTDIR)$(PREFIX)/bin
CONF_DIR := $(HOME)/.config/crtty

SO       := target/release/libcrtty_crt.so
BIN      := target/release/crtty

.PHONY: build install uninstall clean

build:
	cargo build --release --workspace

install: build
	@mkdir -p $(LIB_DIR) $(BIN_DIR)
	@cp $(SO) $(LIB_DIR)/libcrtty_crt.so
	@cp $(BIN) $(BIN_DIR)/crtty
	@if [ -z "$(DESTDIR)" ] && [ ! -f $(CONF_DIR)/kitty.conf ]; then \
		mkdir -p $(CONF_DIR); \
		cp kitty.conf.example $(CONF_DIR)/kitty.conf; \
		echo "  Created $(CONF_DIR)/kitty.conf"; \
	fi
	@if [ -z "$(DESTDIR)" ] && [ ! -f $(CONF_DIR)/alacritty.conf ]; then \
		mkdir -p $(CONF_DIR); \
		cp alacritty.conf.example $(CONF_DIR)/alacritty.conf; \
		echo "  Created $(CONF_DIR)/alacritty.conf"; \
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
	@echo "  Config kept at $(CONF_DIR)/kitty.conf and $(CONF_DIR)/alacritty.conf"

clean:
	cargo clean
