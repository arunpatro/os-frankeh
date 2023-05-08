#include <unistd.h>

#include <fstream>
#include <iostream>
#include <queue>
#include <sstream>
#include <string>
#include <vector>

struct io_operation {
    int time;
    int track;
    int start_time;
    int completd_time;
    io_operation(int time, int track)
        : time(time), track(track), start_time(-1), completd_time(-1) {}
};

std::vector<io_operation> read_input_file(const std::string &filename) {
    std::ifstream file(filename);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open file");
    }

    std::vector<io_operation> io_operations;
    std::string line;
    while (std::getline(file, line)) {
        if (line.empty() || line[0] == '#') {
            continue;
        }

        std::istringstream iss(line);
        int time, track;
        if (!(iss >> time >> track)) {
            throw std::runtime_error("Failed to parse time and track");
        }

        io_operations.push_back(io_operation(time, track));
    }

    return io_operations;
}

class Scheduler {
   public:
    std::queue<io_operation *> io_queue;
    Scheduler() {}
    virtual ~Scheduler() {}
    virtual void add(int arrival, int track) = 0;
    virtual void next() = 0;
    virtual bool is_empty() = 0;
};

class FIFO : public Scheduler {
   public:
    FIFO() {}
    ~FIFO() {}
    void add(int arrival, int track) override {}
    void next() override {}
    bool is_empty() override { return true; }
};

int main(int argc, char *argv[]) {
    int c;
    char alg;
    Scheduler *sched;
    // Update the getopt string to include 's', 'v', 'q', and 'f'
    while ((c = getopt(argc, argv, "s:vqf")) != -1) {
        switch (c) {
            case 's':
                alg = optarg[0];
                switch (alg) {
                    case 'N':
                        sched = new FIFO();
                        break;
                    // case 'S':
                    //     sched = new SSTF();
                    //     break;
                    // case 'L':
                    //     sched = new LOOK();
                    //     break;
                    // case 'C':
                    //     sched = new CLOOK();
                    //     break;
                    // case 'F':
                    //     sched = new FLOOK();
                    //     break;
                    default:
                        std::cerr << "Invalid scheduler algorithm specified."
                                  << std::endl;
                        exit(1);
                }
                break;
            case 'v':
                break;
            case 'q':
                break;
            case 'f':
                break;
            default:
                std::cerr << "Invalid option specified." << std::endl;
                exit(1);
        }
    }

    if (optind >= argc) {
        std::cerr << "No input file specified." << std::endl;
        exit(1);
    }

    std::string inputfile = argv[optind];

    read_input_file(inputfile);
}
