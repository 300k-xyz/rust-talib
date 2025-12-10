use std::collections::{HashMap, VecDeque};

use crate::sma_keeper::SmaKeeper;

pub struct AtrKeeper {
    period: usize,
    candle_period: usize,
    high: VecDeque<f64>,
    low: VecDeque<f64>,
    close: VecDeque<f64>,
    atr_keeper: SmaKeeper,
    timestamp_counter: u64,
}

impl AtrKeeper {
    pub fn new(period: usize, candle_period: usize) -> Result<Self, String> {
        if period < 2 {
            return Err("ATR Period at least 2".to_string());
        }

        Ok(AtrKeeper {
            period,
            candle_period,
            high: VecDeque::new(),
            low: VecDeque::new(),
            close: VecDeque::new(),
            atr_keeper: SmaKeeper::new(period, 0, 0.0),
            timestamp_counter: 1,
        })
    }

    pub fn get_tr(&self, high: f64, low: f64, prev_close: f64) -> f64 {
        let hl = high - low;
        let hc = (high - prev_close).abs();
        let lc = (low - prev_close).abs();
        hl.max(hc).max(lc)
    }

    pub fn fast_get_tr(&self) -> f64 {
        let prev_close = if self.close.len() >= 2 {
            self.close.get(self.close.len() - 2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        self.get_tr(
            self.high.back().copied().unwrap_or(0.0),
            self.low.back().copied().unwrap_or(0.0),
            prev_close,
        )
    }

    pub fn add(&mut self, high_val: f64, low_val: f64, close_val: f64) {
        self.high.push_back(high_val);
        self.low.push_back(low_val);
        self.close.push_back(close_val);

        // Maintain max length
        while self.high.len() > self.period {
            self.high.pop_front();
        }
        while self.low.len() > self.period {
            self.low.pop_front();
        }
        while self.close.len() > self.period {
            self.close.pop_front();
        }

        if self.close.len() > 1 {
            self.atr_keeper.add(self.timestamp_counter, self.fast_get_tr());
            self.timestamp_counter += 1;
        }
    }

    pub fn peek_next(&self, high_val: f64, low_val: f64) -> f64 {
        (self.atr_keeper.get() * (self.period - 1) as f64
            + self.get_tr(high_val, low_val, self.close.back().copied().unwrap_or(0.0)))
            / self.period as f64
    }

    pub fn get(&self) -> f64 {
        self.atr_keeper.get()
    }

    pub fn fluctuant_index(&self, day_average_atr: &HashMap<usize, f64>) -> f64 {
        if self.close.is_empty() {
            return 1e-6;
        }
        let avg_atr = day_average_atr.get(&self.candle_period).copied().unwrap_or(0.0);
        10000.0 * (self.atr_keeper.get() / self.close.back().copied().unwrap_or(0.0) - avg_atr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atr_keeper_new() {
        let keeper = AtrKeeper::new(14, 60);
        assert!(keeper.is_ok());

        let result = AtrKeeper::new(1, 60);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.contains("at least 2"));
        }
    }

    #[test]
    fn test_get_tr() {
        let keeper = AtrKeeper::new(14, 60).unwrap();
        // Test with high=110, low=100, prev_close=105
        // Expected: max(110-100, |110-105|, |100-105|) = max(10, 5, 5) = 10
        let tr = keeper.get_tr(110.0, 100.0, 105.0);
        assert_eq!(tr, 10.0);
    }

    #[test]
    fn test_add_and_get() {
        let mut keeper = AtrKeeper::new(14, 60).unwrap();
        
        // Add first candle - ATR should not be calculated yet (need 2 candles)
        keeper.add(110.0, 100.0, 105.0);
        assert_eq!(keeper.close.len(), 1);
        
        // Add second candle - now ATR can be calculated
        keeper.add(115.0, 105.0, 110.0);
        assert_eq!(keeper.close.len(), 2);
        let atr = keeper.get();
        assert!(atr > 0.0);
    }

    #[test]
    fn test_peek_next() {
        let mut keeper = AtrKeeper::new(14, 60).unwrap();
        keeper.add(110.0, 100.0, 105.0);
        keeper.add(115.0, 105.0, 110.0);
        
        // Peek next ATR with new high/low values
        let peeked = keeper.peek_next(120.0, 110.0);
        assert!(peeked > 0.0);
    }

    #[test]
    fn test_fluctuant_index() {
        let mut keeper = AtrKeeper::new(14, 60).unwrap();
        keeper.add(110.0, 100.0, 105.0);
        keeper.add(115.0, 105.0, 110.0);
        
        let mut day_avg_atr = HashMap::new();
        day_avg_atr.insert(60, 0.01);
        
        let index = keeper.fluctuant_index(&day_avg_atr);
        // Should return a value (could be positive or negative depending on ATR)
        assert!(index.is_finite());
    }

    #[test]
    fn test_fluctuant_index_empty() {
        let keeper = AtrKeeper::new(14, 60).unwrap();
        let day_avg_atr = HashMap::new();
        let index = keeper.fluctuant_index(&day_avg_atr);
        assert_eq!(index, 1e-6);
    }
}

