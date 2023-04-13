import argparse
import re
from collections import deque


class Process:
    def __init__(self):
        self.page_table: list[int] = [0] * 64
        pass


class Frame:
    def __init__(self):
        self.pid = None
        self.vpage = None


class PTE:
    def __init__(self):
        self.present = False
        self.modified = False
        self.referenced = False
        self.pagedout = False
        self.frame = None


class Pager:
    def __init__(self):
        self.hand = 0

    def select_victim_frame(self):
        hand = self.hand
        self.hand = (hand + 1) % 64
        return hand


class MMU:
    def __init__(self, max_frames):
        self.frame_table = [Frame() for _ in range(max_frames)]
        self.free_frames = deque(range(max_frames))
        self.page_table = [PTE() for _ in range(64)]
        self.pager = Pager()

    def get_frame(self) -> int:
        if len(self.free_frames) > 0:
            frame_idx = self.free_frames.popleft()
            return frame_idx
        else:
            frame_idx = self.pager.select_victim_frame()
            print(
                f" UNMAP {self.frame_table[frame_idx].pid}:{self.frame_table[frame_idx].vpage}")
            if self.page_table[self.frame_table[frame_idx].vpage].modified:
                # {self.frame_table[frame_idx].pid}:{self.frame_table[frame_idx].vpage}")
                print(f" OUT")
            return frame_idx

    def process_instruction(self, operation, vpage):
        if operation == 'c':
            pass
        elif operation == 'r':
            
            if self.page_table[vpage].present:
                pass
            else:
                frame_idx = self.get_frame()
                frame = self.frame_table[frame_idx]
                self.page_table[vpage].present = True
                self.page_table[vpage].frame = frame
                frame.pid = 0
                frame.vpage = vpage
                print(' ZERO')
                print(f' MAP {frame_idx}')
            self.page_table[vpage].referenced = True
        elif operation == 'w':
            if self.page_table[vpage].present:
                pass
            else:
                frame_idx = self.get_frame()
                frame = self.frame_table[frame_idx]
                self.page_table[vpage].present = True
                self.page_table[vpage].frame = frame
                self.page_table[vpage].modified = True
                frame.pid = 0
                frame.vpage = vpage
                print(' ZERO')
                print(f' MAP {frame_idx}')
            self.page_table[vpage].referenced = True
        else:
            raise ValueError(f'Invalid operation: {operation}')

# Example Pager subclass


class SamplePager(Pager):
    def select_victim_frame(self):
        # Implement the victim frame selection logic here
        pass


def read_input_file(filename: str):
    with open(filename, 'r') as file:
        # Skip the first 3 comment lines
        for _ in range(3):
            file.readline()

        num_processes = int(file.readline().strip())

        processes = []
        for _ in range(num_processes):
            # Skip comment lines
            while (line := file.readline().strip()):
                if not line.startswith('#'):
                    break

            num_vmas = int(line.strip())

            vmas = []
            for _ in range(num_vmas):
                start, end, wprot, mmap = map(
                    int, file.readline().strip().split())
                vmas.append((start, end, wprot, mmap))

            processes.append(vmas)

        # Skip comment line
        file.readline()

        instructions = []
        while (line := file.readline().strip()):
            if line.startswith('#'):
                break
            operation, address = line.split()
            instructions.append((operation, int(address)))

    return processes, instructions


def main(args):

    # Read input file
    processes, instructions = read_input_file(args.inputfile)
    # print(processes)
    # print(instructions)
    mmu = MMU(args.f)

    for idx, (operation, address) in enumerate(instructions):
        print(f"{idx}: ==> {operation} {address}")
        mmu.process_instruction(operation, address)

    # print end stats in this format
    # PT[0]: 0:R-- 1:RM- * 3:R-- 4:R-- 5:RM- * * 8:R-- * * # * 13:R-- * * * * * * # 21:R-- * * 24:R-- * * * 28:RM- * 30:R-- 31:R-- * * * * * * 38:RM- * * * * * 44:R-- * * * 48:RM- * * # * * * # 56:R-- * * * * * * *
    # FT: 0:28 0:56 0:31 0:4 0:48 0:3 0:5 0:24 0:8 0:1 0:38 0:0 0:30 0:13 0:44 0:21
    # PROC[0]: U=10 M=26 I=0 O=4 FI=0 FO=0 Z=26 SV=0 SP=0
    # TOTALCOST 31 1 0 28260 4
    print("PT[0]:", end=" ")
    for vpage, pte in enumerate(mmu.page_table):
        if pte.present:
            print(f"{vpage}:{'R' if pte.referenced else '-'}{'M' if pte.modified else '-'}{'S' if pte.pagedout else '-'}", end=" ")
        else:
            print('*', end=" ")
    print(" ")

    print("FT:", " ".join(
        f"{frame.pid}:{frame.vpage}" for frame in mmu.frame_table))
    
    # for idx, process in enumerate(processes):
        # print(f"PROC[{idx}]: U={process['unmaps']} M={process['maps']} I={process['ins']} O={process['outs']} FI={process['fins']} FO={process['fouts']} Z={process['zeros']} SV={process['segv']} SP={process['segprot']}")
    
    ## inst_count, ctx_switches, process_exits, cost, sizeof(pte_t));
    # print(f"TOTALCOST {mmu.inst_count} {mmu.ctx_switches} {mmu.process_exits} {mmu.cost} 'sizeof(pte_t)'")


if __name__ == "__main__":

    def valid_schedspec(value):
        # Use regular expression to check for valid specification
        if not re.match(r'^[OPFSxXyYaAfF]+$', value):
            raise argparse.ArgumentTypeError(
                f'Invalid scheduler specification: {value}. Must only contain any of the following options: O, P, F, S, x, y, a, f.')
        return value

    parser = argparse.ArgumentParser(description="MMU program")
    parser.add_argument("-f", type=int, required=True, help="number of frames")
    parser.add_argument("-a", type=str, required=True, help="algorithm")
    parser.add_argument("-o", type=valid_schedspec,
                        default="OPFS", help="options")
    parser.add_argument("inputfile", type=str, help="input file")
    parser.add_argument("randomfile", type=str, help="random file")

    # args = parser.parse_args()
    args = parser.parse_args(
        "-f16 -aF mmu/lab3_assign/in1 mmu/lab3_assign/rfile".split())

    num_frames = args.f
    algorithm = args.a
    options = args.o
    inputfile = args.inputfile
    randomfile = args.randomfile

    # print("Options:")
    # print(f"Number of frames: {num_frames}")
    # print(f"Algorithm: {algorithm}")
    # print(f"Options: {options}")
    # print(f"Input file: {inputfile}")
    # print(f"Random file: {randomfile}")

    main(args)