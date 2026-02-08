PREFIX   ?= $(HOME)/.local
LIB_DIR  := $(PREFIX)/lib
BIN_DIR  := $(PREFIX)/bin
CONF_DIR := $(HOME)/.config

SO       := target/release/libcrtty_crt.so

.PHONY: build install uninstall clean

build:
	cargo build --release --workspace

install: build
	@mkdir -p $(LIB_DIR) $(BIN_DIR)
	@cp $(SO) $(LIB_DIR)/libcrtty_crt.so
	@echo '#!/bin/sh'                                          >  $(BIN_DIR)/crtty
	@echo 'exec env LD_PRELOAD=$(LIB_DIR)/libcrtty_crt.so \'  >> $(BIN_DIR)/crtty
	@echo '         ENABLE_CRTTY=1 \'                          >> $(BIN_DIR)/crtty
	@echo '         kitty "$$@"'                               >> $(BIN_DIR)/crtty
	@chmod +x $(BIN_DIR)/crtty
	@if [ ! -f $(CONF_DIR)/crtty.conf ]; then \
		mkdir -p $(CONF_DIR); \
		echo "# CRTty config";                        >  $(CONF_DIR)/crtty.conf; \
		echo "enabled=1";                             >> $(CONF_DIR)/crtty.conf; \
		echo "scanline_intensity=0.75";               >> $(CONF_DIR)/crtty.conf; \
		echo "phosphor_strength=1.1";                 >> $(CONF_DIR)/crtty.conf; \
		echo "curvature=0.04";                        >> $(CONF_DIR)/crtty.conf; \
		echo "vignette=0.35";                         >> $(CONF_DIR)/crtty.conf; \
		echo "aberration=0.003";                      >> $(CONF_DIR)/crtty.conf; \
		echo "  Created $(CONF_DIR)/crtty.conf"; \
	fi
	@echo ""
	@echo "  Installed:"
	@echo "    $(LIB_DIR)/libcrtty_crt.so"
	@echo "    $(BIN_DIR)/crtty"
	@echo ""
	@echo "  Run:  crtty"
	@echo ""

uninstall:
	@rm -f $(LIB_DIR)/libcrtty_crt.so
	@rm -f $(BIN_DIR)/crtty
	@echo "  Removed library and launcher."
	@echo "  Config kept at $(CONF_DIR)/crtty.conf (delete manually if wanted)."

clean:
	cargo clean
