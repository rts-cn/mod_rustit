FREESWITCH_DIR ?= /usr/local/freeswitch
FREESWITCH_MOD_DIR ?= $(FREESWITCH_DIR)/mod

build:
	cargo build

install:
	cp ./target/debug/libmod_zrapi.so $(FREESWITCH_MOD_DIR)/mod_zrapi.so && \
	chmod 0755 $(FREESWITCH_MOD_DIR)/mod_zrapi.so