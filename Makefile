UCONSYN=uconsyn/target/release/uconsyn

all: $(UCONSYN)
	$(UCONSYN) conmacro.ucon
	clips -l conmacro_boot.clp -f conmacro.con -f2 produce.clp

$(UCONSYN):
	cd uconsyn && cargo build --release


