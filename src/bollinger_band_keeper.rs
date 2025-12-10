use std::collections::VecDeque;

use crate::sma_keeper::SmaKeeper;

pub struct BollingerBandKeeper {
    arr: VecDeque<f64>,
    sma_keeper: SmaKeeper,
    window_size: usize,
    std_dev_multiplier: f64,
    upper_band: f64,
    lower_band: f64,
    timestamp_counter: u64,
}

impl BollingerBandKeeper {
    pub fn new() -> Self {
        BollingerBandKeeper {
            arr: VecDeque::new(),
            sma_keeper: SmaKeeper::new(1, 0, 0.0),
            window_size: 1,
            std_dev_multiplier: 2.0,
            upper_band: 0.0,
            lower_band: 0.0,
            timestamp_counter: 1,
        }
    }

    pub fn with_window(
        window_size: usize,
        std_dev_multiplier: f64,
        window_values: Option<Vec<f64>>,
    ) -> Self {
        let mut keeper = BollingerBandKeeper {
            arr: VecDeque::new(),
            sma_keeper: SmaKeeper::new(window_size, 0, 0.0),
            window_size,
            std_dev_multiplier,
            upper_band: 0.0,
            lower_band: 0.0,
            timestamp_counter: 1,
        };

        if let Some(values) = window_values {
            for value in values {
                keeper.add(value);
            }
        }

        keeper
    }

    pub fn size(&self) -> usize {
        self.arr.len()
    }

    pub fn add(&mut self, value: f64) {
        self.arr.push_back(value);
        while self.arr.len() > self.window_size {
            self.arr.pop_front();
        }

        self.sma_keeper.add(self.timestamp_counter, value);
        self.timestamp_counter += 1;
        let mean = self.sma_keeper.get();

        let mut sq_sum = 0.0;
        for i in 0..self.arr.len() {
            let diff = self.arr[i] - mean;
            sq_sum += diff * diff;
        }

        let variance = if self.arr.is_empty() {
            0.0
        } else {
            sq_sum / self.arr.len() as f64
        };

        let stddev = variance.sqrt();

        self.upper_band = mean + self.std_dev_multiplier * stddev;
        self.lower_band = mean - self.std_dev_multiplier * stddev;
    }

    pub fn is_above_upper_band(&self, value: f64) -> bool {
        value > self.upper_band
    }

    pub fn is_below_lower_band(&self, value: f64) -> bool {
        value < self.lower_band
    }

    pub fn is_inside_band(&self, value: f64) -> bool {
        value >= self.lower_band && value <= self.upper_band
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_band_new() {
        let keeper = BollingerBandKeeper::new();
        assert_eq!(keeper.size(), 0);
    }

    #[test]
    fn test_bollinger_band_with_window() {
        let keeper = BollingerBandKeeper::with_window(5, 2.0, None);
        assert_eq!(keeper.window_size, 5);
        assert_eq!(keeper.std_dev_multiplier, 2.0);
    }

    #[test]
    fn test_add_and_bands() {
        let mut keeper = BollingerBandKeeper::with_window(5, 2.0, None);
        keeper.add(100.0);
        keeper.add(101.0);
        keeper.add(102.0);
        assert_eq!(keeper.size(), 3);
        assert!(keeper.upper_band > keeper.lower_band);
    }

    #[test]
    fn test_band_checks() {
        let mut keeper = BollingerBandKeeper::with_window(5, 2.0, None);
        keeper.add(100.0);
        keeper.add(101.0);
        keeper.add(102.0);

        // Test with a value way above the band
        let high_value = keeper.upper_band + 10.0;
        assert!(keeper.is_above_upper_band(high_value));

        // Test with a value way below the band
        let low_value = keeper.lower_band - 10.0;
        assert!(keeper.is_below_lower_band(low_value));

        // Test with a value inside the band
        let mid_value = (keeper.upper_band + keeper.lower_band) / 2.0;
        assert!(keeper.is_inside_band(mid_value));
    }
}

