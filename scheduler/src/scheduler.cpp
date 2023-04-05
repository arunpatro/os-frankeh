#include <unistd.h>

#include <algorithm>
#include <cstdio>
// #include <cstring>
#include <fstream>
#include <list>
#include <sstream>
// #include <string>
#include <vector>

int verbose_mode = 0;

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
          current_state_start_time(at),
          preempted(false) {}
};

class RandGenerator {
   private:
    std::vector<int> random_numbers;

   public:
    int rand_index = 0;
    RandGenerator(const std::string& filename) {
        std::ifstream infile(filename);
        int number;
        infile >> number;  // first number is the number of random numbers
        random_numbers.reserve(number);
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
   public:
    std::vector<Process*> process_array;
    std::list<Event*> eventQ;
    int total_io_time = 0;

    DES(std::vector<Process*> process_array) {
        this->process_array = process_array;
        for (auto p : process_array) {
            add_event(
                new Event(p->at, p, process_transition::CREATED_TO_READY));
        }
    }

    void add_event(Event* e) {
        if (eventQ.empty()) {
            eventQ.push_back(e);
            return;
        }

        auto it = eventQ.begin();
        while (it != eventQ.end()) {
            if ((*it)->clock > e->clock) {
                eventQ.insert(it, e);
                return;
            }
            it++;
        }
        eventQ.push_back(e);
    }

    Event next_event() {
        Event e = *eventQ.front();
        eventQ.pop_front();
        return e;
    }

    void delete_event(Process* p) {
        auto it = eventQ.begin();
        while (it != eventQ.end()) {
            if ((*it)->p == p) {
                eventQ.erase(it);
                return;
            }
            it++;
        }
    }

    int next_event_time() {
        if (!eventQ.empty()) {
            return eventQ.front()->clock;
        }
        return -1;
    }

    int next_event_time_for_proc(Process* p) {
        auto it = eventQ.begin();
        while (it != eventQ.end()) {
            if ((*it)->p == p) {
                return (*it)->clock;
            }
            it++;
        }
        return -1;
    }
    bool empty() { return eventQ.empty(); }
};

class Scheduler {
   public:
    std::string name;
    int quantum;
    int maxprio;
    std::list<Process*> runQ;

    Scheduler() {
        name = "FCFS";
        quantum = (int)1e4;
        maxprio = 4;
    }

    virtual void add_process(Process* p) {
        p->dynamic_prio = p->static_prio - 1;
        runQ.push_back(p);
    }

    virtual Process* get_next_process() {
        if (!runQ.empty()) {
            Process* p = runQ.front();
            runQ.pop_front();
            return p;
        }
        return nullptr;
    }

    virtual bool does_preempt() { return false; }
};

class FCFS : public Scheduler {};

class LCFS : public Scheduler {
   public:
    LCFS() { name = "LCFS"; }

    Process* get_next_process() {
        if (!runQ.empty()) {
            Process* p = runQ.back();
            runQ.pop_back();
            return p;
        }
        return nullptr;
    }
};

class SRTF : public Scheduler {
   public:
    SRTF() { name = "SRTF"; }

    Process* get_next_process() {
        if (!runQ.empty()) {
            // Find the process with the smallest remaining time
            auto shortest_process = std::min_element(
                runQ.begin(), runQ.end(),
                [](const Process* p1, const Process* p2) {
                    return p1->remaining_time < p2->remaining_time;
                });

            Process* p = *shortest_process;
            runQ.erase(shortest_process);
            return p;
        }

        return nullptr;
    }
};

class RR : public Scheduler {
   public:
    RR(int quantum) {
        name = "RR";
        this->quantum = quantum;
    }
};

class PRIO : public Scheduler {
   public:
    std::vector<std::list<Process*>> activeQ;
    std::vector<std::list<Process*>> expiredQ;
    PRIO(int quantum, int maxprio = 4) {
        name = "PRIO";
        this->quantum = quantum;
        this->maxprio = maxprio;
        activeQ.resize(maxprio);
        expiredQ.resize(maxprio);
    }

    void add_process(Process* p) {
        if (p->dynamic_prio < 0) {
            p->dynamic_prio = p->static_prio - 1;
            expiredQ[p->dynamic_prio].push_back(p);
        } else {
            activeQ[p->dynamic_prio].push_back(p);
        }
    }

    Process* get_next_process() {
        if (std::all_of(activeQ.begin(), activeQ.end(),
                        [](std::list<Process*> q) { return q.empty(); }) &&
            std::all_of(expiredQ.begin(), expiredQ.end(),
                        [](std::list<Process*> q) { return q.empty(); })) {
            return nullptr;
        }

        if (std::all_of(activeQ.begin(), activeQ.end(),
                        [](std::list<Process*> q) { return q.empty(); })) {
            activeQ.swap(expiredQ);
        }

        for (auto it = activeQ.rbegin(); it != activeQ.rend(); ++it) {
            if (!it->empty()) {
                auto nextProcess = it->front();
                it->pop_front();
                return nextProcess;
            }
        }

        return nullptr;
    }
};

class PREPRIO : public PRIO {
   public:
    PREPRIO(int quantum, int maxprio = 4) : PRIO(quantum, maxprio) {
        name = "PREPRIO";
    }

    bool does_preempt() { return true; }
};

std::vector<Process*> create_process_array(std::string filename,
                                           RandGenerator* rand_generator,
                                           int maxprio) {
    int pid = 0;
    int at, tc, cb, io, sprio;
    std::ifstream infile(filename);
    std::string line;
    std::vector<Process*> process_array;
    while (std::getline(infile, line)) {
        std::istringstream iss(line);
        iss >> at >> tc >> cb >> io;

        sprio = rand_generator->next(maxprio);
        Process* p = new Process(pid++, at, tc, cb, io, sprio);
        process_array.push_back(p);
    }

    return process_array;
}

void simulation_loop(DES* des, Scheduler* scheduler,
                     RandGenerator* rand_generator) {
    Process* current_running_process = NULL;
    bool call_scheduler = false;
    int cpuburst, ioburst;
    int n_io_blocked = 0;
    int io_start_time;

    while (!des->empty()) {
        Event e = des->next_event();
        auto time_in_state = e.clock - e.p->current_state_start_time;
        switch (e.transition) {
            case process_transition::CREATED_TO_READY:
                if (verbose_mode) {
                    printf("%d %d %d: CREATED -> READY\n", e.clock, e.p->id,
                           time_in_state);
                }
                e.p->state = process_state::READY;
                e.p->current_state_start_time = e.clock;

                if ((current_running_process != NULL) &
                    scheduler->does_preempt()) {
                    bool cond1 = e.p->dynamic_prio >
                                 current_running_process->dynamic_prio;
                    int next_time_for_curr_proc =
                        des->next_event_time_for_proc(current_running_process);
                    bool cond2 = next_time_for_curr_proc != e.clock;
                    if (cond1 && cond2) {
                        des->delete_event(current_running_process);
                        current_running_process->preempted = true;
                        des->add_event(
                            new Event(e.clock, current_running_process,
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

                if (verbose_mode) {
                    printf("%d %d %d: READY -> RUNNG cb=%d rem=%d prio=%d\n",
                           e.clock, e.p->id, time_in_state, cpuburst,
                           e.p->remaining_time, e.p->dynamic_prio);
                }
                e.p->state = process_state::RUNNING;
                e.p->current_state_start_time = e.clock;
                current_running_process = e.p;

                e.p->waiting_time += time_in_state;
                e.p->preempted = false;

                if (scheduler->quantum < cpuburst) {
                    des->add_event(
                        new Event(e.clock + scheduler->quantum, e.p,
                                  process_transition::RUNNING_TO_READY));

                } else {
                    if (cpuburst >= e.p->remaining_time) {
                        des->add_event(
                            new Event(e.clock + cpuburst, e.p,
                                      process_transition::RUNNING_TO_DONE));
                    } else {
                        des->add_event(
                            new Event(e.clock + cpuburst, e.p,
                                      process_transition::RUNNING_TO_BLOCKED));
                    }
                }
                break;
            case process_transition::RUNNING_TO_READY:
                e.p->remaining_time -= time_in_state;
                e.p->current_burst -= time_in_state;
                current_running_process = NULL;

                if (verbose_mode) {
                    printf("%d %d %d: RUNNG -> READY cb=%d rem=%d prio=%d\n",
                           e.clock, e.p->id, time_in_state, cpuburst,
                           e.p->remaining_time, e.p->dynamic_prio);
                }
                e.p->state = process_state::READY;
                e.p->current_state_start_time = e.clock;

                e.p->dynamic_prio -= 1;
                scheduler->add_process(e.p);

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
                if (verbose_mode) {
                    printf("%d %d %d: RUNNG -> BLOCK  ib=%d rem=%d\n", e.clock,
                           e.p->id, time_in_state, ioburst,
                           e.p->remaining_time);
                }
                e.p->state = process_state::BLOCKED;
                e.p->current_state_start_time = e.clock;

                des->add_event(new Event(e.clock + ioburst, e.p,
                                         process_transition::BLOCKED_TO_READY));
                call_scheduler = true;
                break;
            case process_transition::BLOCKED_TO_READY:
                n_io_blocked -= 1;
                if (n_io_blocked == 0) {
                    des->total_io_time += e.clock - io_start_time;
                }

                e.p->dynamic_prio = e.p->static_prio - 1;
                e.p->io_time += time_in_state;
                if (verbose_mode) {
                    printf("%d %d %d: BLOCK -> READY\n", e.clock, e.p->id,
                           time_in_state);
                }
                e.p->state = process_state::READY;
                e.p->current_state_start_time = e.clock;

                if ((current_running_process != nullptr) &
                    (scheduler->does_preempt())) {
                    bool cond1 = e.p->dynamic_prio >
                                 current_running_process->dynamic_prio;

                    int next_time_for_curr_proc =
                        des->next_event_time_for_proc(current_running_process);

                    bool cond2 = next_time_for_curr_proc != e.clock;
                    if (cond1 && cond2) {
                        des->delete_event(current_running_process);
                        current_running_process->preempted = true;
                        des->add_event(
                            new Event(e.clock, current_running_process,
                                      process_transition::RUNNING_TO_READY));
                    }
                }

                scheduler->add_process(e.p);
                call_scheduler = true;
                break;
            case process_transition::RUNNING_TO_DONE:
                e.p->remaining_time -= time_in_state;
                current_running_process = NULL;
                if (verbose_mode) {
                    printf("%d %d %d: Done\n", e.clock, e.p->id, time_in_state);
                }
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
                if (current_running_process == nullptr) {
                    auto proc = scheduler->get_next_process();
                    if (proc != nullptr) {
                        des->add_event(
                            new Event(e.clock, proc,
                                      process_transition::READY_TO_RUNNING));
                    }
                }
            }
        }
    }
}

void print_summary(DES* des, Scheduler* scheduler) {
    if (scheduler->quantum < 1e4) {
        printf("%s %d\n", scheduler->name.c_str(), scheduler->quantum);
    } else {
        printf("%s\n", scheduler->name.c_str());
    }
    for (auto p : des->process_array) {
        printf("%04d: %4d %4d %4d %4d %1d | %5d %5d %5d %5d\n", p->id, p->at,
               p->tc, p->cb, p->io, p->static_prio, p->finish_time,
               p->turnaround_time, p->io_time, p->waiting_time);
    }

    int num_processes = des->process_array.size();
    int finishtime = 0;
    double cpu_time = 0;
    double total_tat = 0;
    double total_wait = 0;
    for (auto p : des->process_array) {
        cpu_time += p->tc;
        total_tat += p->turnaround_time;
        total_wait += p->waiting_time;

        finishtime = std::max(finishtime, p->finish_time);
    }

    double avg_tat = total_tat / num_processes;
    double avg_wait = total_wait / num_processes;

    double cpu_util = 100.0 * (cpu_time / (double)finishtime);
    double io_util = 100.0 * (des->total_io_time / (double)finishtime);
    double throughput = 100.0 * (num_processes / (double)finishtime);

    printf("SUM: %d %.2lf %.2lf %.2lf %.2lf %.3lf\n", finishtime, cpu_util,
           io_util, avg_tat, avg_wait, throughput);
}

int main(int argc, char** argv) {
    int opt;
    char* scheduler_option = NULL;
    char* inputfile = NULL;
    char* randomfile = NULL;

    if (argc < 4) {
        printf("Usage: %s -s <scheduler_option> <inputfile> <randomfile>\n",
               argv[0]);
        exit(EXIT_FAILURE);
    }
    while ((opt = getopt(argc, argv, "vs:")) != -1) {
        switch (opt) {
            case 'v':
                verbose_mode = 1;
                break;
            case 's':
                scheduler_option = optarg;
                break;
        }
    }

    if (!scheduler_option) {
        printf("No scheduler option provided. Exiting.\n");
        exit(EXIT_FAILURE);
    }

    inputfile = argv[optind];
    randomfile = argv[optind + 1];

    Scheduler* scheduler;
    switch (scheduler_option[0]) {
        case 'F':
            scheduler = new FCFS();
            break;
        case 'L':
            scheduler = new LCFS();
            break;
        case 'S':
            scheduler = new SRTF();
            break;
        case 'R':
            scheduler = new RR(atoi(&scheduler_option[1]));
            break;
        case 'P':
            if (strchr(scheduler_option, ':')) {
                char* quantum_str = strtok(&scheduler_option[1], ":");
                char* maxprio_str = strtok(NULL, ":");
                scheduler = new PRIO(atoi(quantum_str), atoi(maxprio_str));
            } else {
                scheduler = new PRIO(atoi(&scheduler_option[1]));
            }
            break;
        case 'E':
            if (strchr(scheduler_option, ':')) {
                char* quantum_str = strtok(&scheduler_option[1], ":");
                char* maxprio_str = strtok(NULL, ":");
                scheduler = new PREPRIO(atoi(quantum_str), atoi(maxprio_str));
            } else {
                scheduler = new PREPRIO(atoi(&scheduler_option[1]));
            }
            break;
        default:
            printf("Invalid scheduler option provided. Exiting.\n");
            exit(EXIT_FAILURE);
    }

    auto rand_generator = RandGenerator(randomfile);
    auto process_array =
        create_process_array(inputfile, &rand_generator, scheduler->maxprio);
    auto des = DES(process_array);

    simulation_loop(&des, scheduler, &rand_generator);
    print_summary(&des, scheduler);
}
