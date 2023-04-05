import argparse
import re
from collections import deque


frame_table: list[int] = [0] * 256


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
    def select_victim_frame(self):
        raise NotImplementedError()


class MMU:
    def __init__(self, max_frames):
        self.frame_table = [Frame() for _ in range(max_frames)]
        self.free_frames = deque(range(max_frames))
        self.page_table = [PTE() for _ in range(64)]

    def get_frame(self) -> int:
        if len(self.free_frames) > 0:
            frame_idx = self.free_frames.popleft()
            return frame_idx
        else:
            # TODO: Implement page replacement algorithm
            return None #self.pager.select_victim_frame()

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
                print('ZERO')
                print(f'MAP {frame_idx}')
        elif operation == 'w':
            pass
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

    print("Options:")
    print(f"Number of frames: {num_frames}")
    print(f"Algorithm: {algorithm}")
    print(f"Options: {options}")
    print(f"Input file: {inputfile}")
    print(f"Random file: {randomfile}")

    main(args)
