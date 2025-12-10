use std::collections::VecDeque;

pub struct SmaKeeper {
    arr: VecDeque<f64>,
    max_len: usize,
    sma: f64,
    prev_sma: f64,
    sum: f64,
    pub prev_timestamp: u64,
    time_gap_ms: u64,
}

impl SmaKeeper {
    /// Creates a new SmaKeeper with the specified maximum length, time gap, and initial SMA value
    pub fn new(max_len: usize, time_gap_ms: u64, initial_sma: f64) -> Self {
        SmaKeeper {
            arr: VecDeque::new(),
            max_len,
            sma: initial_sma,
            prev_sma: 0.0,
            sum: 0.0,
            prev_timestamp: 0,
            time_gap_ms,
        }
    }

    /// Returns the current size of the array
    pub fn size(&self) -> usize {
        self.arr.len()
    }

    /// Checks if the array has reached its maximum length
    pub fn is_full(&self) -> bool {
        self.arr.len() == self.max_len
    }

    /// Adds a new value with timestamp, updating the SMA
    pub fn add(&mut self, timestamp: u64, value: f64) -> f64 {
        if timestamp < self.prev_timestamp + self.time_gap_ms {
            return self.sma;
        }
        self.prev_timestamp = timestamp;

        self.arr.push_back(value);
        self.sum += value;

        while self.arr.len() > self.max_len {
            if let Some(remove) = self.arr.pop_front() {
                self.sum -= remove;
            }
        }

        self.prev_sma = self.sma; // Store previous SMA before updating
        self.sma = self.sum / self.arr.len() as f64;
        return self.sma;
    }

    /// Gets the current SMA value
    pub fn get(&self) -> f64 {
        self.sma
    }

    /// Gets the previous SMA value
    pub fn get_prev(&self) -> f64 {
        self.prev_sma
    }
}

