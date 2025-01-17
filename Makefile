UNAME := $(shell uname)

install_verilator:
ifeq ($(UNAME), Darwin)
	brew install verilator
else
	sudo apt-get install -y verilator
endif
