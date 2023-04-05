import argparse
import re


# def main(args):


if __name__ == "__main__":

    def valid_schedspec(value):
        # Use regular expression to check for valid specification
        if not re.match(r'^[OPFSxXyYaAfF]+$', value):
            raise argparse.ArgumentTypeError(f'Invalid scheduler specification: {value}. Must only contain any of the following options: O, P, F, S, x, y, a, f.')
        return value

    parser = argparse.ArgumentParser(description="MMU program")
    parser.add_argument("-f", type=int, required=True, help="number of frames")
    parser.add_argument("-a", type=str, required=True, help="algorithm")
    parser.add_argument("-o", type=valid_schedspec, default="OPFS", help="options")
    parser.add_argument("inputfile", type=str, help="input file")
    parser.add_argument("randomfile", type=str, help="random file")
    
    # args = parser.parse_args()
    args = parser.parse_args("-f16 -aF mmu/lab3_assign/in1 mmu/lab3_assign/rfile".split())


    num_frames = args.f
    algorithm = args.a
    options = args.o
    inputfile = args.inputfile
    randomfile = args.randomfile

    print(f"Number of frames: {num_frames}")
    print(f"Algorithm: {algorithm}")
    print(f"Options: {options}")
    print(f"Input file: {inputfile}")
    print(f"Random file: {randomfile}")

    # main(args)
