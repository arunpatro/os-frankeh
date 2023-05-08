# os-frankeh

These are solutions to the 4 assignments for Operating Systems by Franke H. Lab3 and Lab4 are also done in rust. Lab2 is also done in python. 

Some notes:
1. For mmu, especially in aging and working set, if we enable the a_trace, then we append a lot of strings while rotating the head and this is the culprit for slowing down the program exponentially. 
2. mmu-rust uses bool flags instead of bit fields like in cpp, need to optimize that
3. I intend to benchmark the MMU on no. of files and parameters etc. 
4. Did not implement -q and -f for FLOOK as logic is the same as LOOK, and it works

Some comparisions between Rust and CPP:
1. Global variables in Rust is not so straight forward. I could manage to do it using thread_local! and accessing it with some weird closure. The simpler way to access globals would be via unsafe blocks. CPP makes global variables really easy, and obv much messier too.
2. CPP Objects and extension system of Pagers and Schedulers takes much less space then struct-impl-trait system in Rust, which can get a little verbose
3. CPP debugging was a pain, makefile is a pain, vscode-cpp is also a pain, its hard to watch global variables in cpp. Rust debugging and dev is a breeze - vscode support is amazing and much easier to 
4. Type System: Programming in rust and vscode makes it really easy to see which pointers and const references to use. For this project, i used a lot of unsigned integers and usize which would otherwise be used by a int (uint32_t is so cumbersome). I would prefer Option<usize> instead of int = -1 values to error check. Primitive type system is easier to reason with in rust. 

Overall, I get a feeling about the performance:
1. CPP can be faster than Rust because of lots of unsafe code, avoiding bound checks and option enums
2. Rust can be faster because it makes it easy to use more stack-memory and writing in safer/declarative programming could actually be more optimal to compile
