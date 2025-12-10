use crate::min_max_keeper::MinMaxKeeper;
use crate::sma_keeper::SmaKeeper;

pub struct KdjKeeper {
    period_fast_k: usize,
    period_slow_k: usize,
    period_slow_d: usize,
    min_max_keeper: MinMaxKeeper,
    slow_k: SmaKeeper,
    slow_d: SmaKeeper,
    j: f64,
    timestamp_counter: u64,
}

impl KdjKeeper {
    pub fn new(period_fast_k: usize, period_slow_k: usize, period_slow_d: usize) -> Self {
        KdjKeeper {
            period_fast_k,
            period_slow_k,
            period_slow_d,
            slow_k: SmaKeeper::new(period_slow_k, 0, 0.0),
            slow_d: SmaKeeper::new(period_slow_d, 0, 0.0),
            min_max_keeper: MinMaxKeeper::with_capacity(period_fast_k * 2, 0.0001),
            j: 0.0,
            timestamp_counter: 1,
        }
    }

    pub fn add(&mut self, high: f64, low: f64, close: f64) -> Result<(), String> {
        self.min_max_keeper.add(high).map_err(|e| e.to_string())?;
        self.min_max_keeper.add(low).map_err(|e| e.to_string())?;

        let k_fast = self.peek_next(close);
        self.slow_k.add(self.timestamp_counter, k_fast);
        self.slow_d.add(self.timestamp_counter, self.slow_k.get());
        self.timestamp_counter += 1;
        let k = self.slow_k.get();
        let d = self.slow_d.get();
        self.j = 3.0 * k - 2.0 * d;

        if self.j.is_nan() {
            return Err(format!("KDJ J is nan K={} D={}", k, d));
        }

        Ok(())
    }

    pub fn peek_next(&self, close: f64) -> f64 {
        let rolling_high = self.min_max_keeper.get_max();
        let rolling_low = self.min_max_keeper.get_min();
        if rolling_high == rolling_low {
            return 0.0;
        }
        (100.0 * (close - rolling_low)) / (rolling_high - rolling_low)
    }

    pub fn get_j_centered(&self) -> f64 {
        self.j - 50.0
    }

    pub fn get(&self) -> (f64, f64, f64) {
        (self.slow_k.get(), self.slow_d.get(), self.j)
    }

    pub fn length(&self) -> usize {
        self.min_max_keeper.get_len()
    }

    pub fn is_over_bought_sold(
        &self,
        over_bought_thresh: f64,
        over_sold_thresh: f64,
    ) -> f64 {
        if self.slow_k.size() == 0 {
            return 1e-6;
        }
        let d = self.slow_d.get();
        if d > over_bought_thresh {
            return 1.0;
        }
        if d < over_sold_thresh {
            return -1.0;
        }
        1e-6
    }

    pub fn is_cross_golden_death(
        &self,
        cross_golden_thresh: f64,
        cross_death_thresh: f64,
    ) -> f64 {
        if self.slow_k.size() < 2 {
            return 1e-6;
        }
        let k = self.slow_k.get();
        let d = self.slow_d.get();
        let k_prev = self.slow_k.get_prev();
        let d_prev = self.slow_d.get_prev();

        if k > d && k_prev < d_prev && k <= cross_golden_thresh {
            return 1.0;
        }
        if k < d && k_prev > d_prev && k >= cross_death_thresh {
            return -1.0;
        }
        1e-6
    }

    pub fn is_peak_bottom(&self, peak_thresh: f64, bottom_thresh: f64) -> f64 {
        if self.slow_k.size() == 0 {
            return 1e-6;
        }
        if self.j > peak_thresh {
            return 1.0;
        }
        if self.j < bottom_thresh {
            return -1.0;
        }
        1e-6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdj_new() {
        let keeper = KdjKeeper::new(9, 3, 3);
        let (k, d, j) = keeper.get();
        assert_eq!(k, 0.0);
        assert_eq!(d, 0.0);
        assert_eq!(j, 0.0);
    }

    #[test]
    fn test_peek_next() {
        let mut keeper = KdjKeeper::new(9, 3, 3);
        keeper.add(110.0, 100.0, 105.0).unwrap();
        keeper.add(115.0, 105.0, 110.0).unwrap();

        let k_fast = keeper.peek_next(112.0);
        assert!(k_fast >= 0.0 && k_fast <= 100.0);
    }

    #[test]
    fn test_add() {
        let mut keeper = KdjKeeper::new(9, 3, 3);
        keeper.add(110.0, 100.0, 105.0).unwrap();
        keeper.add(115.0, 105.0, 110.0).unwrap();

        let (k, d, j) = keeper.get();
        assert!(k.is_finite());
        assert!(d.is_finite());
        assert!(j.is_finite());
    }

    #[test]
    fn test_get_j_centered() {
        let mut keeper = KdjKeeper::new(9, 3, 3);
        keeper.add(110.0, 100.0, 105.0).unwrap();
        keeper.add(115.0, 105.0, 110.0).unwrap();

        let j_centered = keeper.get_j_centered();
        assert!(j_centered.is_finite());
    }

    #[test]
    fn test_is_over_bought_sold() {
        let mut keeper = KdjKeeper::new(9, 3, 3);
        // Add enough data to get meaningful values
        for i in 0..20 {
            keeper.add(110.0 + i as f64, 100.0, 105.0 + i as f64).unwrap();
        }

        let result = keeper.is_over_bought_sold(80.0, 20.0);
        assert!(result.is_finite());
    }

    #[test]
    fn test_is_peak_bottom() {
        let mut keeper = KdjKeeper::new(9, 3, 3);
        for i in 0..20 {
            keeper.add(110.0 + i as f64, 100.0, 105.0 + i as f64).unwrap();
        }

        let result = keeper.is_peak_bottom(90.0, 10.0);
        assert!(result.is_finite());
    }
}

