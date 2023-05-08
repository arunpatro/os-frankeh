#include <unistd.h>

#include <fstream>
#include <iostream>
#include <queue>
#include <sstream>
#include <string>
#include <vector>

struct io_operation {
    int arr_time;
    int track;
    int start_time;
    int completed_time;
    io_operation(int arr_time, int track)
        : arr_time(arr_time), track(track), start_time(-1), completed_time(-1) {}
};

class Scheduler {
   public:
    std::queue<int> io_queue;
    Scheduler() {}
    virtual ~Scheduler() {}
    virtual void add(int io_index) = 0;
    virtual int next() = 0;
    virtual bool is_empty() = 0;
};


// globals
std::vector<io_operation> io_operations;
int track_head = 0;
int active_io = -1;
int simul_time = 0;
Scheduler *sched;

void read_input_file(const std::string &filename) {
    std::ifstream file(filename);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open file");
    }

    std::string line;
    while (std::getline(file, line)) {
        if (line.empty() || line[0] == '#') {
            continue;
        }

        std::istringstream iss(line);
        int arr_time, track;
        if (!(iss >> arr_time >> track)) {
            throw std::runtime_error("Failed to parse time and track");
        }

        io_operations.push_back(io_operation(arr_time, track));
    }
}

void print_summary() {
    int total_movement = 0;
    double total_turnaround_time = 0.0;
    double total_wait_time = 0.0;
    int max_wait_time = 0;

    for (size_t i = 0; i < io_operations.size(); ++i) {
        const io_operation &io = io_operations[i];
        printf("%5zu: %5d %5d %5d\n", i, io.arr_time, io.start_time,
               io.completed_time);

        int movement = io.completed_time - io.start_time;
        int turnaround_time = io.completed_time - io.arr_time;
        int wait_time = io.start_time - io.arr_time;

        total_movement += movement;
        total_turnaround_time += turnaround_time;
        total_wait_time += wait_time;

        if (wait_time > max_wait_time) {
            max_wait_time = wait_time;
        }
    }

    double num_requests = static_cast<double>(io_operations.size());

    double io_utilization = static_cast<double>(total_movement) / simul_time;
    double avg_turnaround_time = total_turnaround_time / num_requests;
    double avg_wait_time = total_wait_time / num_requests;

    printf("SUM: %d %d %.4f %.2f %.2f %d\n", simul_time, total_movement,
           io_utilization, avg_turnaround_time, avg_wait_time, max_wait_time);
}


class FIFO : public Scheduler {
   public:
    FIFO() {}
    ~FIFO() {}
    void add(int io_index) override { io_queue.push(io_index); }
    int next() override {
        if (!io_queue.empty()) {
            int next_io = io_queue.front();
            io_queue.pop();
            return next_io;
        } else {
            return -1;
        }
    }
    bool is_empty() override { return true; }
};

void simulation() {
    int io_ptr = 0;  // index to the next IO request to be processed
    while (true) {
        printf("TRACE: %d\n", simul_time);
        // add new io to the scheduler
        if (io_ptr < io_operations.size()) {
            if (io_operations[io_ptr].arr_time == simul_time) {
                sched->add(io_ptr);
                io_ptr++;
            }
        }

        // check active io for completion
        if (active_io > 0) {
            if (io_operations[active_io].track == track_head) {
                io_operations[active_io].completed_time = simul_time;
                active_io = -1;
            }
        }

        // check if there is any IO request to be processed
        while (active_io < 0) {
            // get the next IO request from the scheduler
            int next_io = sched->next();
            if (next_io < 0) {
                break;
            } else if (io_ptr >= io_operations.size()) {
                return;
            }

            io_operations[next_io].start_time = simul_time;
            active_io = next_io;

            if (io_operations[next_io].track == track_head) {
                io_operations[next_io].completed_time = simul_time;
                active_io = -1;
            }
        }

        // if io is active, move the head towards the track
        if (active_io >= 0) {
            if (io_operations[active_io].track > track_head) {
                track_head++;
            } else if (io_operations[active_io].track < track_head) {
                track_head--;
            }
        }

        // update the time
        simul_time++;
    }
}

int main(int argc, char *argv[]) {
    int c;
    char alg;
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
    simulation();
    // print_summary();
}
