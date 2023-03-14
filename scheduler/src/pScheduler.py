from collections import deque
from enum import Enum
import heapq
from collections import namedtuple
import bisect
import bisect


current_running_process = None

# final stats
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
    RUNNG_TO_READY = 3
    RUNNG_TO_BLOCK = 4
    BLOCK_TO_READY = 5
    DONE = 6


class PriorityQueue:
    def __init__(self):
        self.queue = []

    def isEmpty(self):
        return len(self.queue) == 0

    def insertStable(self, key, value):
        insert_at = bisect.bisect_right([x[0] for x in self.queue], key)
        self.queue.insert(insert_at, (key, value))

    def pop(self):
        if self.isEmpty():
            return None

        return self.queue.pop(0)

    def delete(self, pid):
        for i, (_, (proc, _)) in enumerate(self.queue):
            if proc.id == pid:
                del self.queue[i]
                return

    def __repr__(self) -> str:
        return str(list(self.queue))


Event = namedtuple('Event', ['process', 'transition'])


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

        # new fields
        self.state = "CREATED"
        self.state_start = at


class DES:
    def __init__(self, process_arr):
        self.process_arr = process_arr
        self.event_queue = PriorityQueue()

        for process in process_arr:
            process.state_start = process.at
            event = Event(process, process_transition.CREATED_TO_READY)
            self.event_queue.insertStable(process.at, event)

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
        self.runQ = PriorityQueue()

    def add_process(self, clock, pid):
        self.runQ.insertStable(clock, (pid))

    def get_next_process(self):
        if self.runQ:
            return self.runQ.pop()

        return None


class FCFS(Scheduler):
    def __init__(self):
        super().__init__('FCFS')


class LCFS(Scheduler):
    def __init__(self):
        super().__init__('LCFS')

    def add_process(self, clock, process):
        # TODO need to modify this
        self.runQ.queue.insert(0, (clock, process))


class SRTF(Scheduler):
    def __init__(self):
        super().__init__('SRTF')

    def add_process(self, clock, process):
        # TODO need to modify this
        insert_at = bisect.bisect_right([x.remaining_time for _, x in self.runQ.queue], process.remaining_time)
        self.runQ.queue.insert(insert_at, (clock, process))


class RR(Scheduler):
    def __init__(self, quantum):
        super().__init__('RR')
        self.quantum = quantum


class PRIO(Scheduler):
    def __init__(self, quantum, maxprio=4):
        super().__init__('PRIO', quantum, maxprio)
        self.active_queues = [PriorityQueue() for _ in range(maxprio)]
        self.expired_queues = [PriorityQueue() for _ in range(maxprio)]

    def add_process(self, clock, process):
        # insert_at = bisect.bisect_right([x[0] for x in self.active_queues[prio].queue], prio)
        self.active_queues[process.dynamic_priority].insertStable(clock, process)

    def get_next_process(self):
        if not any(i.queue for i in self.active_queues) and not any(i.queue for i in self.expired_queues):
            return None

        if not any(i.queue for i in self.active_queues):
            self.active_queues, self.expired_queues = self.expired_queues, self.active_queues

        # get highest priority process in in active queue
        for queue in self.active_queues[::-1]:
            if not queue.isEmpty():
                return queue.pop()


class PREPRIO(Scheduler):
    def __init__(self, quantum, maxprio=4):
        super().__init__('PREPRIO', quantum, maxprio)
        self.active_queues = [PriorityQueue() for _ in range(maxprio)]
        self.expired_queues = [PriorityQueue() for _ in range(maxprio)]

    def add_process(self, clock, process):
        prio = process.dynamic_priority  # priority
        # insert_at = bisect.bisect_right([x[0] for x in self.active_queues[prio].queue], prio)
        self.active_queues[prio].insertStable(clock, process)

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


def create_process_array(filename: str, rand_generator: RandGenerator, maxprio: int) -> deque[Process]:
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
    last_process_finish_time = max([p.finish_time for p in process_arr])
    cpu_util = cpu_time / last_process_finish_time * 100
    io_util = total_io_time / last_process_finish_time * 100
    avg_tat = sum([p.turnaround_time for p in process_arr]) / len(process_arr)
    avg_cw = sum([p.cw for p in process_arr]) / len(process_arr)
    throughput = len(process_arr) / last_process_finish_time * 100.

    print(f"SUM: {last_process_finish_time} {cpu_util:.2f} {io_util:.2f} {avg_tat:.2f} {avg_cw:.2f} {throughput:.3f}")


def simulation(des, rand_generator, process_arr, scheduler):
    global last_process_finish_time, cpu_time, current_running_process, n_io_blocked, io_start_time, total_io_time, debug_mode
    while (event := des.next_event()):
        if debug_mode:
            print("\ndebug info (capture new event):")
            print(f"clock={event[0]} pid={event[1][0].id} transition={event[1][1]}")
            if scheduler.name in ["PREPRIO", "PRIO"]:
                print(f"{scheduler.active_queues=}")
                print(f"{scheduler.expired_queues=}")
            else:
                print(f"{scheduler.runQ=}")
            print(f"BEFORE DOING EVENT {[(i[0], i[1][0].id, i[1][1]) for i in des.event_queue.queue]}")
            print(f"{current_running_process=}, {n_io_blocked=}, {io_start_time=}, {total_io_time=}, {cpu_time=}")
            for process in process_arr:
                print(f"{process.id=} {process.remaining_time=} {process.dynamic_priority=} {process.prempted=}")

        clock, (process, transition) = event
        time_in_state = clock - process.state_start

        match transition:
            case process_transition.CREATED_TO_READY:
                # must come from BLOCKED or CREATED
                # add to run queue, no event created
                print(f"{clock} {process.id} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                scheduler.add_process(clock, process)
                call_scheduler = True

            case process_transition.READY_TO_RUNNG:

                # if not coming from pre-emption
                if not process.prempted:
                    cpuburst = rand_generator.next(process.cb)
                    cpuburst = min(cpuburst, process.remaining_time)
                    process.current_cpu_burst = cpuburst
                else:
                    cpuburst = process.current_cpu_burst

                print(f"{clock} {process.id} {time_in_state}: {transition.name.replace('_TO_', ' -> ')} cb={cpuburst} rem={process.remaining_time} prio={process.dynamic_priority}")
                process.state = "RUNNG"
                process.state_start = clock
                current_running_process = process.id

                # increase cw time
                process.cw += time_in_state

                # since process is running, remove prememted flag
                process.prempted = False

                # accumulate the cpu time, update the remaining time
                # check quantum and decide to run normally or fire a preemt signal
                if scheduler.quantum < cpuburst:
                    # need to set up preempt signal
                    cpu_time += scheduler.quantum
                    process.remaining_time -= scheduler.quantum

                    next_event = Event(process, process_transition.RUNNG_TO_READY)
                    des.event_queue.insertStable(clock + scheduler.quantum, next_event)
                else:
                    # can exhaust cpuburst now, so send to block or done
                    cpu_time += cpuburst

                    # create event for done or blocking
                    if (cpuburst >= process.remaining_time):
                        # done with this process
                        next_event = Event(process, process_transition.DONE)
                        des.event_queue.insertStable(clock + cpuburst, next_event)
                    else:
                        next_event = Event(process, process_transition.RUNNG_TO_BLOCK)
                        des.event_queue.insertStable(clock + cpuburst, next_event)

                    # process_arr[pid].remaining_time -= cpuburst

            case process_transition.RUNNG_TO_READY:

                # stop the current running process
                current_running_process = None

                if process.prempted:
                    process.remaining_time -= time_in_state

                process.current_cpu_burst -= time_in_state

                print(f"{clock} {process.id} {time_in_state}: {'RUNNG_TO_READY'.replace('_TO_', ' -> ')}  cb={process.current_cpu_burst} rem={process.remaining_time} prio={process.dynamic_priority}")
                process.state = "READY"
                process.state_start = clock

                # adjust the dynamic priority
                # TODO need to make this generic
                if scheduler.name in ["PRIO", "PREPRIO"]:
                    process.dynamic_priority -= 1
                    if process.dynamic_priority < 0:
                        process.dynamic_priority = process.static_priority - 1
                        # same as add process but insert into expired queue
                        scheduler.expired_queues[process.dynamic_priority].insertStable(clock, process)
                    else:
                        scheduler.add_process(clock, process)
                else:
                    scheduler.add_process(clock, process)

                process.prempted = True
                call_scheduler = True

            case process_transition.RUNNG_TO_BLOCK:
                process.remaining_time -= time_in_state

                # finished running now blocking
                current_running_process = None

                # choose a random number between 1 and io or if io is greater than time left, choose time left
                ioburst = rand_generator.next(process.io)

                # update the start time of io
                if n_io_blocked == 0:
                    io_start_time = clock

                n_io_blocked += 1

                print(f"{clock} {process.id} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}  ib={ioburst} rem={process.remaining_time}")
                process.state = "BLOCK"
                process.state_start = clock

                next_event = Event(process, process_transition.BLOCK_TO_READY)
                des.event_queue.insertStable(clock + ioburst, next_event)
                call_scheduler = True

            case process_transition.BLOCK_TO_READY:
                # when a process is ready from block state, check if it is more important than the current running process
                # cond1 = the block_to_ready process has higher priority than the current running process
                # cond2 = the running process doesn't have any pending event for current time

                # update n_io_blocked
                n_io_blocked -= 1
                if n_io_blocked == 0:
                    total_io_time += clock - io_start_time

                # reset dynamic priority
                process.dynamic_priority = process.static_priority - 1

                process.io_time += time_in_state
                print(f"{clock} {process.id} {time_in_state}: {transition.name.replace('_TO_', ' -> ')}")
                process.state = "READY"
                process.state_start = clock

                if (current_running_process is not None) and (scheduler.name in ["PREPRIO"]):
                    cond1 = process.dynamic_priority > process_arr[current_running_process].dynamic_priority
                    cond2 = [c for c, ev in des.event_queue.queue if ev.process.id == current_running_process][0] != clock  # current running process doesn't have any pending event for current time
                    if cond1 and cond2:
                        desc = "YES"
                        # if yes, then preempt and stop the current running process, delete the current running process event and modify it to be a ready_to_running event
                        des.event_queue.delete(current_running_process)
                        process_arr[current_running_process].prempted = True
                        des.event_queue.insertStable(clock, Event(process_arr[current_running_process], process_transition.RUNNG_TO_READY))
                    else:
                        desc = "NO"

                    scheduler.add_process(clock, process)

                    print(f"    --> PrioPreempt Cond1={int(cond1)} Cond2={int(cond2)} ({current_running_process}) --> {desc}")
                else:
                    scheduler.add_process(clock, process)

                call_scheduler = True

            case process_transition.DONE:
                process.remaining_time -= time_in_state
                # assert process.remaining_time == 0
                current_running_process = None
                print(f"{clock} {process.id} {time_in_state}: {'Done'}")
                process.finish_time = clock
                process.turnaround_time = clock - process.at
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
                        _, next_process = item
                        next_event = Event(next_process, process_transition.READY_TO_RUNNG)
                        des.event_queue.insertStable(clock, next_event)

        if debug_mode:
            print(f"AFTER DOING EVENT + SCHEDULING {[(i[0], i[1][0].id, i[1][1]) for i in des.event_queue.queue]}")


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
        case "E":
            if ":" in args.s:
                quantum, maxprio = args.s[1:].split(":")
                scheduler = PREPRIO(int(quantum), int(maxprio))
            else:
                scheduler = PREPRIO(int(args.s[1:]))

    rand_generator = RandGenerator(args.rfile)
    process_arr = create_process_array(args.inputfile, rand_generator, scheduler.maxprio)

    des = DES(process_arr)

    simulation(des, rand_generator, process_arr, scheduler)
    print_summary(scheduler, process_arr)


if __name__ == "__main__":
    import argparse
    import re

    def valid_schedspec(value):
        # Use regular expression to check for valid specification
        if not re.match(r'^[FLS]|[R|P|E]\d+(:\d+)?$', value):
            raise argparse.ArgumentTypeError(f'Invalid scheduler specification: {value}. Must be one of F, L, S, R<num>, P<num>, or P<num>:<num>.')
        return value


    parser = argparse.ArgumentParser(description='Scheduler algorithms for OS')
    parser.add_argument('--inputfile', type=str, default="lab2_assign/input1", help='Process array input file')
    parser.add_argument('--rfile', type=str, default="lab2_assign/rfile", help='random number file')
    parser.add_argument('-s', metavar='schedspec', type=valid_schedspec, help='Scheduler specification (F, L, S, R<num>, P<num>, P<num>:<num> or E<num>:<num>).')
    parser.add_argument('--debug', action='store_true', help='Enable debug mode')

    args = parser.parse_args()
    # args = parser.parse_args("-sE2:5 --inputfile lab2_assign/input3".split())

    debug_mode = args.debug
    debug_mode = True

    main(args)
