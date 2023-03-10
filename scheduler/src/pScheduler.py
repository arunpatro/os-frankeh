from collections import deque
from enum import Enum
import heapq
from collections import deque, namedtuple
import bisect
import bisect


maxprio = 4
current_running_process = None

# final stats
last_process_finish_time = float('-inf')
cpu_time = 0


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


Event = namedtuple('Event', ['time', 'pid', 'time_entered_state', 'transition'])


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
        self.time = 0

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
    def __init__(self, name):
        self.name = name
        self.runQ = []

    def get_next_process(self):
        if self.runQ:
            return self.runQ.pop(0)

        return None


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


def get_process_array(filename: str, rand_generator: RandGenerator) -> deque[Process]:
    process_array = []
    with open(filename) as f:
        pid = 0
        for line in f:
            at, tc, cb, io = [int(i) for i in line.split()]
            process_array.append(Process(pid, at, tc, cb, io, rand_generator))
            pid += 1
    return process_array


def print_summary(scheduler, process_arr):
    global last_process_finish_time, cpu_time
    # total_cpu_time = sum(p.tc - p.remaining_time for p in process_arr if p.finish_time is not None)
    total_io_time = sum(p.io_time for p in process_arr if p.finish_time is not None)
    total_elapsed_time = last_process_finish_time - min([i.at for i in process_arr])

    print(scheduler.name)
    for process in process_arr:
        print(f"{process.id:04d}:\t{process.at}\t{process.tc}\t{process.cb}\t{process.io}\t{process.static_priority} | {process.finish_time}\t{process.turnaround_time}\t{process.io_time}\t{process.cw}")

    cpu_util = cpu_time / total_elapsed_time * 100
    io_util = total_io_time / total_elapsed_time * 100
    avg_tat = sum([p.turnaround_time for p in process_arr]) / len(process_arr)
    avg_cw = sum([p.cw for p in process_arr]) / len(process_arr)
    throughput = len(process_arr) / last_process_finish_time * 100.

    print(f"SUM: {last_process_finish_time} {cpu_util:.2f} {io_util:.2f} {avg_tat:.2f} {avg_cw:.2f} {throughput:.3f}")


def simulation(des, rand_generator, process_arr, scheduler):
    global last_process_finish_time, cpu_time, current_running_process
    while (event := des.next_event()):
        clock, pid, time_entered_state, transition = event
        time_in_state = clock - time_entered_state
        remaining_time = process_arr[pid].remaining_time

        match transition:
            case process_transition.CREATED_TO_READY:
                # must come from BLOCKED or CREATED
                # add to run queue, no event created
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                scheduler.runQ.append((pid, clock))
                call_scheduler = True

            case process_transition.READY_TO_RUNNG:
                # get params and add the process to the runQ
                cpuburst = rand_generator.next(process_arr[pid].cb)
                cpuburst = min(cpuburst, process_arr[pid].remaining_time)
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')} cb={cpuburst} rem={remaining_time} prio={process_arr[pid].dynamic_priority}")

                if (cpuburst < process_arr[pid].remaining_time):
                    # accumulate the cpu time
                    cpu_time += cpuburst
                    # update the remaining time
                    process_arr[pid].remaining_time -= cpuburst

                    # create event for preemption or blocking
                    next_event_for_this_pid = Event(clock + cpuburst, pid, clock, process_transition.RUNNG_TO_BLOCK)

                    des.event_queue.insert(next_event_for_this_pid)
                else:
                    next_event_for_this_pid = Event(clock + cpuburst, pid, clock, process_transition.DONE)
                    des.event_queue.insert(next_event_for_this_pid)

            case process_transition.RUNNG_TO_BLOCK:
                # finished running now blocking

                current_running_process = None
                # choose a random number between 1 and io or if io is greater than time left, choose time left
                ioburst = rand_generator.next(process_arr[pid].io)

                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}  ib={ioburst} rem={remaining_time}")
                next_event_for_this_pid = Event(clock + ioburst, pid, clock, process_transition.BLOCK_TO_READY)
                des.event_queue.insert(next_event_for_this_pid)
                call_scheduler = True

            case process_transition.BLOCK_TO_READY:
                process_arr[pid].io_time += time_in_state
                print(f"{clock} {pid} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                scheduler.runQ.append((pid, clock))
                call_scheduler = True

            case process_transition.DONE:
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
                    if item == None:
                        continue
                    else:
                        # create event to run it for some time
                        current_running_process, ready_at_time = item
                        next_event_for_this_pid = Event(clock, current_running_process, ready_at_time, process_transition.READY_TO_RUNNG)
                        des.event_queue.insert(next_event_for_this_pid)


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
    parser.add_argument('--inputfile', type=str, default="lab2_assign/input3", help='Process array input file')
    parser.add_argument('--rfile', type=str, default="lab2_assign/rfile", help='random number file')
    args = parser.parse_args()

    main(args)
