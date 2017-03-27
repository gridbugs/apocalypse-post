use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize)]
struct ScheduleEntry<T> {
    value: T,
    abs_time: u64,
    rel_time: u64,
    seq: u64,
}

impl<T> ScheduleEntry<T> {
    fn new(value: T, abs_time: u64, rel_time: u64, seq: u64) -> Self {
        ScheduleEntry {
            value: value,
            abs_time: abs_time,
            rel_time: rel_time,
            seq: seq,
        }
    }
}

impl<T> Ord for ScheduleEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let abs_time_ord = other.abs_time.cmp(&self.abs_time);
        if abs_time_ord == Ordering::Equal {
            other.seq.cmp(&self.seq)
        } else {
            abs_time_ord
        }
    }
}

impl<T> PartialOrd for ScheduleEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for ScheduleEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.abs_time == other.abs_time && self.seq == other.seq
    }
}

impl<T> Eq for ScheduleEntry<T> {}

#[derive(Debug)]
pub struct ScheduleEvent<T> {
    pub event: T,
    pub time_delta: u64,
    pub time_queued: u64,
    pub absolute_time: u64,
}

impl<T> ScheduleEvent<T> {
    fn new(event: T, time_delta: u64, time_queued: u64, absolute_time: u64) -> Self {
        ScheduleEvent {
            event: event,
            time_delta: time_delta,
            time_queued: time_queued,
            absolute_time: absolute_time,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ScheduleTicket {
    seq: u64,
}

pub struct Schedule<T> {
    heap: BinaryHeap<ScheduleEntry<T>>,
    invalid: HashSet<u64>,
    abs_time: u64,
    seq: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableSchedule<T> {
    entries: Vec<ScheduleEntry<T>>,
    invalid: HashSet<u64>,
    abs_time: u64,
    seq: u64,
}

impl<T> From<Schedule<T>> for SerializableSchedule<T> {
    fn from(schedule: Schedule<T>) -> Self {
        let Schedule {
            heap,
            invalid,
            abs_time,
            seq,
        } = schedule;

        SerializableSchedule {
            entries: heap.into_vec(),
            invalid: invalid,
            abs_time: abs_time,
            seq: seq,
        }
    }
}

impl<T> From<SerializableSchedule<T>> for Schedule<T> {
    fn from(schedule: SerializableSchedule<T>) -> Self {
        let SerializableSchedule {
            entries,
            invalid,
            abs_time,
            seq,
        } = schedule;

        Schedule {
            heap: BinaryHeap::from(entries),
            invalid: invalid,
            abs_time: abs_time,
            seq: seq,
        }
    }
}

impl<T> Schedule<T> {
    pub fn new() -> Self {
        Schedule {
            heap: BinaryHeap::new(),
            invalid: HashSet::new(),
            abs_time: 0,
            seq: 0,
        }
    }

    pub fn insert(&mut self, value: T, rel_time: u64) -> ScheduleTicket {
        let seq = self.seq;
        let abs_time = self.abs_time + rel_time;

        let entry = ScheduleEntry::new(value, abs_time, rel_time, seq);
        self.heap.push(entry);
        self.seq += 1;

        ScheduleTicket {
            seq: seq,
        }
    }

    pub fn insert_with_ticket(&mut self, value: T, rel_time: u64, ticket: ScheduleTicket) -> ScheduleTicket {

        // it must be a ticket we've given out in the past
        assert!(ticket.seq < self.seq, "Invalid schedule ticket");

        let abs_time = self.abs_time + rel_time;
        let entry = ScheduleEntry::new(value, abs_time, rel_time, ticket.seq);
        self.heap.push(entry);

        ScheduleTicket {
            seq: ticket.seq,
        }
    }

    pub fn next(&mut self) -> Option<ScheduleEvent<T>> {

        while let Some(entry) = self.heap.pop() {
            if self.invalid.remove(&entry.seq) {
                continue;
            }

            let time_delta = entry.abs_time - self.abs_time;
            self.abs_time = entry.abs_time;

            return Some(ScheduleEvent::new(entry.value, time_delta, entry.rel_time, entry.abs_time));
        }

        None
    }

    pub fn reset(&mut self) {
        self.heap.clear();
        self.abs_time = 0;
        self.seq = 0;
    }

    pub fn time(&self) -> u64 {
        self.abs_time
    }

    pub fn invalidate(&mut self, ticket: ScheduleTicket) {
        self.invalid.insert(ticket.seq);
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }
}
