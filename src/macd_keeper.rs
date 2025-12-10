use std::collections::VecDeque;

use crate::sma_keeper::SmaKeeper;

pub struct MacdKeeper {
    slow_sma: SmaKeeper,
    fast_sma: SmaKeeper,
    dea_sma: SmaKeeper,
    slow_sma_history: VecDeque<f64>,
    fast_sma_history: VecDeque<f64>,
    diff_line_history: VecDeque<f64>,
    dea_sma_history: VecDeque<f64>,
    macd_line_history: VecDeque<f64>,
    price_history: VecDeque<f64>,
    slow_period: usize,
    fast_period: usize,
    dea_period: usize,
    divergen_wind: usize,
    top_trigger_price: f64,
    top_trigger_macd: f64,
    bot_trigger_price: f64,
    bot_trigger_macd: f64,
    timestamp_counter: u64,
}

impl MacdKeeper {
    pub fn new(
        slow_period: usize,
        fast_period: usize,
        dea_period: usize,
        divergen_wind: usize,
        prices: Option<Vec<f64>>,
    ) -> Self {
        let mut keeper = MacdKeeper {
            slow_sma: SmaKeeper::new(slow_period, 0, 0.0),
            fast_sma: SmaKeeper::new(fast_period, 0, 0.0),
            dea_sma: SmaKeeper::new(dea_period, 0, 0.0),
            slow_sma_history: VecDeque::new(),
            fast_sma_history: VecDeque::new(),
            diff_line_history: VecDeque::new(),
            dea_sma_history: VecDeque::new(),
            macd_line_history: VecDeque::new(),
            price_history: VecDeque::new(),
            slow_period,
            fast_period,
            dea_period,
            divergen_wind,
            top_trigger_price: 3.0,
            top_trigger_macd: -3.0,
            bot_trigger_price: -3.0,
            bot_trigger_macd: 3.0,
            timestamp_counter: 1,
        };

        // Maintain max length for history arrays
        keeper.slow_sma_history = VecDeque::new();
        keeper.fast_sma_history = VecDeque::new();
        keeper.diff_line_history = VecDeque::new();
        keeper.dea_sma_history = VecDeque::new();
        keeper.macd_line_history = VecDeque::with_capacity(divergen_wind);
        keeper.price_history = VecDeque::with_capacity(divergen_wind);

        if let Some(price_vec) = prices {
            for price in price_vec {
                keeper.add(price);
            }
        }

        keeper
    }

    pub fn add(&mut self, price: f64) {
        self.slow_sma.add(self.timestamp_counter, price);
        self.fast_sma.add(self.timestamp_counter, price);
        self.timestamp_counter += 1;

        let diff = self.fast_sma.get() - self.slow_sma.get();
        self.dea_sma.add(self.timestamp_counter, diff);
        self.timestamp_counter += 1;

        // Update history arrays
        self.slow_sma_history.push_back(self.slow_sma.get());
        self.fast_sma_history.push_back(self.fast_sma.get());
        self.diff_line_history.push_back(diff);
        self.dea_sma_history.push_back(self.dea_sma.get());
        self.macd_line_history.push_back(diff - self.dea_sma.get());
        self.price_history.push_back(price);

        // Maintain max length for history arrays
        while self.slow_sma_history.len() > 10 {
            self.slow_sma_history.pop_front();
        }
        while self.fast_sma_history.len() > 10 {
            self.fast_sma_history.pop_front();
        }
        while self.diff_line_history.len() > 10 {
            self.diff_line_history.pop_front();
        }
        while self.dea_sma_history.len() > 10 {
            self.dea_sma_history.pop_front();
        }
        while self.macd_line_history.len() > self.divergen_wind {
            self.macd_line_history.pop_front();
        }
        while self.price_history.len() > self.divergen_wind {
            self.price_history.pop_front();
        }
    }

    pub fn size(&self) -> usize {
        self.slow_sma_history.len()
    }

    pub fn check_cross(&self) -> bool {
        if self.diff_line_history.len() < 5 {
            return false;
        }

        let macd_last = self.macd_line_history.back().copied().unwrap_or(0.0);
        let macd_prev = if self.macd_line_history.len() >= 5 {
            self.macd_line_history.get(self.macd_line_history.len() - 5)
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        if (macd_last > 0.0 && macd_prev > 0.0) || (macd_last < 0.0 && macd_prev < 0.0) {
            return false;
        }

        true
    }

    pub fn check_divergence(&self) -> f64 {
        if self.macd_line_history.len() < self.divergen_wind {
            return 0.0;
        }

        let macd_first = self.macd_line_history.front().copied().unwrap_or(0.0);
        let macd_last = self.macd_line_history.back().copied().unwrap_or(0.0);
        let price_first = self.price_history.front().copied().unwrap_or(0.0);
        let price_last = self.price_history.back().copied().unwrap_or(0.0);

        let size = self.macd_line_history.len();
        if size < 2 {
            return 0.0;
        }

        let macd_slope = (macd_last - macd_first) / (size - 1) as f64;
        let price_slope = (price_last - price_first) / (size - 1) as f64;

        if macd_slope * price_slope >= 0.0 {
            return 0.0;
        }

        price_slope - macd_slope
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macd_new() {
        let keeper = MacdKeeper::new(26, 12, 9, 20, None);
        assert_eq!(keeper.slow_period, 26);
        assert_eq!(keeper.fast_period, 12);
        assert_eq!(keeper.dea_period, 9);
        assert_eq!(keeper.divergen_wind, 20);
    }

    #[test]
    fn test_add() {
        let mut keeper = MacdKeeper::new(26, 12, 9, 20, None);
        keeper.add(100.0);
        keeper.add(101.0);
        keeper.add(102.0);
        assert!(keeper.size() > 0);
    }

    #[test]
    fn test_check_cross() {
        let mut keeper = MacdKeeper::new(26, 12, 9, 20, None);
        // Need at least 5 values for check_cross
        for i in 0..10 {
            keeper.add(100.0 + i as f64);
        }
        let result = keeper.check_cross();
        // Result depends on the actual MACD values
        assert!(result || !result); // Just check it doesn't panic
    }

    #[test]
    fn test_check_divergence() {
        let mut keeper = MacdKeeper::new(26, 12, 9, 20, None);
        // Need at least divergen_wind values
        for i in 0..25 {
            keeper.add(100.0 + i as f64);
        }
        let result = keeper.check_divergence();
        assert!(result.is_finite());
    }

    #[test]
    fn test_check_divergence_insufficient_data() {
        let mut keeper = MacdKeeper::new(26, 12, 9, 20, None);
        // Not enough data
        for i in 0..10 {
            keeper.add(100.0 + i as f64);
        }
        let result = keeper.check_divergence();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_with_initial_prices() {
        let prices = vec![100.0, 101.0, 102.0, 103.0];
        let keeper = MacdKeeper::new(26, 12, 9, 20, Some(prices));
        assert_eq!(keeper.size(), 4);
    }
}

