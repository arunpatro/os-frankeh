
#include <unistd.h>

#include <cstring>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

int total_io_time = 0;

enum class process_transition {
    CREATED_TO_READY,
    READY_TO_RUNNING,
    RUNNING_TO_BLOCKED,
    BLOCKED_TO_READY,
    RUNNING_TO_READY,
    RUNNING_TO_DONE
};

enum class process_state { CREATED, READY, RUNNING, BLOCKED, DONE };

class Process {
   public:
    int id;
    int at, tc, cb, io;
    int static_prio, dynamic_prio;
    int remaining_time, finish_time, turnaround_time, io_time, waiting_time;
    int current_burst;

    process_state state;
    int current_state_start_time;
    bool preempted;

    Process(int id, int at, int tc, int cb, int io, int static_prio)
        : id(id),
          at(at),
          tc(tc),
          cb(cb),
          io(io),
          static_prio(static_prio),
          dynamic_prio(static_prio - 1),
          remaining_time(tc),
          finish_time(-1),
          turnaround_time(-1),
          io_time(0),
          waiting_time(0),
          current_burst(-1),
          state(process_state::CREATED),
          current_state_start_time(-1),
          preempted(false) {}
};

class RandGenerator {
   private:
    std::vector<int> random_numbers;
    int rand_index = 0;

   public:
    RandGenerator(const std::string& filename) {
        std::ifstream infile(filename);
        int number;
        infile >> number;  // skip first number
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

struct Event {
    int clock;
    Process* p;
    process_transition transition;

    Event(int clock, Process* p, process_transition transition)
        : clock(clock), p(p), transition(transition) {}
};

class DES {
   private:
    std::vector<Process*> process_array;

   public:
    std::vector<Event> event_queue;

    DES(std::vector<Process*> process_array) {
        this->process_array = process_array;

        for (auto p : process_array) {
            Event e(p->at, p, process_transition::CREATED_TO_READY);
            event_queue.push_back(e);
        }
    }

    void add_event(Event e) { event_queue.push_back(e); }

    Event next_event() {
        Event e = event_queue[0];
        event_queue.erase(event_queue.begin());
        return e;
    }

    void delete_event(Process* p) {
        for (int i = 0; i < event_queue.size(); i++) {
            if (event_queue[i].p == p) {
                event_queue.erase(event_queue.begin() + i);
                break;
            }
        }
    }

    int next_event_time() { return event_queue[0].clock; }
    bool empty() { return event_queue.empty(); }
};

class Scheduler {
   private:
    std::vector<Process*> runQ;

   public:
    std::string name;
    int quantum;
    int maxprio;

    Scheduler() {
        name = "FCFS";
        quantum = (int)10e4;
        maxprio = 4;
    }

    void add_process(Process* p) {
        runQ.push_back(p);
        // std::cout << "Added process " << p->id << " to runQ" << std::endl;
    }
    Process* get_next_process() {
        if (runQ.size() > 0) {
            Process* p = runQ[0];
            runQ.erase(runQ.begin());
            return p;
        }
        return NULL;
    }
};

class FCFS : public Scheduler {};

// des layer that keeps track of the simulation
// class Event {
//    private:
//     int time;
//     Process *p;
//     process_transition transition;

//    public:
//     Event(int time, Process p, process_transition transition);
// };

// // des emits events
// class DES {
//    private:
//     std::vector<Process> process_array;
//     int time = 0;

//    public:
//     DES(std::vector<Process> process_array);
// };

std::vector<Process*> create_process_array(std::string filename,
                                           RandGenerator rand_generator,
                                           int maxprio) {
    int pid = 0;
    int at, tc, cb, io, sprio;
    std::ifstream infile(filename);
    std::string line;
    std::vector<Process*> process_array;
    while (std::getline(infile, line)) {
        std::istringstream iss(line);
        iss >> at >> tc >> cb >> io;

        sprio = rand_generator.next(maxprio);
        Process* p = new Process(pid++, at, tc, cb, io, sprio);
        process_array.push_back(p);
    }

    return process_array;
}

void simulation_loop(DES* des, std::vector<Process*> process_array,
                     Scheduler* scheduler, RandGenerator* rand_generator) {
    // std::cout << "Starting simulation loop" << std::endl;
    // std::cout << "Printing the process array table" << std::endl;
    // std::cout << "Process ID\tArrival Time\tTotal CPU Time\tCPU Burst\tIO
    // Burst"
    //           << std::endl;
    // for (auto p : process_array) {
    //     std::cout << p->id << "\t\t" << p->at << "\t\t" << p->tc << "\t\t"
    //               << p->cb << "\t\t" << p->io << std::endl;
    // }

    Process* current_running_process = NULL;
    bool call_scheduler = false;
    int cpuburst, ioburst, n_io_blocked, io_start_time;

    while (!des->empty()) {
        Event e = des->next_event();
        // std::cout << "Event: " << e.clock << " " << e.p->id << " "
        //   << (int)e.transition << std::endl;

        auto time_in_state = e.clock - e.p->current_state_start_time;

        switch (e.transition) {
            case process_transition::CREATED_TO_READY:
                e.p->state = process_state::READY;
                e.p->current_state_start_time = e.clock;

                if ((current_running_process != NULL) &
                    (scheduler->name == "PREPRIO")) {
                    bool cond1 = e.p->dynamic_prio >
                                 current_running_process->dynamic_prio;
                    int next_time_for_curr_proc = -1;
                    for (auto event : des->event_queue) {
                        if (event.p == current_running_process) {
                            next_time_for_curr_proc = event.clock;
                            break;
                        }
                    }

                    bool cond2 = next_time_for_curr_proc != e.clock;
                    if (cond1 && cond2) {
                        des->delete_event(current_running_process);
                        current_running_process->preempted = true;
                        des->add_event(
                            Event(e.clock, current_running_process,
                                  process_transition::RUNNING_TO_READY));
                    }
                }

                scheduler->add_process(e.p);
                call_scheduler = true;
                break;
            case process_transition::READY_TO_RUNNING:
                if (e.p->preempted) {
                    cpuburst = e.p->current_burst;
                } else {
                    cpuburst = rand_generator->next(e.p->cb);
                    cpuburst = std::min(cpuburst, e.p->remaining_time);
                    e.p->current_burst = cpuburst;
                }

                e.p->state = process_state::RUNNING;
                e.p->current_state_start_time = e.clock;
                current_running_process = e.p;

                e.p->waiting_time += time_in_state;
                e.p->preempted = false;

                if (scheduler->quantum < cpuburst) {
                    des->add_event(Event(e.clock + scheduler->quantum, e.p,
                                         process_transition::RUNNING_TO_READY));

                } else {
                    if (cpuburst >= e.p->remaining_time) {
                        des->add_event(
                            Event(e.clock + cpuburst, e.p,
                                  process_transition::RUNNING_TO_DONE));
                    } else {
                        des->add_event(
                            Event(e.clock + cpuburst, e.p,
                                  process_transition::RUNNING_TO_BLOCKED));
                    }
                }
                break;
            case process_transition::RUNNING_TO_READY:
                e.p->remaining_time -= time_in_state;
                e.p->current_burst -= time_in_state;
                current_running_process = NULL;

                e.p->state = process_state::READY;
                e.p->current_state_start_time = e.clock;

                if (scheduler->name == "PREPRIO" || scheduler->name == "PRIO") {
                    e.p->dynamic_prio -= 1;
                    if (e.p->dynamic_prio < 0) {
                        e.p->dynamic_prio = e.p->static_prio - 1;
                        // TODO
                        // scheduler->expired_queues[e.p->dynamic_prio].push_back(e.p);
                    } else {
                        scheduler->add_process(e.p);
                    }
                } else {
                    scheduler->add_process(e.p);
                }

                e.p->preempted = true;
                call_scheduler = true;
                break;
            case process_transition::RUNNING_TO_BLOCKED:
                e.p->remaining_time -= time_in_state;
                current_running_process = NULL;
                ioburst = rand_generator->next(e.p->io);

                if (n_io_blocked == 0) {
                    io_start_time = e.clock;
                }

                n_io_blocked++;

                e.p->state = process_state::BLOCKED;
                e.p->current_state_start_time = e.clock;

                des->add_event(Event(e.clock + ioburst, e.p,
                                     process_transition::BLOCKED_TO_READY));
                call_scheduler = true;
                break;
            case process_transition::BLOCKED_TO_READY:
                n_io_blocked -= 1;
                if (n_io_blocked == 0) {
                    total_io_time += e.clock - io_start_time;
                }

                e.p->dynamic_prio = e.p->static_prio - 1;
                e.p->io_time += time_in_state;
                e.p->state = process_state::READY;
                e.p->current_state_start_time = e.clock;

                if ((current_running_process != NULL) &
                    (scheduler->name == "PREPRIO")) {
                    bool cond1 = e.p->dynamic_prio >
                                 current_running_process->dynamic_prio;
                    int next_time_for_curr_proc = -1;
                    for (auto event : des->event_queue) {
                        if (event.p == current_running_process) {
                            next_time_for_curr_proc = event.clock;
                            break;
                        }
                    }

                    bool cond2 = next_time_for_curr_proc != e.clock;
                    if (cond1 && cond2) {
                        des->delete_event(current_running_process);
                        current_running_process->preempted = true;
                        des->add_event(
                            Event(e.clock, current_running_process,
                                  process_transition::RUNNING_TO_READY));
                    }
                }

                scheduler->add_process(e.p);
                call_scheduler = true;
                break;
            case process_transition::RUNNING_TO_DONE:
                e.p->remaining_time -= time_in_state;
                current_running_process = NULL;
                e.p->finish_time = e.clock;
                e.p->turnaround_time = e.p->finish_time - e.p->at;
                call_scheduler = true;
                break;
        }

        if (call_scheduler) {
            if (des->next_event_time() == e.clock) {
                continue;
            } else {
                call_scheduler = false;
                if (current_running_process == NULL) {
                    auto proc = scheduler->get_next_process();
                    if (proc != NULL) {
                        des->add_event(
                            Event(e.clock, proc,
                                  process_transition::READY_TO_RUNNING));
                    }
                }
            }
        }
    }
}

void print_summary(std::vector<Process*> process_array, Scheduler* scheduler) {
    for (auto p : process_array) {
        printf("%d %d %d %d %d | %d %d %d %d %d\n", p->id, p->at, p->tc, p->cb,
               p->io, p->static_prio, p->finish_time, p->turnaround_time,
               p->io_time, p->waiting_time);
    }

    int last_process_finish_time = 0;
    float cpu_util = 0;
    float io_util = 0;
    float avg_tat = 0;
    float avg_wait = 0;
    float throughput = 0;
    for (auto p : process_array) {
        if (p->finish_time > last_process_finish_time) {
            last_process_finish_time = p->finish_time;
        }

        cpu_util += p->tc;
        avg_tat += p->turnaround_time;
        avg_wait += p->waiting_time;
    }

    cpu_util = cpu_util / last_process_finish_time * 100;
    io_util = total_io_time / last_process_finish_time * 100;
    avg_tat = avg_tat / process_array.size();
    avg_wait = avg_wait / process_array.size();
    throughput = process_array.size() * 100.0 / last_process_finish_time;

    printf("SUM: %d %.2f %.2f %.2f %.2f %.3f", last_process_finish_time,
           cpu_util, io_util, avg_tat, avg_wait, throughput);
}

int main(int argc, char** argv) {
    int opt;
    // char* scheduler_option = NULL;
    char* scheduler_option = "F";
    while ((opt = getopt(argc, argv, "s:")) != -1) {
        switch (opt) {
            case 's':
                scheduler_option = optarg;
                std::cout << "Scheduler option: " << scheduler_option
                          << std::endl;
                break;
        }
    }

    if (!scheduler_option) {
        printf("No scheduler option provided. Exiting.\n");
        exit(EXIT_FAILURE);
    }

    Scheduler* scheduler;
    switch (scheduler_option[0]) {
        case 'F':
            scheduler = new FCFS();
            break;
        // case 'L':
        //     scheduler = new LCFS();
        //     break;
        // case 'S':
        //     scheduler = new SRTF();
        //     break;
        // case 'R':
        //     scheduler = new RR(atoi(&scheduler_option[1]));
        //     break;
        // case 'P':
        //     if (strchr(scheduler_option, ':'))
        //     {
        //         char *quantum_str = strtok(&scheduler_option[1], ":");
        //         char *maxprio_str = strtok(NULL, ":");
        //         scheduler = new PRIO(atoi(quantum_str), atoi(maxprio_str));
        //     }
        //     else
        //     {
        //         scheduler = new PRIO(atoi(&scheduler_option[1]));
        //     }
        //     break;
        // case 'E':
        //     if (strchr(scheduler_option, ':'))
        //     {
        //         char *quantum_str = strtok(&scheduler_option[1], ":");
        //         char *maxprio_str = strtok(NULL, ":");
        //         scheduler = new PREPRIO(atoi(quantum_str),
        //         atoi(maxprio_str));
        //     }
        //     else
        //     {
        //         scheduler = new PREPRIO(atoi(&scheduler_option[1]));
        //     }
        //     break;
        default:
            printf("Invalid scheduler option provided. Exiting.\n");
            exit(EXIT_FAILURE);
    }
    // printf("Scheduler: %s\n", scheduler->name());

    auto rand_generator = RandGenerator("../lab2_assign/rfile");
    // for (int i = 0; i < 10; i++) {
    // std::cout << "Random number: " << rand_generator.next(10) << std::endl;
    // }

    auto process_array = create_process_array(
        "../lab2_assign/input1", rand_generator, scheduler->maxprio);

    auto des = DES(process_array);

    simulation_loop(&des, process_array, scheduler, &rand_generator);
    print_summary(process_array, scheduler);
}
