// imports
#include <stdlib.h>
#include <unistd.h>
#include <iostream>
#include <fstream>
#include <sstream>
#include <vector>
#include <string>


// constants
#define MAX_FRAMES 128
#define MAX_VPAGES 64

bool O_option;
bool P_option;
bool F_option;
bool S_option;
bool x_option;
bool y_option;
bool f_option;
bool a_option;

#define O_trace(fmt...) do { if (O_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define P_trace(fmt...) do { if (P_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define F_trace(fmt...) do { if (F_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define S_trace(fmt...) do { if (S_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define x_trace(fmt...) do { if (x_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define y_trace(fmt...) do { if (y_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define f_trace(fmt...) do { if (f_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)
#define a_trace(fmt...) do { if (a_option) { printf(fmt); printf("\n"); fflush(stdout); } } while(0)


// basic classes
class VirtualMemoryArea {
   public:
    int start;
    int end;
    bool w_protected;
    bool f_mapped;
    VirtualMemoryArea(int start, int end, bool w_protected, bool f_mapped) {
        this->start = start;
        this->end = end;
        this->w_protected = w_protected;
        this->f_mapped = f_mapped;
    }
};

class PageTableEntry {
   public:
    int frame_number;
    bool valid;
    bool referenced;
    bool modified;
    bool paged_out;
    bool write_protected;
    bool file_mapped;
    bool is_valid_vma;
    PageTableEntry() {
        this->frame_number = -1;
        this->valid = false;
        this->referenced = false;
        this->modified = false;
        this->paged_out = false;
        this->write_protected = false;
        this->file_mapped = false;
        this->is_valid_vma = false;
    }
};

class PageTable {
   public:
    std::vector<PageTableEntry> entries;
    PageTable() { entries.reserve(MAX_VPAGES); }
};

class Process {
   public:
    std::vector<VirtualMemoryArea> virtual_memory_areas;
    PageTable page_table;

    // stats
    unsigned int unmaps;
    unsigned int maps;
    unsigned int ins;
    unsigned int outs;
    unsigned int fins;
    unsigned int fouts;
    unsigned int zeros;
    unsigned int segv;
    unsigned int segprot;

    Process(std::vector<VirtualMemoryArea> virtual_memory_areas) {
        this->virtual_memory_areas = virtual_memory_areas;
        this->page_table = PageTable();
        this->unmaps = 0;
        this->maps = 0;
        this->ins = 0;
        this->outs = 0;
        this->fins = 0;
        this->fouts = 0;
        this->zeros = 0;
        this->segv = 0;
        this->segprot = 0;
    }
};

class Pager {};

class FIFO : public Pager {};
class Random : public Pager {};
class Clock : public Pager {};
class NRU : public Pager {};
class Aging : public Pager {};
class WorkingSet : public Pager {};

/// util functions
class RandGenerator {
   private:
    std::vector<int> random_numbers;

   public:
    int rand_index = 0;
    int number_of_random_values;
    RandGenerator(const std::string &filename) {
        int number;
        std::ifstream infile(filename);

        infile >> number_of_random_values;
        random_numbers.reserve(number_of_random_values);
        while (infile >> number) {
            random_numbers.push_back(number);
        }
    }

    int next(int value) {
        int rand = 1 + (random_numbers[rand_index] % value);
        rand_index = (rand_index + 1) % random_numbers.size();
        return rand;
    }
};

void read_input_file(const std::string &filename) {
    std::ifstream file(filename);
    std::string line;
    std::vector<Process *> processes;
    std::vector<std::pair<char, int> > instructions;

    // skip comments [at the beginning]
    while (getline(file, line)) {
        if (line.at(0) == '#') continue;
        break;
    }

    // first line that is not a comment is the number of processes
    int n_processes = stoi(line);

    for (int i = 0; i < n_processes; i++) {
        // skip comments
        while (getline(file, line)) {
            if (line.at(0) == '#') continue;
            break;
        }
        // processes.push_back(new Process);
        int n_vmas = stoi(line);  // number of virtual memory areas

        for (int j = 0; j < n_vmas; j++) {
            while (getline(file, line)) {
                if (line.at(0) == '#') continue;  // Ignoring all the comments
                std::stringstream vma;
                vma << line;
                int start, end;
                bool w_protected, f_mapped;
                vma >> start >> end >> w_protected >> f_mapped;
                // processes[i]->virtual_memory_areas.push_back(
                //     new VirtualMemoryArea(start, end, w_protected, f_mapped));
                break;
            }
        }
    }
    while (getline(file, line)) {
        if (line.at(0) == '#') continue;  // Ignoring all the comments

        char instruction;
        int argument;
        std::istringstream line_stream(line);
        line_stream >> instruction >> argument;
        instructions.push_back(std::make_pair(instruction, argument));
    }
}

int main(int argc, char *argv[]) {
    // string usage = "./mmu –f<num_frames> -a<algo> [-o<options>] inputfile
    // randomfile";
    int c;
    char alg;
    int n_frames;
    Pager *pager;
    while ((c = getopt(argc, argv, "f:a:o:")) != -1) {
        switch (c) {
            case 'f':
                n_frames = std::stoi(optarg);
                if (MAX_FRAMES < n_frames) {
                    exit(0);
                }
                break;
            case 'a':
                alg = optarg[0];
                switch (alg) {
                    case 'f':
                        pager = new FIFO();
                        break;
                    case 'r':
                        pager = new Random();
                        break;
                    case 'c':
                        pager = new Clock();
                        break;
                    case 'e':
                        pager = new NRU();
                        break;
                    case 'a':
                        pager = new Aging();
                        break;
                    case 'w':
                        pager = new WorkingSet();
                        break;
                }
                break;
            case 'o':
                std::string option_string(optarg);
                for (char const &option : option_string) {
                    if (option == 'o') O_option = 1;
                    if (option == 'P') P_option = 1;
                    if (option == 'F') F_option = 1;
                    if (option == 'S') S_option = 1;
                    if (option == 'x') x_option = 1;
                    if (option == 'y') y_option = 1;
                    if (option == 'f') f_option = 1;
                    if (option == 'a') a_option = 1;
                }
                break;
        }
    }

    std::string inputfile = argv[optind];
    std::string randomfile = argv[optind + 1];
    read_input_file(inputfile);
    RandGenerator random_generator(randomfile);
}
