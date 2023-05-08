#include <unistd.h>

#include <fstream>
#include <iostream>
#include <list>
#include <sstream>
#include <string>
#include <vector>
#include <limits>
#include <algorithm>

struct io_operation {
    int arr_time;
    int track;
    int start_time;
    int completed_time;
    io_operation(int arr_time, int track)
        : arr_time(arr_time),
          track(track),
          start_time(-1),
          completed_time(-1) {}
};

class Scheduler {
   public:
    std::list<int> io_queue;
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
    std::string line;
    while (std::getline(file, line)) {
        if (line.empty() || line[0] == '#') {
            continue;
        }

        std::istringstream iss(line);
        int arr_time, track;
        iss >> arr_time >> track;
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
    void add(int io_index) override { io_queue.push_back(io_index); }
    int next() override {
        if (!io_queue.empty()) {
            int next_io = io_queue.front();
            io_queue.pop_front();
            return next_io;
        } else {
            return -1;
        }
    }
    bool is_empty() override { return io_queue.empty(); }
};

class SSTF : public Scheduler {
   public:
    SSTF() {}
    ~SSTF() {}
    void add(int io_index) override { io_queue.push_back(io_index); }
    int next() override {
        if (!io_queue.empty()) {
            int min_distance = 1000000;
            int min_io = -1;
            auto min_it = io_queue.begin();
            for (auto it = io_queue.begin(); it != io_queue.end(); ++it) {
                int distance = abs(io_operations[*it].track - track_head);
                if (distance < min_distance) {
                    min_distance = distance;
                    min_io = *it;
                    min_it = it;
                }
            }
            io_queue.erase(min_it);
            return min_io;
        } else {
            return -1;
        }
    }
    bool is_empty() override { return io_queue.empty(); }
};

class LOOK : public Scheduler {
   public:
    int direction = 1;
    LOOK() {}
    ~LOOK() {}
    void add(int io_index) override { io_queue.push_back(io_index); }
    int next() override {
        if (!io_queue.empty()) {
            int min_distance = std::numeric_limits<int>::max();
            int min_distance_index = -1;
            for (int i = 0; i < io_queue.size(); ++i) {
                auto it = std::next(io_queue.begin(), i);
                int io = *it;
                if (direction == 1 && io_operations[io].track < track_head) {
                    continue;
                } else if (direction == -1 &&
                           io_operations[io].track > track_head) {
                    continue;
                }
                int distance = std::abs(io_operations[io].track - track_head);
                if (distance < min_distance) {
                    min_distance = distance;
                    min_distance_index = i;
                }
            }
            if (min_distance_index == -1) {
                direction = -direction;
                return next();
            }
            auto it = std::next(io_queue.begin(), min_distance_index);
            int io = *it;
            io_queue.erase(it);
            return io;
        } else {
            return -1;
        }
    }
    bool is_empty() override { return io_queue.empty(); }

   private:
    std::list<int> io_queue;
};

class CLOOK : public Scheduler {
   public:
    CLOOK() {}
    ~CLOOK() {}
    void add(int io_index) override { io_queue.push_back(io_index); }
    int next() override {
        if (!io_queue.empty()) {
            std::vector<int> distances;
            for (const auto &io : io_queue) {
                int distance = io_operations[io].track - track_head;
                distances.push_back(distance);
            }

            auto smallest_positive_index =
                std::min_element(distances.begin(), distances.end(),
                                 [](const int &a, const int &b) {
                                     return a >= 0 && (b < 0 || a < b);
                                 });
            if (*smallest_positive_index >= 0) {
                auto it = io_queue.begin();
                std::advance(it, smallest_positive_index - distances.begin());
                int io = *it;
                io_queue.erase(it);
                return io;
            } else {
                auto smallest_index =
                    std::min_element(distances.begin(), distances.end());
                auto it = io_queue.begin();
                std::advance(it, smallest_index - distances.begin());
                int io = *it;
                io_queue.erase(it);
                return io;
            }
        } else {
            return -1;
        }
    }
    bool is_empty() override { return io_queue.empty(); }

   private:
    std::list<int> io_queue;
};

class FLOOK : public Scheduler {
   public:
    int direction = 1;
    std::list<int> add_queue;

    FLOOK() {}
    ~FLOOK() {}
    void add(int io_index) override { add_queue.push_back(io_index); }
    int next() override {
        if (io_queue.empty()) {
            io_queue.swap(add_queue);
        }

        if (!io_queue.empty()) {
            int min_distance = std::numeric_limits<int>::max();
            int min_distance_index = -1;
            for (int i = 0; i < io_queue.size(); ++i) {
                auto it = std::next(io_queue.begin(), i);
                int io = *it;
                if (direction == 1 && io_operations[io].track < track_head) {
                    continue;
                } else if (direction == -1 &&
                           io_operations[io].track > track_head) {
                    continue;
                }
                int distance = std::abs(io_operations[io].track - track_head);
                if (distance < min_distance) {
                    min_distance = distance;
                    min_distance_index = i;
                }
            }
            if (min_distance_index == -1) {
                direction = -direction;
                return next();
            }
            auto it = std::next(io_queue.begin(), min_distance_index);
            int io = *it;
            io_queue.erase(it);
            return io;
        } else {
            return -1;
        }
    }
    bool is_empty() override { return io_queue.empty(); }

   private:
    std::list<int> io_queue;
};

void simulation() {
    int io_ptr = 0;  // index to the next IO request to be processed
    while (true) {
        // add new io to the scheduler
        if (io_ptr < io_operations.size()) {
            if (io_operations[io_ptr].arr_time == simul_time) {
                sched->add(io_ptr);
                io_ptr++;
            }
        }

        // check active io for completion
        if (active_io >= 0) {
            if (io_operations[active_io].track == track_head) {
                io_operations[active_io].completed_time = simul_time;
                active_io = -1;
            }
        }

        // check if there is any IO request to be processed
        while (active_io < 0) {
            // get the next IO request from the scheduler
            int next_io = sched->next();
            if (next_io >= 0) {
                io_operations[next_io].start_time = simul_time;
                active_io = next_io;

                if (io_operations[next_io].track == track_head) {
                    io_operations[next_io].completed_time = simul_time;
                    active_io = -1;
                }
            } else if (io_ptr >= io_operations.size()) {
                return;
            } else {
                break;
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
                    case 'S':
                        sched = new SSTF();
                        break;
                    case 'L':
                        sched = new LOOK();
                        break;
                    case 'C':
                        sched = new CLOOK();
                        break;
                    case 'F':
                        sched = new FLOOK();
                        break;
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
    print_summary();
}
