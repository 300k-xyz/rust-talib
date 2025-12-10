use std::collections::VecDeque;

/// Keeps track of bid and ask prices using sliding windows
pub struct TickPriceKeeper {
    frequency_ms: usize,
    current_bid: f64,
    current_ask: f64,
    history_bid: VecDeque<f64>,
    history_ask: VecDeque<f64>,
    history_ts: VecDeque<u64>,
    max_length: usize,
}

impl TickPriceKeeper {
    /// Creates a new TickPriceKeeper with the specified frequency and maximum length
    pub fn new(frequency_ms: usize, max_length: usize) -> Self {
        TickPriceKeeper {
            frequency_ms,
            current_bid: 0.0,
            current_ask: 0.0,
            history_bid: VecDeque::with_capacity(max_length),
            history_ask: VecDeque::with_capacity(max_length),
            history_ts: VecDeque::with_capacity(max_length),
            max_length,
        }
    }

    /// Called periodically to record the current bid and ask prices
    pub fn on_period_callback(&mut self, timestamp: u64) {
        if self.current_bid > 0.0 && self.current_ask > 0.0 {
            self.history_bid.push_back(self.current_bid);
            self.history_ask.push_back(self.current_ask);
            self.history_ts.push_back(timestamp);

            // Maintain max length
            while self.history_bid.len() > self.max_length {
                self.history_bid.pop_front();
            }
            while self.history_ask.len() > self.max_length {
                self.history_ask.pop_front();
            }
            while self.history_ts.len() > self.max_length {
                self.history_ts.pop_front();
            }
        }
    }

    /// Updates the current bid and ask prices
    pub fn on_receive_tick(&mut self, bid: f64, ask: f64) {
        self.current_bid = bid;
        self.current_ask = ask;
    }

    /// Gets a history bid price by index (supports negative indexing like Python)
    /// 
    /// # Arguments
    /// * `index` - Index into history (negative values count from the end, -1 is most recent)
    /// 
    /// # Panics
    /// Panics if history is empty or index is out of range
    pub fn get_history_bid(&self, index: i64) -> f64 {
        let size = self.history_bid.len();
        
        if size == 0 {
            panic!("TickPriceKeeper history bid is empty");
        }

        let actual_index = if index < 0 {
            let neg_index = (size as i64 + index) as usize;
            if neg_index >= size {
                panic!(
                    "TickPriceKeeper history bid index out of range index={} size={}",
                    index, size
                );
            }
            neg_index
        } else {
            if index as usize >= size {
                panic!(
                    "TickPriceKeeper history bid index out of range index={} size={}",
                    index, size
                );
            }
            index as usize
        };

        *self.history_bid.get(actual_index).unwrap()
    }

    /// Gets a history ask price by index (supports negative indexing like Python)
    /// 
    /// # Arguments
    /// * `index` - Index into history (negative values count from the end, -1 is most recent)
    /// 
    /// # Panics
    /// Panics if history is empty or index is out of range
    pub fn get_history_ask(&self, index: i64) -> f64 {
        let size = self.history_ask.len();
        
        if size == 0 {
            panic!("TickPriceKeeper history ask is empty");
        }

        let actual_index = if index < 0 {
            let neg_index = (size as i64 + index) as usize;
            if neg_index >= size {
                panic!(
                    "TickPriceKeeper history ask index out of range index={} size={}",
                    index, size
                );
            }
            neg_index
        } else {
            if index as usize >= size {
                panic!(
                    "TickPriceKeeper history ask index out of range index={} size={}",
                    index, size
                );
            }
            index as usize
        };

        *self.history_ask.get(actual_index).unwrap()
    }

    /// Gets a history timestamp by index (supports negative indexing)
    /// 
    /// # Arguments
    /// * `index` - Index into history (negative values count from the end, -1 is most recent)
    /// 
    /// # Panics
    /// Panics if history is empty or index is out of range
    pub fn get_history_ts(&self, index: i64) -> u64 {
        let size = self.history_ts.len();
        
        if size == 0 {
            panic!("TickPriceKeeper history_ts is empty");
        }

        let actual_index = if index < 0 {
            let neg_index = (size as i64 + index) as usize;
            if neg_index >= size {
                panic!(
                    "TickPriceKeeper history_ts index out of range index={} size={}",
                    index, size
                );
            }
            neg_index
        } else {
            if index as usize >= size {
                panic!(
                    "TickPriceKeeper history_ts index out of range index={} size={}",
                    index, size
                );
            }
            index as usize
        };

        *self.history_ts.get(actual_index).unwrap()
    }

    /// Gets the size of the price history
    pub fn get_history_prices_size(&self) -> usize {
        self.history_bid.len()
    }

    /// Gets the current bid price
    pub fn get_current_bid(&self) -> f64 {
        self.current_bid
    }

    /// Gets the current ask price
    pub fn get_current_ask(&self) -> f64 {
        self.current_ask
    }

    /// Gets the current mid price (average of bid and ask)
    pub fn get_current_mid(&self) -> f64 {
        if self.current_bid > 0.0 && self.current_ask > 0.0 {
            (self.current_bid + self.current_ask) / 2.0
        } else {
            0.0
        }
    }

    /// Gets the current spread (ask - bid)
    pub fn get_current_spread(&self) -> f64 {
        if self.current_bid > 0.0 && self.current_ask > 0.0 {
            self.current_ask - self.current_bid
        } else {
            0.0
        }
    }
}
