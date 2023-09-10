# KTF

JAR file extracted from ktf phone contains `client.bin<number>` file instead of java class files. It's actually AOT compiled java class file into arm thumb binary. The number on the filename indicates bss(R/W data) section size.

We can execute it by loading to memory, and calling relocate function on RVA 0.
