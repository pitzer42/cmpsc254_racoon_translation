# Python

`python3 -O ./py65/monitor.py --mpu 65C02 --load /home/j/school/254_py65/emulator-6502/benchmark1/dump/mapache64.bin --goto 5038 >> benchmark1_python.txt`

Note that "./py65/monitor.py" should be a path to the changed monitor.py file inside the py65 project. 
This command is run inside the py65 project, NOT inside this project, but the monitor.py file should
match the one we include here. Additionally th output file will contain lots of extraneous info but the number directly underneath "Wrote +65536 bytes from $0000 to $ffff" is the number of nanoseconds it took to run the program.


Rust:
Build command: `cargo build --release`
Run command: `../../target/release/emulator-6502 >> ../../benchmark1_rust.txt`

