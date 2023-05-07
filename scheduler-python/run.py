modes = [
    "F",
    "L",
    "S",
    "R2",
    "R5",
    "P2",
    "P5:3",
    "E2:5",
    "E4"
]

import os
for mode in modes:
    for i in range(7):
        os.system(f"python src/pScheduler.py --verbose -s{mode} --inputfile lab2_assign/input{i} > arun_out/out_{i}_{mode.replace(':', '_')}")
        os.system(f"python src/pScheduler.py -s{mode} --inputfile lab2_assign/input{i} > arun_outz/out_{i}_{mode.replace(':', '_')}")