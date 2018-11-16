OBJ_DIR = obj
DEFAULT_EMULATOR = bs_emulator
DEFAULT_SERVER = cli-server

.PHONY: clean output_dir emulators servers

marzipan : emulators servers
	g++ -o marzipan $(OBJ_DIR)/$(DEFAULT_EMULATOR).o $(OBJ_DIR)/$(DEFAULT_SERVER).o

emulators: output_dir
	$(MAKE) -C src/emulators

servers: output_dir
	$(MAKE) -C src/servers

output_dir:
	mkdir -p $(OBJ_DIR)

clean:
	rm -r $(OBJ_DIR) ||:
	$(MAKE) -C src/servers clean
	$(MAKE) -C src/emulators clean
