UNAME := $(shell uname)

install_verilator:
ifeq ($(UNAME), Darwin)
	brew install verilator
else
	apt-get install -y verilator
endif
