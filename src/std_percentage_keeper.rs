use std::collections::VecDeque;
use crate::tick_price_keeper::TickPriceKeeper;
use crate::common_utils::calculate_volatility_percentage;

/// Keeps track of percentage-based standard deviation (volatility) values, caching them at specified frequency
pub struct StdPercentageKeeper {
    tick_price_keeper: TickPriceKeeper,
    mid_prices: VecDeque<f64>,
    frequency_ms: u64,
    cached_std: f64,
    last_cache_timestamp: u64,
    period: usize,
    max_length: usize,
}

impl StdPercentageKeeper {
    /// Creates a new StdPercentageKeeper with the specified period, frequency, and maximum length
    /// 
    /// # Arguments
    /// * `period` - Period for volatility calculation
    /// * `frequency_ms` - Frequency in milliseconds for caching STD
    /// * `max_length` - Maximum length for price history, usually same as the period
    pub fn new(period: usize, frequency_ms: u64, max_length: usize) -> Self {
        StdPercentageKeeper {
            tick_price_keeper: TickPriceKeeper::new(frequency_ms as usize, max_length),
            mid_prices: VecDeque::with_capacity(max_length),
            frequency_ms,
            cached_std: 0.0,
            last_cache_timestamp: 0,
            period,
            max_length,
        }
    }

    /// Updates the current bid and ask prices
    pub fn on_receive_tick(&mut self, timestamp: u64, bid: f64, ask: f64) {
        self.tick_price_keeper.on_receive_tick(bid, ask);
        
        // Calculate and store mid price
        let mid = (bid + ask) / 2.0;
        if mid > 0.0 {
            // Update tick price keeper periodically
            self.tick_price_keeper.on_period_callback(timestamp);
            
            // Store mid price for volatility calculation
            self.mid_prices.push_back(mid);
            
            // Maintain max length
            while self.mid_prices.len() > self.max_length {
                self.mid_prices.pop_front();
            }
        }
        
        // Update cache if enough time has passed
        if timestamp >= self.last_cache_timestamp + self.frequency_ms {
            self.update_cache();
            self.last_cache_timestamp = timestamp;
        }
    }

    /// Gets the current percentage-based standard deviation value (from cache if recent, otherwise recalculates)
    pub fn get_std(&self, timestamp: u64) -> f64 {
        if timestamp >= self.last_cache_timestamp + self.frequency_ms {
            // Cache expired, recalculate
            self.calculate_std()
        } else {
            // Return cached value
            self.cached_std
        }
    }

    /// Updates the cache with current STD value
    fn update_cache(&mut self) {
        self.cached_std = self.calculate_std();
    }

    /// Calculates the percentage-based standard deviation from the mid price history
    fn calculate_std(&self) -> f64 {
        if self.mid_prices.is_empty() {
            return 0.0;
        }

        // Convert VecDeque to Vec for calculate_volatility_percentage
        let prices: Vec<f64> = self.mid_prices.iter().copied().collect();
        
        // Calculate volatility for all prices (handles cases where len < period)
        let volatilities = calculate_volatility_percentage(&prices, self.period);
        
        // Return the last (most recent) volatility value, or 0.0 if None
        volatilities.last()
            .and_then(|v| *v)
            .unwrap_or(0.0)
    }

    /// Gets the tick price keeper (for advanced usage)
    pub fn get_tick_price_keeper(&self) -> &TickPriceKeeper {
        &self.tick_price_keeper
    }

    /// Gets the number of mid prices stored
    pub fn get_history_size(&self) -> usize {
        self.mid_prices.len()
    }
}
