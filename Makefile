FREESWITCH_DIR ?= /usr/local/freeswitch
FREESWITCH_MOD_DIR ?= $(FREESWITCH_DIR)/mod

build:
	cargo build
	cp ./target/debug/libmod_zrs.so ./target/mod_zrs.so

install:
	install -s -p -D -m 0755 ./target/mod_zrs.so  $(FREESWITCH_MOD_DIR)/

conf:
	cp ./conf/autoload_configs/zrs.conf.xml $(FREESWITCH_DIR)/conf/autoload_configs
	
generate:
	cargo build --features codegen

release:
	cargo build --release 
	cp ./target/release/libmod_zrs.so ./target/mod_zrs.so