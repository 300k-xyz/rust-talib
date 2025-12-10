use std::collections::VecDeque;
use crate::common_utils::BUY;

/// Represents a trade message
#[derive(Debug, Clone)]
pub struct TradeMessage {
    pub price: f64,
    pub side: bool,
}

/// Keeps track of trade prices, sides, and timestamps using sliding windows
pub struct TradePriceKeeper {
    frequency_ms: usize,
    current_price: f64,
    current_price_side: bool,
    history_price: VecDeque<f64>,
    history_sides: VecDeque<f64>,
    history_ts: VecDeque<u64>,
    max_length: usize,
}

impl TradePriceKeeper {
    /// Creates a new TradePriceKeeper with the specified frequency and maximum length
    pub fn new(frequency_ms: usize, max_length: usize) -> Self {
        TradePriceKeeper {
            frequency_ms,
            current_price: 0.0,
            current_price_side: BUY,
            history_price: VecDeque::with_capacity(max_length),
            history_sides: VecDeque::with_capacity(max_length),
            history_ts: VecDeque::with_capacity(max_length),
            max_length,
        }
    }

    /// Called periodically to record the current price
    pub fn on_period_callback(&mut self, timestamp: u64) {
        if self.current_price > 0.0 {
            self.history_price.push_back(self.current_price);
            self.history_sides.push_back(if self.current_price_side == BUY {
                1.0
            } else {
                -1.0
            });
            self.history_ts.push_back(timestamp);

            // Maintain max length
            while self.history_price.len() > self.max_length {
                self.history_price.pop_front();
            }
            while self.history_sides.len() > self.max_length {
                self.history_sides.pop_front();
            }
            while self.history_ts.len() > self.max_length {
                self.history_ts.pop_front();
            }
        }
    }

    /// Updates the current price and side from a trade message
    pub fn on_receive_trade(&mut self, trade: &TradeMessage) {
        self.current_price = trade.price;
        self.current_price_side = trade.side;
    }

    /// Gets a history price by index (supports negative indexing like Python)
    /// 
    /// # Arguments
    /// * `index` - Index into history (negative values count from the end, -1 is most recent)
    /// 
    /// # Panics
    /// Panics if history is empty or index is out of range
    pub fn get_history_price(&self, index: i64) -> f64 {
        let size = self.history_price.len();
        
        if size == 0 {
            panic!("TradePriceKeeper history price is empty");
        }

        let actual_index = if index < 0 {
            let neg_index = (size as i64 + index) as usize;
            if neg_index >= size {
                panic!(
                    "TradePriceKeeper history price index out of range index={} size={}",
                    index, size
                );
            }
            neg_index
        } else {
            if index as usize >= size {
                panic!(
                    "TradePriceKeeper history price index out of range index={} size={}",
                    index, size
                );
            }
            index as usize
        };

        *self.history_price.get(actual_index).unwrap()
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
            panic!("TradePriceKeeper history_ts is empty");
        }

        let actual_index = if index < 0 {
            let neg_index = (size as i64 + index) as usize;
            if neg_index >= size {
                panic!(
                    "TradePriceKeeper history_ts index out of range index={} size={}",
                    index, size
                );
            }
            neg_index
        } else {
            if index as usize >= size {
                panic!(
                    "TradePriceKeeper history_ts index out of range index={} size={}",
                    index, size
                );
            }
            index as usize
        };

        *self.history_ts.get(actual_index).unwrap()
    }

    /// Gets the size of the price history
    pub fn get_history_prices_size(&self) -> usize {
        self.history_price.len()
    }

    /// Gets the current price
    pub fn get_current_price(&self) -> f64 {
        self.current_price
    }

    /// Gets the current price side based on recent history (last 10 trades)
    /// Returns 1.0 for buy-dominant, -1.0 for sell-dominant
    pub fn get_current_price_side(&self) -> f64 {
        let mut buy_count = 0;
        let mut sell_count = 0;
        
        let lookback = self.history_sides.len().min(10);
        
        for i in 0..lookback {
            let idx = -(i as i64 + 1);
            if let Ok(side) = self.get_history_side(idx) {
                if side > 0.0 {
                    buy_count += 1;
                } else {
                    sell_count += 1;
                }
            }
        }

        if buy_count > sell_count {
            1.0
        } else {
            -1.0
        }
    }

    /// Gets the side ratio for trades up to a given timestamp
    /// Returns (buy_count - sell_count) / (buy_count + sell_count)
    pub fn get_side_ratio(&self, timestamp_to: u64) -> f64 {
        let mut buy_count = 0;
        let mut sell_count = 0;
        
        let size = self.history_sides.len();
        for i in 0..size {
            let idx = -(i as i64 + 1);
            if let Ok(ts) = self.get_history_ts_safe(idx) {
                if ts < timestamp_to {
                    break;
                }
                if let Ok(side) = self.get_history_side(idx) {
                    if side > 0.0 {
                        buy_count += 1;
                    } else {
                        sell_count += 1;
                    }
                }
            }
        }

        let total = buy_count + sell_count;
        if total == 0 {
            return 0.0;
        }
        
        (buy_count as f64 - sell_count as f64) / total as f64
    }

    /// Helper method to get history side safely
    fn get_history_side(&self, index: i64) -> Result<f64, String> {
        let size = self.history_sides.len();
        
        if size == 0 {
            return Err("history_sides is empty".to_string());
        }

        let actual_index = if index < 0 {
            let neg_index = (size as i64 + index) as usize;
            if neg_index >= size {
                return Err(format!("index out of range: {}", index));
            }
            neg_index
        } else {
            if index as usize >= size {
                return Err(format!("index out of range: {}", index));
            }
            index as usize
        };

        Ok(*self.history_sides.get(actual_index).unwrap())
    }

    /// Helper method to get history timestamp safely
    fn get_history_ts_safe(&self, index: i64) -> Result<u64, String> {
        let size = self.history_ts.len();
        
        if size == 0 {
            return Err("history_ts is empty".to_string());
        }

        let actual_index = if index < 0 {
            let neg_index = (size as i64 + index) as usize;
            if neg_index >= size {
                return Err(format!("index out of range: {}", index));
            }
            neg_index
        } else {
            if index as usize >= size {
                return Err(format!("index out of range: {}", index));
            }
            index as usize
        };

        Ok(*self.history_ts.get(actual_index).unwrap())
    }
}
