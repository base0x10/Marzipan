# Marzipan
Joseph Espy

An emulator written in C++ for the Redcode assembly language used in the game CoreWar where assembly programs seek and destroy 
each other in a virtualized cpu core environment.  


### How to use:

Marzipan has minimal dependencies.  We require `make` to build the project.  Pretty much any C++ compilier should work.  

To run Marzipan:
  - run `git clone https://github.com/base0x10/Marzipan.git` to pull the emulator
  - run `make` from the root directory of `Marzipan`
  - execute the marzipan executable.  At this time it takes arguments `<warrior 1 offset> <warrior 1 file> <warrior 2 offset> <warrior 2 file> <number of cycles to execute>`
