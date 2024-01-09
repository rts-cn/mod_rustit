FREESWITCH_DIR ?= /usr/local/freeswitch
FREESWITCH_MOD_DIR ?= $(FREESWITCH_DIR)/mod

build:
	cargo build

install:
	cp ./target/debug/libmod_zrapi.so ./target/debug/mod_zrapi.so && \
	install -s  -p -D -m 0755 ./target/debug/mod_zrapi.so  $(FREESWITCH_MOD_DIR)/