// imports
#include <stdlib.h>
#include <unistd.h>

#include <fstream>
#include <iostream>
#include <queue>
#include <sstream>
#include <string>
#include <vector>

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

#define O_trace(fmt...)     \
    do {                    \
        if (O_option) {     \
            printf(fmt);    \
            printf("\n");   \
            fflush(stdout); \
        }                   \
    } while (0)
#define P_trace(fmt...)     \
    do {                    \
        if (P_option) {     \
            printf(fmt);    \
            printf("\n");   \
            fflush(stdout); \
        }                   \
    } while (0)
#define F_trace(fmt...)     \
    do {                    \
        if (F_option) {     \
            printf(fmt);    \
            printf("\n");   \
            fflush(stdout); \
        }                   \
    } while (0)
#define S_trace(fmt...)     \
    do {                    \
        if (S_option) {     \
            printf(fmt);    \
            printf("\n");   \
            fflush(stdout); \
        }                   \
    } while (0)
#define x_trace(fmt...)     \
    do {                    \
        if (x_option) {     \
            printf(fmt);    \
            printf("\n");   \
            fflush(stdout); \
        }                   \
    } while (0)
#define y_trace(fmt...)     \
    do {                    \
        if (y_option) {     \
            printf(fmt);    \
            printf("\n");   \
            fflush(stdout); \
        }                   \
    } while (0)
#define f_trace(fmt...)     \
    do {                    \
        if (f_option) {     \
            printf(fmt);    \
            printf("\n");   \
            fflush(stdout); \
        }                   \
    } while (0)
#define a_trace(fmt...)     \
    do {                    \
        if (a_option) {     \
            printf(fmt);    \
            printf("\n");   \
            fflush(stdout); \
        }                   \
    } while (0)

// basic classes
typedef struct {
    int pid;
    int virtual_page_number;
    unsigned int age;
} frame_t;

typedef struct {
    int frame_number;
    bool valid;
    bool referenced;
    bool modified;
    bool paged_out;
    bool write_protected;
    bool file_mapped;
    bool is_valid_vma;
} pte_t;

struct PageTable {
    pte_t entries[MAX_VPAGES];
};

struct VirtualMemoryArea {
    int start;
    int end;
    bool write_protected;
    bool file_mapped;
};

struct Process {
    std::vector<VirtualMemoryArea> virtual_memory_areas;
    PageTable page_table;

    // stats
    uint64_t unmaps;
    uint64_t maps;
    uint64_t ins;
    uint64_t outs;
    uint64_t fins;
    uint64_t fouts;
    uint64_t zeros;
    uint64_t segv;
    uint64_t segprot;
};

// global variables
int n_frames;
frame_t frame_table[MAX_FRAMES];
std::queue<int> free_frame_list;
std::vector<Process *> processes;
std::vector<std::pair<char, int> > instructions;
std::vector<uint64_t> random_numbers;

class Pager {
   public:
    uint16_t hand = 0;
    virtual int select_victim_frame() = 0;
    virtual void update_age(frame_t *frame) { ; };
};

class FIFO : public Pager {
   public:
    int select_victim_frame() override {
        int frame = hand;
        a_trace("ASELECT %d", frame % n_frames);
        hand = (hand + 1) % n_frames;
        return frame;
    }
};
class Random : public Pager {
   public:
    int select_victim_frame() override {
        int frame = random_numbers[hand] % n_frames;
        a_trace("ASELECT %d", frame);
        hand = (hand + 1) % random_numbers.size();
        return frame;
    }
};

class Clock : public Pager {
   public:
    int select_victim_frame() override {
        int i = hand;
        while (true) {
            frame_t *frame = &frame_table[i % n_frames];
            pte_t *pte = &processes[frame->pid]
                              ->page_table.entries[frame->virtual_page_number];
            if (pte->referenced) {
                pte->referenced = false;
                i++;
            } else {
                a_trace("ASELECT %d %d", hand, i - hand + 1);
                hand = (i + 1) % n_frames;
                return i % n_frames;
            }
        }
    }
};
// class NRU : public Pager {};
// class Aging : public Pager {};
// class WorkingSet : public Pager {};

std::string frame_table_str() {
    std::string outstring = "FT:";
    for (int i = 0; i < n_frames; i++) {
        frame_t *frame = &frame_table[i];
        if (frame->pid == -1) {
            outstring += " *";
        } else {
            outstring += " " + std::to_string(frame->pid) + ":" +
                         std::to_string(frame->virtual_page_number);
        }
    }

    return outstring;
}

std::string page_table_str(int pid) {
    std::string outstring = "PT[" + std::to_string(pid) + "]:";
    Process *process = processes[pid];

    for (int i = 0; i < MAX_VPAGES; i++) {
        pte_t *pte = &process->page_table.entries[i];
        if (pte->paged_out && !pte->valid && !pte->file_mapped) {
            outstring += " #";
        } else if (!pte->is_valid_vma || !pte->valid) {
            outstring += " *";
        } else {
            outstring += " " + std::to_string(i) + ":";
            if (pte->referenced) {
                outstring += "R";
            } else {
                outstring += "-";
            }
            if (pte->modified) {
                outstring += "M";
            } else {
                outstring += "-";
            }
            if (pte->paged_out) {
                outstring += "S";
            } else {
                outstring += "-";
            }
        }
    }

    return outstring;
}

class Simulator {
   public:
    int current_pid = -1;
    Pager *pager;
    int instruction;

    // stats
    uint64_t process_exits = 0;
    uint64_t ctx_switches = 0;

   public:
    Simulator(Pager *pager) {
        this->pager = pager;

        // initialize frame table
        for (int i = 0; i < MAX_FRAMES; i++) {
            frame_table[i].pid = -1;
            frame_table[i].virtual_page_number = -1;
            frame_table[i].age = 0;
        }

        // initialize free frame list
        for (int i = 0; i < n_frames; i++) {
            free_frame_list.push(i);
        }
    }

    int get_frame() {
        // if there is a free frame, return it
        if (!free_frame_list.empty()) {
            int frame_idx = free_frame_list.front();
            free_frame_list.pop();
            return frame_idx;
        }

        // otherwise, select a victim frame
        int victim_frame_idx = pager->select_victim_frame();
        frame_t *victim_frame = &frame_table[victim_frame_idx];
        Process *victim_process = processes[victim_frame->pid];
        pte_t *victim_pte = &victim_process->page_table
                                 .entries[victim_frame->virtual_page_number];

        O_trace(" UNMAP %d:%d", victim_frame->pid,
                victim_frame->virtual_page_number);
        victim_process->unmaps++;

        victim_pte->valid = false;
        // victim_pte->referenced = false;
        if (victim_pte->modified) {
            victim_pte->modified = false;
            if (victim_pte->file_mapped) {
                O_trace(" FOUT");
                victim_process->fouts++;
            } else {
                victim_pte->paged_out = true;
                O_trace(" OUT");
                victim_process->outs++;
            }
        }
        return victim_frame_idx;
    }

    int page_fault_handler(int virtual_page_number) {
        Process *current_process = processes[current_pid];
        pte_t *pte = &current_process->page_table.entries[virtual_page_number];

        // check if the page is valid vma
        bool is_valid_vma = false;
        for (int i = 0; i < current_process->virtual_memory_areas.size(); i++) {
            VirtualMemoryArea *vma = &current_process->virtual_memory_areas[i];
            if (virtual_page_number >= vma->start &&
                virtual_page_number <= vma->end) {
                is_valid_vma = true;
                pte->is_valid_vma = is_valid_vma;
                pte->file_mapped = vma->file_mapped;
                pte->write_protected = vma->write_protected;
                break;
            }
        }
        if (!is_valid_vma) {
            O_trace(" SEGV");
            current_process->segv++;
            return 1;  // signal error
        }

        // all valid get a free frame
        int frame_idx = get_frame();
        frame_t *frame = &frame_table[frame_idx];
        pte->valid = true;
        pte->referenced = true;
        pte->frame_number = frame_idx;

        frame->pid = current_pid;
        frame->virtual_page_number = virtual_page_number;
        // pager->update_age(frame); // TODO need to check what's happening

        if (pte->file_mapped) {
            current_process->fins++;
            O_trace(" FIN");
        } else if (pte->paged_out) {
            current_process->ins++;
            O_trace(" IN");
        } else {
            current_process->zeros++;
            O_trace(" ZERO");
        }
        O_trace(" MAP %d", frame_idx);
        current_process->maps++;

        return 0;  // signal success
    }

    void run() {
        for (int i = 0; i < instructions.size(); i++) {
            char operation = instructions[i].first;
            int value = instructions[i].second;
            // Process *process = processes[value];
            // PageTable *page_table = &process->page_table;

            O_trace("%d: ==> %c %d", i, operation, value);
            if (operation == 'c') {
                current_pid = value;
                ctx_switches++;
                continue;
            } else if (operation == 'e') {
                printf("EXIT current process %d\n", value);
                process_exits++;
                Process *process = processes[current_pid];
                for (int i = 0; i < MAX_VPAGES; i++) {
                    pte_t *pte = &process->page_table.entries[i];
                    if (pte->valid) {
                        O_trace(" UNMAP %d:%d", current_pid, i);
                        process->unmaps++;
                        frame_t *frame = &frame_table[pte->frame_number];
                        frame->pid = -1;
                        frame->virtual_page_number = -1;
                        frame->age = 0;
                        free_frame_list.push(pte->frame_number);
                        if (pte->modified && pte->file_mapped) {
                            O_trace(" FOUT");
                            process->fouts++;
                        }
                    }
                    pte->valid = false;
                    pte->paged_out = false;
                }
                continue;
            } else if (operation == 'r' || operation == 'w') {
                Process *process = processes[current_pid];
                pte_t *pte = &process->page_table.entries[value];

                // check if the page is valid
                if (!pte->valid) {
                    if (page_fault_handler(value)) {
                        continue;
                    }
                }

                pte->referenced = true;
                if (operation == 'w') {
                    if (pte->write_protected) {
                        O_trace(" SEGPROT");
                        process->segprot++;
                    } else {
                        pte->modified = true;
                    }
                }
                x_trace("%s", page_table_str(current_pid).c_str());
                f_trace("%s", frame_table_str().c_str());
            }
        }

        // print summary
        if (P_option) {
            for (int i = 0; i < processes.size(); i++) {
                P_trace("%s", page_table_str(i).c_str());
            }
        }
        F_trace("%s", frame_table_str().c_str());
        if (S_option) {
            uint64_t cost = 0;
            for (int i = 0; i < processes.size(); i++) {
                Process *proc = processes[i];
                printf(
                    "PROC[%d]: U=%llu M=%llu I=%llu O=%llu FI=%llu FO=%llu "
                    "Z=%llu "
                    "SV=%llu SP=%llu\n",
                    i, proc->unmaps, proc->maps, proc->ins, proc->outs,
                    proc->fins, proc->fouts, proc->zeros, proc->segv,
                    proc->segprot);
                cost +=
                    proc->unmaps * 410 + proc->maps * 350 + proc->ins * 3200 +
                    proc->outs * 2750 + proc->fins * 2350 + proc->fouts * 2800 +
                    proc->zeros * 150 + proc->segv * 440 + proc->segprot * 410;
            }
            cost += ctx_switches * 130 + process_exits * 1230 +
                    instructions.size() - process_exits - ctx_switches;
            printf("TOTALCOST %lu %llu %llu %llu %lu\n", instructions.size(),
                   ctx_switches, process_exits, cost, 4);
        }
    }
};

// functions
void read_random_file(const std::string &randomfile) {
    // Function that takes the name of a file with random numbers and stores
    // them in a global variable called random_values
    std::ifstream rfile(randomfile);
    int n_random, number;
    rfile >> n_random;
    while (rfile >> number) {
        random_numbers.push_back(number);
    }
}
void read_input_file(const std::string &filename) {
    std::ifstream file(filename);
    std::string line;

    // skip comments [at the beginning]
    while (getline(file, line)) {
        if (line.at(0) == '#') continue;
        break;
    }

    // first line that is not a comment is the number of
    // processes
    int n_processes = stoi(line);

    for (int i = 0; i < n_processes; i++) {
        // skip comments
        while (getline(file, line)) {
            if (line.at(0) == '#') continue;
            break;
        }
        int n_vmas = stoi(line);  // number of virtual memory areas
        std::vector<VirtualMemoryArea> vmas;
        for (int j = 0; j < n_vmas; j++) {
            while (getline(file, line)) {
                if (line.at(0) == '#') continue;  // Ignoring all the comments
                std::stringstream vma;
                vma << line;
                int start, end;
                bool w_protected, f_mapped;
                vma >> start >> end >> w_protected >> f_mapped;
                vmas.push_back(
                    VirtualMemoryArea{start, end, w_protected, f_mapped});
                break;
            }
        }
        // create the process
        Process *process = new Process();
        process->virtual_memory_areas = vmas;
        process->page_table = PageTable();
        processes.push_back(process);
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
    // string usage = "./mmu –f<num_frames> -a<algo>
    // [-o<options>] inputfile randomfile";
    int c;
    char alg;
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
                        // case 'e':
                        //     pager = new NRU();
                        //     break;
                        // case 'a':
                        //     pager = new Aging();
                        //     break;
                        // case 'w':
                        //     pager = new WorkingSet();
                        //     break;
                }
                break;
            case 'o':
                std::string option_string(optarg);
                for (char const &option : option_string) {
                    if (option == 'O') O_option = 1;
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

    // // globally mutate the processes and instructions
    read_input_file(inputfile);
    read_random_file(randomfile);

    Simulator simulator(pager);
    simulator.run();
}
