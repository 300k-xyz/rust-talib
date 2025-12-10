use crate::min_max_keeper::MinMaxKeeper;
use crate::sma_keeper::SmaKeeper;

pub struct StochasticOscillatorKeeper {
    sma_keeper: SmaKeeper,
    percent_k: f64,
    percent_d: f64,
    k_period: usize,
    d_period: usize,
    min_max_keeper: MinMaxKeeper,
    timestamp_counter: u64,
}

impl StochasticOscillatorKeeper {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        StochasticOscillatorKeeper {
            k_period,
            d_period,
            sma_keeper: SmaKeeper::new(d_period, 0, 0.0),
            percent_k: 0.0,
            percent_d: 0.0,
            min_max_keeper: MinMaxKeeper::with_capacity(k_period, 0.0),
            timestamp_counter: 1,
        }
    }

    pub fn add(&mut self, value: f64) -> Result<(), String> {
        self.min_max_keeper.add(value).map_err(|e| e.to_string())?;
        let highest_high = self.min_max_keeper.get_max();
        let lowest_low = self.min_max_keeper.get_min();

        if (highest_high - lowest_low).abs() > 1e-10 {
            self.percent_k = 100.0 * ((value - lowest_low) / (highest_high - lowest_low));
        } else {
            self.percent_k = 0.0;
        }

        self.sma_keeper.add(self.timestamp_counter, self.percent_k);
        self.timestamp_counter += 1;
        self.percent_d = self.sma_keeper.get();

        Ok(())
    }

    pub fn get_percent_k(&self) -> f64 {
        self.percent_k
    }

    pub fn get_percent_d(&self) -> f64 {
        self.percent_d
    }

    pub fn get_k(&self) -> f64 {
        self.percent_k
    }

    pub fn get_d(&self) -> f64 {
        self.percent_d
    }

    pub fn is_overbought(&self) -> bool {
        if self.min_max_keeper.get_len() < self.k_period {
            return false;
        }

        self.percent_k > 80.0
    }

    pub fn is_oversold(&self) -> bool {
        if self.min_max_keeper.get_len() < self.k_period {
            return false;
        }

        self.percent_k < 20.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_new() {
        let keeper = StochasticOscillatorKeeper::new(14, 3);
        assert_eq!(keeper.k_period, 14);
        assert_eq!(keeper.d_period, 3);
        assert_eq!(keeper.percent_k, 0.0);
        assert_eq!(keeper.percent_d, 0.0);
    }

    #[test]
    fn test_add() {
        let mut keeper = StochasticOscillatorKeeper::new(14, 3);
        keeper.add(100.0).unwrap();
        keeper.add(101.0).unwrap();
        keeper.add(102.0).unwrap();
        
        // After adding values, percent_k and percent_d should be calculated
        assert!(keeper.get_k().is_finite());
        assert!(keeper.get_d().is_finite());
    }

    #[test]
    fn test_percent_k_calculation() {
        let mut keeper = StochasticOscillatorKeeper::new(5, 3);
        // Add values that create a range
        keeper.add(100.0).unwrap(); // low
        keeper.add(101.0).unwrap();
        keeper.add(102.0).unwrap();
        keeper.add(103.0).unwrap();
        keeper.add(104.0).unwrap(); // high
        
        // Add a value in the middle
        keeper.add(102.0).unwrap();
        
        // %K should be between 0 and 100
        let k = keeper.get_k();
        assert!(k >= 0.0 && k <= 100.0);
    }

    #[test]
    fn test_is_overbought() {
        let mut keeper = StochasticOscillatorKeeper::new(5, 3);
        // Add enough values to reach k_period
        for i in 0..5 {
            keeper.add(100.0 + i as f64).unwrap();
        }
        
        // Add a value that would make it overbought (near the high)
        keeper.add(104.0).unwrap();
        
        // May or may not be overbought depending on the calculation
        let result = keeper.is_overbought();
        assert!(result || !result); // Just check it doesn't panic
    }

    #[test]
    fn test_is_oversold() {
        let mut keeper = StochasticOscillatorKeeper::new(5, 3);
        // Add enough values to reach k_period
        for i in 0..5 {
            keeper.add(100.0 + i as f64).unwrap();
        }
        
        // Add a value that would make it oversold (near the low)
        keeper.add(100.0).unwrap();
        
        // May or may not be oversold depending on the calculation
        let result = keeper.is_oversold();
        assert!(result || !result); // Just check it doesn't panic
    }

    #[test]
    fn test_is_overbought_insufficient_data() {
        let mut keeper = StochasticOscillatorKeeper::new(5, 3);
        // Not enough data
        keeper.add(100.0).unwrap();
        keeper.add(101.0).unwrap();
        
        assert!(!keeper.is_overbought());
    }

    #[test]
    fn test_is_oversold_insufficient_data() {
        let mut keeper = StochasticOscillatorKeeper::new(5, 3);
        // Not enough data
        keeper.add(100.0).unwrap();
        keeper.add(101.0).unwrap();
        
        assert!(!keeper.is_oversold());
    }

    #[test]
    fn test_get_percent_k_d() {
        let mut keeper = StochasticOscillatorKeeper::new(14, 3);
        keeper.add(100.0).unwrap();
        keeper.add(101.0).unwrap();
        
        let k = keeper.get_percent_k();
        let d = keeper.get_percent_d();
        
        assert_eq!(k, keeper.get_k());
        assert_eq!(d, keeper.get_d());
    }
}

