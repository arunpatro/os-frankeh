from collections import deque
from enum import Enum
import heapq
from collections import deque
import bisect
import bisect


        

maxprio = 4
current_running_process = None

# final stats
last_process_finish_time = float('-inf')
cpu_time = 0
io_time = 0



class process_state(Enum):
    CREATED = 1
    READY = 2
    RUNNG = 3
    BLOCK = 4

    
class process_transition(Enum):
    CREATED_TO_READY = 1
    READY_TO_RUNNG = 2
    RUNNG_TO_BLOCK = 3
    BLOCK_TO_READY = 4
    RUNNG_TO_READY = 5
    DONE = 6

class PriorityQueue:
    def __init__(self):
        self.queue = deque()
    
    def isEmpty(self):
        return len(self.queue) == 0
    
    def insert(self, data):
        key, *value = data
        insert_at = bisect.bisect_right([x[0] for x in self.queue], key)
        self.queue.insert(insert_at, data)
        
    def pop(self):
        if self.isEmpty():
            return None
        
        return self.queue.popleft()

class Event:
    def __init__(self, time, pid, time_in_state, transition):
        self.time = time
        self.pid = pid
        self.time_in_state = time_in_state
        self.transition = transition

class Process:
    def __init__(self, id, at, tc, cb, io, rand_generator):
        self.id = id
        
        self.at = at
        self.tc = tc
        self.cb = cb
        self.io = io
        
        self.static_priority = rand_generator.next(maxprio)
        self.dynamic_priority = self.static_priority - 1
        self.remaining_time = tc
        self.finish_time = None
        self.turnaround_time = None
        self.io_time = 0
        self.cw = 0
        

class DES:
    def __init__(self, process_arr):
        self.process_arr = process_arr
        self.event_queue = PriorityQueue()
        
        for process in process_arr:
            event = (process.at, process.id, 0, process_transition.CREATED_TO_READY)
            self.event_queue.insert(event)
        
    def next_event(self):
        return self.event_queue.pop()
    

class Scheduler:
    def __init__(self, name):
        self.name = name
        self.runQ = []
    
    def schedule(self):
        # get the next process from the runQ
        pid = self.runQ.pop(0)
        
    
class RandGenerator:
    def __init__(self, filename):
        self.random_numbers = get_random_numbers(filename)
        self.rand_index = 0
    
    def next(self, burst):
        rand = 1 + (self.random_numbers[self.rand_index] % burst)
        # increment the index
        self.rand_index = (self.rand_index + 1) % len(self.random_numbers)
        return rand
    


def get_random_numbers(filename:str) -> list[int]:
    random_numbers = []
    with open(filename) as f:
        random_numbers = f.readlines()
        
    randos = [int(i) for i in random_numbers[1:]]
    return randos

def get_process_array(filename:str, rand_generator:RandGenerator) -> deque[Process]:
    process_array = []
    with open(filename) as f:
        pid = 0
        for line in f:
            at, tc, cb, io = [int(i) for i in line.split()]
            process_array.append(Process(pid, at, tc, cb, io, rand_generator))
            pid+=1
    return process_array

def print_summary(scheduler, process_arr):
    global last_process_finish_time, cpu_time, io_time
    total_cpu_time = sum(p.tc - p.remaining_time for p in process_arr if p.finish_time is not None)
    total_elapsed_time = last_process_finish_time - process_arr[0].at
    cpu_util = total_cpu_time / total_elapsed_time * 100

    io_util = io_time / last_process_finish_time * 100.
    avg_tat = sum([p.turnaround_time for p in process_arr]) / len(process_arr)
    avg_cw = sum([p.cw for p in process_arr]) / len(process_arr)
    throughput = len(process_arr) / last_process_finish_time * 100.
    
    print(scheduler.name)
    for process in process_arr:
        print(f"{process.id:04d}:\t{process.at}\t{process.tc}\t{process.cb}\t{process.io}\t{process.static_priority} | {process.finish_time}\t{process.turnaround_time}\t{process.io_time}\t{process.cw}")
        
    print(f"SUM: {last_process_finish_time} {cpu_util:.2f} {io_util:.2f} {avg_tat:.2f} {avg_cw:.2f} {throughput:.3f}")
    
    
def simulation(des, rand_generator, process_arr, scheduler):
    global last_process_finish_time, cpu_time, io_time
    while (event := des.next_event()):
        clock, pid, time_in_state, transition = event
        remaining_time = process_arr[pid].remaining_time
        
        match transition:
            case process_transition.CREATED_TO_READY:
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                next_event_for_this_pid = (clock, pid, 0, process_transition.READY_TO_RUNNG)
                process_arr[pid].cw += time_in_state
                # heapq.heappush(des.event_queue, next_event_for_this_pid)
                des.event_queue.insert(next_event_for_this_pid)
                
            case process_transition.READY_TO_RUNNG:
                cpuburst = rand_generator.next(process_arr[pid].cb)
                cpuburst = min(cpuburst, process_arr[pid].remaining_time)
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')} cb={cpuburst} rem={remaining_time} prio={process_arr[pid].dynamic_priority}")
                
                ## if the cpuburst is equal to the remaining time, then the process is done
                if (cpuburst == process_arr[pid].remaining_time):
                    next_event_for_this_pid = (clock + cpuburst, pid, cpuburst, process_transition.DONE)
                else:
                    next_event_for_this_pid = (clock + cpuburst, pid, cpuburst, process_transition.RUNNG_TO_BLOCK)
                process_arr[pid].remaining_time -= cpuburst
                
                des.event_queue.insert(next_event_for_this_pid)

            case process_transition.RUNNG_TO_BLOCK:
                # accumulate the cpu time
                cpu_time += time_in_state
                ## choose a random number between 1 and io or if io is greater than time left, choose time left
                ioburst = rand_generator.next(process_arr[pid].io)
                
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')} ib={ioburst} rem={remaining_time}")
                next_event_for_this_pid = (clock + ioburst, pid, ioburst, process_transition.BLOCK_TO_READY)
                # heapq.heappush(des.event_queue, next_event_for_this_pid)
                des.event_queue.insert(next_event_for_this_pid)
                
            case process_transition.BLOCK_TO_READY:
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                
                # accumulate the io time
                process_arr[pid].io_time += time_in_state
                io_time += time_in_state
                
                next_event_for_this_pid = (clock, pid, 0, process_transition.READY_TO_RUNNG)
                # heapq.heappush(des.event_queue, next_event_for_this_pid)
                des.event_queue.insert(next_event_for_this_pid)
            case process_transition.RUNNG_TO_READY:
                pass
            case process_transition.DONE:
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                process_arr[pid].finish_time = clock
                process_arr[pid].turnaround_time = clock - process_arr[pid].at
                last_process_finish_time = max(last_process_finish_time, clock)
                print(f"DONE {last_process_finish_time} {clock}")

                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                process_arr[pid].finish_time = clock
                process_arr[pid].turnaround_time = clock - process_arr[pid].at
                last_process_finish_time = max(last_process_finish_time, clock)
                print(f"DONE {last_process_finish_time} {clock}")

                
def main(args):
    rand_generator = RandGenerator(args.rfile)
    process_arr = get_process_array(args.inputfile, rand_generator)

    des = DES(process_arr)
    scheduler = Scheduler("FCFS")

    simulation(des, rand_generator, process_arr, scheduler)
    print_summary(scheduler, process_arr)
        

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description='Process some strings.')
    parser.add_argument('--inputfile', type=str, default="lab2_assign/input2", help='Process array input file')
    parser.add_argument('--rfile', type=str, default="lab2_assign/rfile", help='random number file')
    args = parser.parse_args()
    
    main(args)