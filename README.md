# os-frankeh

These are solutions to the 4 assignments for Operating Systems by Franke H. Lab3 and Lab4 are also done in rust. Lab2 is also done in python. 

Some notes:
1. For mmu, especially in aging and working set, if we enable the a_trace, then we append a lot of strings while rotating the head and this is the culprit for slowing down the program exponentially. 
2. mmu-rust uses bool flags instead of bit fields like in cpp, need to optimize that
3. I intend to benchmark the MMU on no. of files and parameters etc. 
4. Did not implement -q and -f for FLOOK as logic is the same as LOOK, and it works