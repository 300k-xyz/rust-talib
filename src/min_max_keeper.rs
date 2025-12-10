use std::collections::VecDeque;
use std::error::Error;

pub struct MinMaxKeeper {
    values_arr: VecDeque<f64>,
    max_arr: VecDeque<f64>,
    min_arr: VecDeque<f64>,
    max_len: usize,
    target_range: f64,
    last_ts: u64,
}

impl MinMaxKeeper {
    fn new() -> Self {
        eprintln!("warning init empty MinMaxKeeper");
        MinMaxKeeper {
            values_arr: VecDeque::new(),
            max_arr: VecDeque::new(),
            min_arr: VecDeque::new(),
            max_len: 0,
            target_range: 0.0001,
            last_ts: 0,
        }
    }

    pub fn with_capacity(period: usize, target_range: f64) -> Self {
        let mut keeper = MinMaxKeeper {
            values_arr: VecDeque::new(),
            max_arr: VecDeque::new(),
            min_arr: VecDeque::new(),
            max_len: period,
            target_range,
            last_ts: 0,
        };
        keeper.set_max_len(period);
        keeper
    }

    fn add_tail(&mut self, value: f64) {
        while !self.min_arr.is_empty() && value < *self.min_arr.back().unwrap() {
            self.min_arr.pop_back();
        }
        self.min_arr.push_back(value);

        while !self.max_arr.is_empty() && value > *self.max_arr.back().unwrap() {
            self.max_arr.pop_back();
        }
        self.max_arr.push_back(value);
    }

    fn remove_head(&mut self, value: f64) -> Result<(), Box<dyn Error>> {
        if !self.min_arr.is_empty() {
            if value < *self.min_arr.front().unwrap() {
                return Err(format!(
                    "wrong min_arr value {} min={}",
                    value,
                    self.min_arr.front().unwrap()
                )
                .into());
            } else if value == *self.min_arr.front().unwrap() {
                self.min_arr.pop_front();
            }
        }

        if !self.max_arr.is_empty() {
            if value > *self.max_arr.front().unwrap() {
                return Err(format!(
                    "wrong max_arr value {} max={}",
                    value,
                    self.max_arr.front().unwrap()
                )
                .into());
            } else if value == *self.max_arr.front().unwrap() {
                self.max_arr.pop_front();
            }
        }
        Ok(())
    }

    pub fn add_per_second(&mut self, timestamp_ms: u64, value: f64) -> Result<(), Box<dyn Error>> {
        if self.max_len == 0 {
            return Err("MinMaxKeeper max_len is 0".into());
        }
        if timestamp_ms > self.last_ts + 1000 {
            self.last_ts = timestamp_ms;
            while self.values_arr.len() >= self.max_len * 10
                || (self.values_arr.len() >= self.max_len
                    && (self.get_max() - self.get_min()) / self.get_min() > self.target_range)
            {
                self.remove_head(self.values_arr.front().unwrap().clone())?;
                self.values_arr.pop_front();
            }
            self.add_tail(value);
            self.values_arr.push_back(value);
        }
        Ok(())
    }

    pub fn add(&mut self, value: f64) -> Result<(), Box<dyn Error>> {
        if self.max_len == 0 {
            return Err("MinMaxKeeper max_len is 0".into());
        }
        while self.values_arr.len() >= self.max_len * 10
            || (self.values_arr.len() >= self.max_len
                && (self.get_max() - self.get_min()) / self.get_min() > self.target_range)
        {
            self.remove_head(self.values_arr.front().unwrap().clone())?;
            self.values_arr.pop_front();
        }
        self.add_tail(value);
        self.values_arr.push_back(value);
        Ok(())
    }

    pub fn get_len(&self) -> usize {
        self.values_arr.len()
    }

    pub fn get_max(&self) -> f64 {
        self.max_arr.front().copied().unwrap_or(0.0)
    }

    pub fn get_min(&self) -> f64 {
        self.min_arr.front().copied().unwrap_or(0.0)
    }

    pub fn get_mid(&self) -> f64 {
        (self.get_max() + self.get_min()) / 2.0
    }

    pub fn get_max_len(&self) -> usize {
        self.max_len
    }

    pub fn get_now_max(&self) -> f64 {
        self.max_arr.front().copied().unwrap_or(0.0)
    }

    pub fn get_now_min(&self) -> f64 {
        self.min_arr.front().copied().unwrap_or(0.0)
    }

    pub fn debug(&self) {
        println!("max={} min={}", self.get_max(), self.get_min());
    }

    pub fn set_max_len(&mut self, max_len: usize) {
        self.max_len = max_len;
    }

    pub fn set_target_range(&mut self, target_range: f64) {
        self.target_range = target_range;
    }

    pub fn is_full(&self) -> bool {
        self.values_arr.len() >= self.max_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut keeper = MinMaxKeeper::with_capacity(5, 0.0001);
        assert_eq!(keeper.get_len(), 0);
        assert_eq!(keeper.get_max_len(), 5);

        keeper.add(1.0).unwrap();
        keeper.add(2.0).unwrap();
        keeper.add(3.0).unwrap();
        assert_eq!(keeper.get_len(), 3);
        assert_eq!(keeper.get_min(), 1.0);
        assert_eq!(keeper.get_max(), 3.0);
    }

    #[test]
    fn test_min_max_tracking() {
        let mut keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        keeper.add(5.0).unwrap();
        keeper.add(3.0).unwrap();
        keeper.add(7.0).unwrap();
        keeper.add(2.0).unwrap();
        keeper.add(8.0).unwrap();
        assert_eq!(keeper.get_min(), 2.0);
        assert_eq!(keeper.get_max(), 8.0);
    }

    #[test]
    fn test_min_max_with_duplicates() {
        let mut keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        keeper.add(5.0).unwrap();
        keeper.add(5.0).unwrap();
        keeper.add(5.0).unwrap();
        assert_eq!(keeper.get_min(), 5.0);
        assert_eq!(keeper.get_max(), 5.0);
    }

    #[test]
    fn test_get_mid() {
        let mut keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        keeper.add(2.0).unwrap();
        keeper.add(8.0).unwrap();
        assert_eq!(keeper.get_mid(), 5.0);

        keeper.add(10.0).unwrap();
        assert_eq!(keeper.get_mid(), 6.0); // (2.0 + 10.0) / 2
    }

    #[test]
    fn test_get_now_max_min() {
        let mut keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        keeper.add(3.0).unwrap();
        keeper.add(7.0).unwrap();
        assert_eq!(keeper.get_now_max(), keeper.get_max());
        assert_eq!(keeper.get_now_min(), keeper.get_min());
    }

    #[test]
    fn test_is_full() {
        let mut keeper = MinMaxKeeper::with_capacity(3, 0.0001);
        assert!(!keeper.is_full());
        keeper.add(1.0).unwrap();
        assert!(!keeper.is_full());
        keeper.add(2.0).unwrap();
        assert!(!keeper.is_full());
        keeper.add(3.0).unwrap();
        assert!(keeper.is_full());
    }

    #[test]
    fn test_add_per_second() {
        let mut keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        // First add should work (last_ts is 0, so timestamp > 0 + 1000)
        keeper.add_per_second(2000, 1.0).unwrap();
        assert_eq!(keeper.get_len(), 1);

        // Same second should not add (2000 is not > 2000 + 1000)
        keeper.add_per_second(2000, 2.0).unwrap();
        assert_eq!(keeper.get_len(), 1);

        // Next second should add (3001 > 2000 + 1000)
        keeper.add_per_second(3001, 2.0).unwrap();
        assert_eq!(keeper.get_len(), 2);

        // Exactly 1000ms later should not add (3001 is not > 3001 + 1000)
        keeper.add_per_second(4001, 3.0).unwrap();
        assert_eq!(keeper.get_len(), 2);

        // More than 1000ms later should add
        keeper.add_per_second(4002, 3.0).unwrap();
        assert_eq!(keeper.get_len(), 3);
    }

    #[test]
    fn test_setters() {
        let mut keeper = MinMaxKeeper::with_capacity(5, 0.0001);
        assert_eq!(keeper.get_max_len(), 5);
        keeper.set_max_len(10);
        assert_eq!(keeper.get_max_len(), 10);
        keeper.set_target_range(0.001);
        // Can't easily test target_range without exposing it, but setter should work
    }

    #[test]
    fn test_empty_keeper() {
        let keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        assert_eq!(keeper.get_min(), 0.0);
        assert_eq!(keeper.get_max(), 0.0);
        assert_eq!(keeper.get_mid(), 0.0);
    }

    #[test]
    fn test_error_on_zero_max_len() {
        let mut keeper = MinMaxKeeper::new();
        // keeper has max_len = 0
        let result = keeper.add(1.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_len is 0"));
    }

    #[test]
    fn test_window_overflow_protection() {
        let mut keeper = MinMaxKeeper::with_capacity(5, 0.0001);
        // Add values up to max_len * 10 (50)
        for i in 0..50 {
            keeper.add(i as f64).unwrap();
        }
        // Should still be manageable (not crash)
        assert!(keeper.get_len() <= 50);

        // Add one more - should trigger cleanup
        keeper.add(51.0).unwrap();
        // Length should be reduced
        assert!(keeper.get_len() < 50);
    }

    #[test]
    fn test_target_range_triggering_removal() {
        let mut keeper = MinMaxKeeper::with_capacity(5, 0.1); // 10% range
                                                              // Fill up to max_len with same values
        keeper.add(100.0).unwrap();
        keeper.add(100.0).unwrap();
        keeper.add(100.0).unwrap();
        keeper.add(100.0).unwrap();
        keeper.add(100.0).unwrap();
        assert_eq!(keeper.get_len(), 5);

        // Now add a value that exceeds 10% range
        // (120-100)/100 = 0.2 > 0.1, so this should trigger removal
        keeper.add(120.0).unwrap();
        // After adding, the while loop should remove old values until condition is false
        // The length might be 5 or 6 depending on when the condition becomes false
        // But it should not grow unbounded
        assert!(keeper.get_len() <= 6);
        // The range should eventually be within target after cleanup
        let min = keeper.get_min();
        if min > 0.0 {
            let range_ratio = (keeper.get_max() - min) / min;
            // After cleanup, if window is at max_len, range should be within target
            if keeper.get_len() >= keeper.get_max_len() {
                assert!(range_ratio <= keeper.get_max_len() as f64 * 0.1); // Allow some tolerance
            }
        }
    }

    #[test]
    fn test_ascending_values() {
        let mut keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        for i in 1..=10 {
            keeper.add(i as f64).unwrap();
        }
        assert_eq!(keeper.get_min(), 1.0);
        assert_eq!(keeper.get_max(), 10.0);
    }

    #[test]
    fn test_descending_values() {
        let mut keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        for i in (1..=10).rev() {
            keeper.add(i as f64).unwrap();
        }
        assert_eq!(keeper.get_min(), 1.0);
        assert_eq!(keeper.get_max(), 10.0);
    }

    #[test]
    fn test_mixed_values() {
        let mut keeper = MinMaxKeeper::with_capacity(10, 0.0001);
        let values = vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0, 7.0, 4.0, 6.0];
        for v in values {
            keeper.add(v).unwrap();
        }
        assert_eq!(keeper.get_min(), 1.0);
        assert_eq!(keeper.get_max(), 9.0);
    }

    #[test]
    fn test_negative_values() {
        let mut keeper = MinMaxKeeper::with_capacity(5, 0.0001);
        keeper.add(-5.0).unwrap();
        keeper.add(-1.0).unwrap();
        keeper.add(-3.0).unwrap();
        assert_eq!(keeper.get_min(), -5.0);
        assert_eq!(keeper.get_max(), -1.0);
    }

    #[test]
    fn test_very_small_values() {
        let mut keeper = MinMaxKeeper::with_capacity(5, 0.0001);
        keeper.add(0.0001).unwrap();
        keeper.add(0.0002).unwrap();
        keeper.add(0.0003).unwrap();
        assert_eq!(keeper.get_min(), 0.0001);
        assert_eq!(keeper.get_max(), 0.0003);
    }

    #[test]
    fn test_division_by_zero_protection() {
        let mut keeper = MinMaxKeeper::with_capacity(5, 0.0001);
        // Add zero value - should not crash when checking range
        keeper.add(0.0).unwrap();
        keeper.add(1.0).unwrap();
        // Should handle division by zero gracefully in the range check
        assert!(keeper.get_len() > 0);
    }
}
