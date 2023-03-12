from collections import deque
from enum import Enum
import heapq
from collections import deque, namedtuple
import bisect
import bisect


current_running_process = None

# final stats
last_process_finish_time = float('-inf')
cpu_time = 0
total_io_time = 0
n_io_blocked = 0
io_start_time = 0


class process_state(Enum):
    CREATED = 1
    READY = 2
    RUNNG = 3
    BLOCK = 4


class process_transition(Enum):
    CREATED_TO_READY = 1
    READY_TO_RUNNG = 2
    RUNNG_TO_PREEMPT = 3
    RUNNG_TO_BLOCK = 4
    BLOCK_TO_READY = 5
    RUNNG_TO_READY = 6
    DONE = 7


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


Event = namedtuple('Event', ['time', 'pid', 'time_entered_state', 'transition'])


class Process:
    def __init__(self, id, at, tc, cb, io, rand_generator, maxprio=4):
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
        self.prempted = False
        
        self.current_cpu_burst = None


class DES:
    def __init__(self, process_arr):
        self.process_arr = process_arr
        self.event_queue = PriorityQueue()

        for process in process_arr:
            event = Event(process.at, process.id, process.at, process_transition.CREATED_TO_READY)
            self.event_queue.insert(event)

    def next_event_time(self):
        if self.event_queue.isEmpty():
            return None

        return self.event_queue.queue[0][0]

    def next_event(self):
        return self.event_queue.pop()


class Scheduler:
    def __init__(self, name, quantum=1e4, maxprio=4):
        self.name = name
        self.quantum = quantum
        self.maxprio = maxprio
        self.runQ = deque()
        
    def add_process(self, process):
        self.runQ.append(process)
        
    def get_next_process(self):
        if self.runQ:
            return self.runQ.popleft()

        return None
    
    
class FCFS(Scheduler):
    def __init__(self):
        super().__init__('FCFS')
        
    def add_process(self, process):
        self.runQ.append(process)

    def get_next_process(self):
        if self.runQ:
            return self.runQ.popleft()

        return None
        
    
    
class LCFS(Scheduler):
    def __init__(self):
        super().__init__('LCFS')
        
    def add_process(self, process):
        self.runQ.append(process)

    def get_next_process(self):
        if self.runQ:
            return self.runQ.pop()

        return None


class SRTF(Scheduler):
    def __init__(self):
        super().__init__('SRTF')
        
    def add_process(self, item):
        _, _, _, remaining_time = item
        
        insert_at = bisect.bisect_right([x[3] for x in self.runQ], remaining_time)
        self.runQ.insert(insert_at, item)

    def get_next_process(self):
        if self.runQ:
            return self.runQ.popleft()

        return None
    
class RR(Scheduler):
    def __init__(self, quantum):
        super().__init__('RR')
        self.quantum = quantum
    
    def add_process(self, process):
        self.runQ.append(process)
        
    def get_next_process(self):
        if self.runQ:
            return self.runQ.popleft()

        return None
        
class PRIO(Scheduler):
    def __init__(self, quantum, maxprio=4):
        super().__init__('PRIO', quantum, maxprio)
        self.active_queues = [PriorityQueue() for _ in range(maxprio)]
        self.expired_queues = [PriorityQueue() for _ in range(maxprio)]
    
    def add_process(self, process):
        prio = process[0] # priority
        # insert_at = bisect.bisect_right([x[0] for x in self.active_queues[prio].queue], prio)
        self.active_queues[prio].insert(process)
    
    def get_next_process(self):
        if not any(i.queue for i in self.active_queues) and not any(i.queue for i in self.expired_queues):
            return None

        if not any(i.queue for i in self.active_queues):
            self.active_queues, self.expired_queues = self.expired_queues, self.active_queues
        
        # get highest priority process in in active queue
        for queue in self.active_queues[::-1]:
            if not queue.isEmpty():
                return queue.pop()
        
    
class RandGenerator:
    def __init__(self, filename):
        self.random_numbers = get_random_numbers(filename)
        self.rand_index = 0

    def next(self, burst):
        rand = 1 + (self.random_numbers[self.rand_index] % burst)
        # increment the index
        self.rand_index = (self.rand_index + 1) % len(self.random_numbers)
        return rand


def get_random_numbers(filename: str) -> list[int]:
    random_numbers = []
    with open(filename) as f:
        random_numbers = f.readlines()

    randos = [int(i) for i in random_numbers[1:]]
    return randos


def get_process_array(filename: str, rand_generator: RandGenerator, maxprio:int) -> deque[Process]:
    process_array = []
    with open(filename) as f:
        pid = 0
        for line in f:
            at, tc, cb, io = [int(i) for i in line.split()]
            process_array.append(Process(pid, at, tc, cb, io, rand_generator, maxprio))
            pid += 1
    return process_array


def print_summary(scheduler: Scheduler, process_arr: list[Process]):
    if scheduler.quantum < 1e4:
        print(f"{scheduler.name} {scheduler.quantum}")
    else:
        print(f"{scheduler.name}")
        
    for process in process_arr:
        print(f"{process.id:04d}:\t{process.at}\t{process.tc}\t{process.cb}\t{process.io}\t{process.static_priority} | {process.finish_time}\t{process.turnaround_time}\t{process.io_time}\t{process.cw}")
    
    # assert cpu_time == sum([p.tc for p in process_arr])
    # print(cpu_time, sum([p.tc for p in process_arr]))
    cpu_util = cpu_time / last_process_finish_time * 100
    io_util = total_io_time / last_process_finish_time * 100
    avg_tat = sum([p.turnaround_time for p in process_arr]) / len(process_arr)
    avg_cw = sum([p.cw for p in process_arr]) / len(process_arr)
    throughput = len(process_arr) / last_process_finish_time * 100.

    print(f"SUM: {last_process_finish_time} {cpu_util:.2f} {io_util:.2f} {avg_tat:.2f} {avg_cw:.2f} {throughput:.3f}")


def simulation(des, rand_generator, process_arr, scheduler):
    global last_process_finish_time, cpu_time, current_running_process, n_io_blocked, io_start_time, total_io_time
    while (event := des.next_event()):
        clock, pid, time_entered_state, transition = event
        time_in_state = clock - time_entered_state
        remaining_time = process_arr[pid].remaining_time
        dyna_prio = process_arr[pid].dynamic_priority

        match transition:
            case process_transition.CREATED_TO_READY:
                # must come from BLOCKED or CREATED
                # add to run queue, no event created
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                scheduler.add_process((dyna_prio, pid, clock, remaining_time))
                call_scheduler = True

            case process_transition.READY_TO_RUNNG:
                
                # if not coming from pre-emption
                if not process_arr[pid].prempted:
                    cpuburst = rand_generator.next(process_arr[pid].cb)
                    cpuburst = min(cpuburst, process_arr[pid].remaining_time)
                    process_arr[pid].current_cpu_burst = cpuburst
                else:
                    cpuburst = process_arr[pid].current_cpu_burst
                    
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')} cb={cpuburst} rem={remaining_time} prio={process_arr[pid].dynamic_priority}")

                # increase cw time
                process_arr[pid].cw += time_in_state
                
                # since process is running, remove prememted flag
                process_arr[pid].prempted = False 
                
                # accumulate the cpu time, update the remaining time
                # check quantum and decide to run normally or fire a preemt signal
                if scheduler.quantum < cpuburst:
                    # need to set up preempt signal
                    cpu_time += scheduler.quantum
                    process_arr[pid].remaining_time -= scheduler.quantum
                    
                    next_event_for_this_pid = Event(clock + scheduler.quantum, pid, clock, process_transition.RUNNG_TO_PREEMPT)
                    des.event_queue.insert(next_event_for_this_pid)
                else:
                    # can exhaust cpuburst now, so send to block or done
                    cpu_time += cpuburst
                    
                    # create event for done or blocking
                    if (cpuburst >= process_arr[pid].remaining_time):
                        # done with this process
                        next_event_for_this_pid = Event(clock + cpuburst, pid, clock, process_transition.DONE)
                        des.event_queue.insert(next_event_for_this_pid)
                    else:
                        next_event_for_this_pid = Event(clock + cpuburst, pid, clock, process_transition.RUNNG_TO_BLOCK)
                        des.event_queue.insert(next_event_for_this_pid)
                    
                    # process_arr[pid].remaining_time -= cpuburst
                    
            case process_transition.RUNNG_TO_PREEMPT:
                # stop the current running process
                current_running_process = None
                
                remaining_time = process_arr[pid].remaining_time
                process_arr[pid].current_cpu_burst -= time_in_state
                print(f"{clock} {pid} {time_in_state}: {'RUNNG_TO_READY'.replace('_TO_', ' -> ')}  cb={process_arr[pid].current_cpu_burst} rem={remaining_time} prio={process_arr[pid].dynamic_priority}")
                
                # adjust the dynamic priority
                if scheduler.name in ["PRIO"]:
                    process_arr[pid].dynamic_priority -= 1
                    if process_arr[pid].dynamic_priority < 0:
                        process_arr[pid].dynamic_priority = process_arr[pid].static_priority - 1
                        # same as add process but insert into expired queue
                        scheduler.expired_queues[process_arr[pid].dynamic_priority].insert((process_arr[pid].dynamic_priority, pid, clock, remaining_time))
                    else:
                        scheduler.add_process((process_arr[pid].dynamic_priority, pid, clock, remaining_time))
                else:
                    scheduler.add_process((dyna_prio, pid, clock, remaining_time))
                        
                process_arr[pid].prempted = True
                call_scheduler = True

            case process_transition.RUNNG_TO_BLOCK:
                process_arr[pid].remaining_time -= time_in_state
                
                # finished running now blocking
                current_running_process = None

                # choose a random number between 1 and io or if io is greater than time left, choose time left
                ioburst = rand_generator.next(process_arr[pid].io)
                
                # update the start time of io
                if n_io_blocked == 0:
                    io_start_time = clock
                    
                n_io_blocked += 1

                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}  ib={ioburst} rem={process_arr[pid].remaining_time}")
                
                next_event_for_this_pid = Event(clock + ioburst, pid, clock, process_transition.BLOCK_TO_READY)
                des.event_queue.insert(next_event_for_this_pid)
                call_scheduler = True

            case process_transition.BLOCK_TO_READY:
                # update n_io_blocked
                n_io_blocked -= 1
                if n_io_blocked == 0:
                    total_io_time += clock - io_start_time
                    
                # reset dynamic priority
                process_arr[pid].dynamic_priority = process_arr[pid].static_priority - 1
                
                process_arr[pid].io_time += time_in_state
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                scheduler.add_process((process_arr[pid].dynamic_priority, pid, clock, remaining_time))
                call_scheduler = True

            case process_transition.DONE:
                process_arr[pid].remaining_time -= time_in_state
                current_running_process = None
                print(f"{clock} {pid} {time_in_state}: {'Done'}")
                process_arr[pid].finish_time = clock
                process_arr[pid].turnaround_time = clock - process_arr[pid].at
                last_process_finish_time = max(last_process_finish_time, clock)
                call_scheduler = True

        if (call_scheduler):
            if des.next_event_time() == clock:
                # process the next event because it is now
                continue
            else:
                call_scheduler = False
                if current_running_process == None:
                    item = scheduler.get_next_process()
                    if item:
                        dyna_prio, current_running_process, ready_at_time, remaining_time = item
                        next_event_for_this_pid = Event(clock, current_running_process, ready_at_time, process_transition.READY_TO_RUNNG)
                        des.event_queue.insert(next_event_for_this_pid)

def main(args):
    match args.s[0]:
        case "F":
            scheduler = FCFS()
        case "L":
            scheduler = LCFS()
        case "S":
            scheduler = SRTF()
        case "R":
            scheduler = RR(int(args.s[1:]))
        case "P":
            if ":" in args.s:
                quantum, maxprio = args.s[1:].split(":")
                scheduler = PRIO(int(quantum), int(maxprio))
            else:
                scheduler = PRIO(int(args.s[1:]))
    
    rand_generator = RandGenerator(args.rfile)
    process_arr = get_process_array(args.inputfile, rand_generator, scheduler.maxprio)

    des = DES(process_arr)
    

    simulation(des, rand_generator, process_arr, scheduler)
    print_summary(scheduler, process_arr)




if __name__ == "__main__":
    import argparse
    import re
    def valid_schedspec(value):
        # Use regular expression to check for valid specification
        if not re.match(r'^[FLS]|[R|P]\d+(:\d+)?$', value):
            raise argparse.ArgumentTypeError(f'Invalid scheduler specification: {value}. Must be one of F, L, S, R<num>, P<num>, or P<num>:<num>.')
        return value


    parser = argparse.ArgumentParser(description='Scheduler algorithms for OS')
    parser.add_argument('--inputfile', type=str, default="lab2_assign/input1", help='Process array input file')
    parser.add_argument('--rfile', type=str, default="lab2_assign/rfile", help='random number file')
    parser.add_argument('-s', metavar='schedspec', type=valid_schedspec, help='Scheduler specification (F, L, S, or R<num>)')

    args = parser.parse_args()
    # args = parser.parse_args("-sF --inputfile lab2_assign/input0".split())

    main(args)
