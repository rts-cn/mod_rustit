FREESWITCH_DIR ?= /usr/local/freeswitch
FREESWITCH_MOD_DIR ?= $(FREESWITCH_DIR)/mod

build:
	cargo build

install:
	cp ./conf/autoload_configs/zrs.conf.xml $(FREESWITCH_DIR)/conf/autoload_configs
	
	cp ./target/debug/libmod_zrs.so ./target/debug/mod_zrs.so && \
	install -s  -p -D -m 0755 ./target/debug/mod_zrs.so  $(FREESWITCH_MOD_DIR)/

generate:
	cargo build --features codegen