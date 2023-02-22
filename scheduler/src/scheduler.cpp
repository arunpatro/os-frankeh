
// need to make the following schedulers:
// 1. FCFS
// 2. LCFS
// 3. SRTF
// 4. RR
// 5. PRIO
// 6. PREPRIO

// other notes
// - implment queues of process pointer to avoid pass by value

#include <iostream>
#include <string>
#include <vector>
#include <fstream>
#include <sstream>
#include <cstring>

class Process {
   public:
    std::string name;
    int id;
    int at, tc, cb, io;
};



enum class process_transition { 
    CREATED_TO_READY, 
    READY_TO_RUNNING,
    RUNNING_TO_BLOCKED,
    BLOCKED_TO_READY,
    RUNNING_TO_READY,
};

enum class process_state { 
    CREATED, 
    READY, 
    RUNNING, 
    BLOCKED, 
};

class Scheduler {
   private:
    std::vector<Process> eventQ;
    std::vector<Process> runQ;

   public:
    std::string name;

    void add_process(Process p);
    Process get_next_process();
    bool does_preempt();
};

std::vector<Process> build_process_array(std::string filename) {
    int id = 0;
    std::ifstream infile(filename);
    std::string line;
    std::vector<Process> process_array;
    while (std::getline(infile, line)) {
        Process p;
        std::istringstream iss(line);
        iss >> p.at >> p.tc >> p.cb >> p.io;

        p.id = id++;
        p.name = "P" + std::to_string(p.id);

        process_array.push_back(p);
    }


    return process_array;
}

int main() {
    // read input file and load into process_array
    std::vector<Process> process_array = build_process_array("../lab2_assign/input1");

    // Print the loaded processes
    for (auto p : process_array) {
        std::cout << p.name << ": " << p.at << " " << p.tc << " " << p.cb << " " << p.io << std::endl;
    }

    return 0;
}