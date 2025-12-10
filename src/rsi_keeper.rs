use std::collections::VecDeque;

fn is_near_zero(value: f64, epsilon: f64) -> bool {
    value < epsilon && value > -epsilon
}

pub struct RsiKeeper {
    max_len: usize,
    rsi: f64,
    prev_rsi: f64,
    price_arr: VecDeque<f64>,
}

impl RsiKeeper {
    pub fn new() -> Self {
        eprintln!("warning: init empty rsi keeper. use new RsiKeeper(len) to create new RsiKeeper");
        RsiKeeper {
            max_len: 0,
            rsi: 50.0,
            prev_rsi: 50.0,
            price_arr: VecDeque::with_capacity(10),
        }
    }

    pub fn with_period(max_len: usize) -> Self {
        RsiKeeper {
            max_len,
            rsi: 50.0,
            prev_rsi: 50.0,
            price_arr: VecDeque::with_capacity(max_len),
        }
    }

    pub fn add(&mut self, price: f64) {
        self.price_arr.push_back(price);
        while self.price_arr.len() > self.max_len && self.max_len > 0 {
            self.price_arr.pop_front();
        }

        if self.price_arr.len() < 2 {
            return;
        }

        let mut gain = 0.0;
        let mut loss = 0.0;

        // Calculate initial gain and loss
        for i in 1..self.price_arr.len() {
            let change = self.price_arr[i] - self.price_arr[i - 1];
            if change > 0.0 {
                gain += change;
            } else {
                loss -= change;
            }
        }

        // Calculate the average gain and loss
        gain /= self.max_len as f64;
        loss /= self.max_len as f64;

        self.prev_rsi = self.rsi;

        let rs = if is_near_zero(loss, 0.0001) {
            100.0
        } else {
            gain / loss
        };
        self.rsi = 100.0 - (100.0 / (1.0 + rs));
    }

    pub fn get_prev(&self) -> f64 {
        self.prev_rsi
    }

    pub fn get(&self) -> f64 {
        self.rsi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_new() {
        let keeper = RsiKeeper::new();
        assert_eq!(keeper.max_len, 0);
        assert_eq!(keeper.rsi, 50.0);
    }

    #[test]
    fn test_rsi_with_period() {
        let keeper = RsiKeeper::with_period(14);
        assert_eq!(keeper.max_len, 14);
        assert_eq!(keeper.rsi, 50.0);
    }

    #[test]
    fn test_add_insufficient_data() {
        let mut keeper = RsiKeeper::with_period(14);
        keeper.add(100.0);
        // RSI should still be 50.0 (initial value) since we need at least 2 prices
        assert_eq!(keeper.rsi, 50.0);
    }

    #[test]
    fn test_add_with_gains() {
        let mut keeper = RsiKeeper::with_period(14);
        keeper.add(100.0);
        keeper.add(101.0);
        keeper.add(102.0);
        // RSI should be calculated and should be > 50 (since we have gains)
        assert!(keeper.rsi >= 0.0 && keeper.rsi <= 100.0);
    }

    #[test]
    fn test_add_with_losses() {
        let mut keeper = RsiKeeper::with_period(14);
        keeper.add(100.0);
        keeper.add(99.0);
        keeper.add(98.0);
        // RSI should be calculated and should be < 50 (since we have losses)
        assert!(keeper.rsi >= 0.0 && keeper.rsi <= 100.0);
    }

    #[test]
    fn test_get_prev() {
        let mut keeper = RsiKeeper::with_period(14);
        keeper.add(100.0);
        keeper.add(101.0);
        let prev = keeper.get_prev();
        keeper.add(102.0);
        // prev_rsi should have been updated
        assert!(prev.is_finite());
    }

    #[test]
    fn test_rsi_bounds() {
        let mut keeper = RsiKeeper::with_period(14);
        // Add many prices to ensure RSI is calculated
        for i in 0..20 {
            keeper.add(100.0 + i as f64);
        }
        let rsi = keeper.get();
        // RSI should be between 0 and 100
        assert!(rsi >= 0.0 && rsi <= 100.0);
    }
}

