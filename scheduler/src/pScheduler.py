from collections import deque
from enum import Enum
import random
import heapq

current_running_process = None

# final stats
last_process_finish_time = float('-inf')
cpu_time = 0
io_time = 0
class process_state(Enum):
    CREATED = 1
    READY = 2
    RUNNING = 3
    BLOCKED = 4
    
class process_transition(Enum):
    CREATED_TO_READY = 1
    READY_TO_RUNNING = 2
    RUNNING_TO_BLOCKED = 3
    BLOCKED_TO_READY = 4
    RUNNING_TO_READY = 5
    DONE = 6

class Process:
    def __init__(self, id, at, tc, cb, io):
        self.id = id
        
        self.at = at
        self.tc = tc
        self.cb = cb
        self.io = io
        
        self.prio = None
        self.remaining_time = tc
        self.finish_time = None
        self.turnaround_time = None
        self.io_time = 0
        self.cw = 0
def get_random_numbers(filename:str) -> list[int]:
    random_numbers = []
    with open(filename) as f:
        random_numbers = f.readlines()
        
    randos = [int(i) for i in random_numbers[1:]]
    return randos

def get_process_array(filename:str) -> deque[Process]:
    process_array = []
    with open(filename) as f:
        pid = 0
        for line in f:
            at, tc, cb, io = [int(i) for i in line.split()]
            process_array.append(Process(pid, at, tc, cb, io))
            pid+=1
    return process_array

def print_summary(scheduler, process_arr):
    global last_process_finish_time, cpu_time, io_time
    cpu_util = cpu_time / last_process_finish_time * 100.
    io_util = io_time / last_process_finish_time * 100.
    avg_tat = sum([p.turnaround_time for p in process_arr]) / len(process_arr)
    avg_cw = sum([p.cw for p in process_arr]) / len(process_arr)
    throughput = len(process_arr) / last_process_finish_time * 100.
    
    print(scheduler.name)
    for process in process_arr:
        print(f"{process.id:04d}:\t{process.at}\t{process.tc}\t{process.cb}\t{process.io}\t{process.prio} | {process.finish_time}\t{process.turnaround_time}\t{process.io_time}\t{process.cw}")
        
    print(f"SUM: {last_process_finish_time} {cpu_util:.2f} {io_util:.2f} {avg_tat:.2f} {avg_cw:.2f} {throughput:.3f}")

class Scheduler:
    def __init__(self, name):
        self.name = name
        self.runQ = []
    
    def schedule(self):
        # get the next process from the runQ
        pid = self.runQ.pop(0)
        
    
class DES:
    def __init__(self, process_arr):
        self.process_arr = process_arr
        self.event_queue = []
        for process in process_arr:
            event = (process.at, process.id, 0, process_transition.CREATED_TO_READY)
            heapq.heappush(self.event_queue, event)
        
    def next_event(self):
        if len(self.event_queue) == 0:
            return None

        return heapq.heappop(self.event_queue)
    
    
class RandGenerator:
    def __init__(self, filename):
        self.random_numbers = get_random_numbers(filename)
        self.rand_index = 0
    
    def next(self):
        rand = self.random_numbers[self.rand_index]
        # increment the index
        self.rand_index = (self.rand_index + 1) #% len(self.random_numbers)
        return rand

def simulation(des, rand_generator, process_arr, scheduler):
    global last_process_finish_time, cpu_time, io_time
    while (event := des.next_event()):
        clock, pid, time_in_state, transition = event
        remaining_time = process_arr[pid].remaining_time
        
        match transition:
            case process_transition.CREATED_TO_READY:
                # print("Transition is created to ready")
                # scheduler.runQ.append(pid)
                # call_scheduler = True
                # scheduler.schedule()
                print(f"{clock} {pid} {time_in_state}: {transition.name}")
                next_event_for_this_pid = (clock, pid, 0, process_transition.READY_TO_RUNNING)
                process_arr[pid].cw += time_in_state
                heapq.heappush(des.event_queue, next_event_for_this_pid)
                # scheduler.runQ.append(pid)
                # scheduler.schedule()
            case process_transition.READY_TO_RUNNING:
                # print("Transition is ready to running")
                ## make the first process in the ready queue the running process
                # running_pid = ready_queue.pop(0)
                ## choose a random number between 1 and cb or if cb is greater than time left, choose time left
                # cpuburst = min(process_arr[running_pid].time_left, )
                cpuburst = 1 + (rand_generator.next() % int(process_arr[pid].cb))
                cpuburst = min(cpuburst, process_arr[pid].remaining_time)
                print(f"{clock} {pid} {time_in_state}: {transition.name} cb={cpuburst} rem={remaining_time}")
                
                ## if the cpuburst is equal to the remaining time, then the process is done
                if (cpuburst == process_arr[pid].remaining_time):
                    next_event_for_this_pid = (clock + cpuburst, pid, cpuburst, process_transition.DONE)
                else:
                    next_event_for_this_pid = (clock + cpuburst, pid, cpuburst, process_transition.RUNNING_TO_BLOCKED)
                process_arr[pid].remaining_time -= cpuburst
                
                heapq.heappush(des.event_queue, next_event_for_this_pid)
            case process_transition.RUNNING_TO_BLOCKED:
                # accumulate the cpu time
                cpu_time += time_in_state
                ## choose a random number between 1 and io or if io is greater than time left, choose time left
                ioburst = 1 + (rand_generator.next() % int(process_arr[pid].io))
                
                print(f"{clock} {pid} {time_in_state}: {transition.name} io={ioburst} rem={remaining_time}")
                next_event_for_this_pid = (clock + ioburst, pid, ioburst, process_transition.BLOCKED_TO_READY)
                heapq.heappush(des.event_queue, next_event_for_this_pid)
            case process_transition.BLOCKED_TO_READY:
                # scheduler.runQ.append(pid)
                print(f"{clock} {pid} {time_in_state}: {transition.name}")
                
                # store the io time
                process_arr[pid].io_time += time_in_state
                io_time += time_in_state
                
                next_event_for_this_pid = (clock, pid, 0, process_transition.READY_TO_RUNNING)
                heapq.heappush(des.event_queue, next_event_for_this_pid)
            case process_transition.RUNNING_TO_READY:
                print("Transition is running to ready")
            case process_transition.DONE:
                print(f"{clock} {pid} {time_in_state}: {transition.name}")
                process_arr[pid].finish_time = clock
                process_arr[pid].turnaround_time = clock - process_arr[pid].at
                last_process_finish_time = max(last_process_finish_time, clock)
                print(f"DONE {last_process_finish_time} {clock}")
# def main():
process_arr = get_process_array("../lab2_assign/input4")
rand_generator = RandGenerator("../lab2_assign/rfile")

des = DES(process_arr)
scheduler = Scheduler("FCFS")

simulation(des, rand_generator, process_arr, scheduler)
print_summary(scheduler, process_arr)