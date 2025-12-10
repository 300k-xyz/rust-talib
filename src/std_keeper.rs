use crate::sma_keeper::SmaKeeper;
use crate::tick_price_keeper::TickPriceKeeper;

/// Keeps track of SMA and standard deviation values, caching them at specified frequency
pub struct StdKeeper {
    sma_keeper: SmaKeeper,
    tick_price_keeper: TickPriceKeeper,
    frequency_ms: u64,
    cached_sma: f64,
    cached_std: f64,
    last_cache_timestamp: u64,
    period: usize,
}

impl StdKeeper {
    /// Creates a new StdKeeper with the specified period, frequency, and maximum length
    /// 
    /// # Arguments
    /// * `period` - Period for SMA calculation
    /// * `frequency_ms` - Frequency in milliseconds for caching SMA and STD
    /// * `max_length` - Maximum length for price history
    pub fn new(period: usize, frequency_ms: u64, max_length: usize) -> Self {
        StdKeeper {
            sma_keeper: SmaKeeper::new(period, 0, 0.0),
            tick_price_keeper: TickPriceKeeper::new(frequency_ms as usize, max_length),
            frequency_ms,
            cached_sma: 0.0,
            cached_std: 0.0,
            last_cache_timestamp: 0,
            period,
        }
    }

    /// Updates the current bid and ask prices
    pub fn on_receive_tick(&mut self, timestamp: u64, bid: f64, ask: f64) {
        self.tick_price_keeper.on_receive_tick(bid, ask);
        
        // Update tick price keeper periodically
        self.tick_price_keeper.on_period_callback(timestamp);
        
        // Update SMA with mid price
        let mid = (bid + ask) / 2.0;
        if mid > 0.0 {
            self.sma_keeper.add(timestamp, mid);
        }
        
        // Update cache if enough time has passed
        if timestamp >= self.last_cache_timestamp + self.frequency_ms {
            self.update_cache(timestamp);
        }
    }

    /// Gets the current SMA value (from cache if recent, otherwise recalculates)
    pub fn get_sma(&self, timestamp: u64) -> f64 {
        if timestamp >= self.last_cache_timestamp + self.frequency_ms {
            // Cache expired, return current SMA from keeper
            self.sma_keeper.get()
        } else {
            // Return cached value
            self.cached_sma
        }
    }

    /// Gets the current standard deviation value (from cache if recent, otherwise recalculates)
    pub fn get_std(&self, timestamp: u64) -> f64 {
        if timestamp >= self.last_cache_timestamp + self.frequency_ms {
            // Cache expired, recalculate
            self.calculate_std()
        } else {
            // Return cached value
            self.cached_std
        }
    }

    /// Gets both SMA and STD values (from cache if recent, otherwise recalculates)
    pub fn get_sma_and_std(&self, timestamp: u64) -> (f64, f64) {
        if timestamp >= self.last_cache_timestamp + self.frequency_ms {
            // Cache expired, return current values
            (self.sma_keeper.get(), self.calculate_std())
        } else {
            // Return cached values
            (self.cached_sma, self.cached_std)
        }
    }

    /// Updates the cache with current SMA and STD values
    fn update_cache(&mut self, timestamp: u64) {
        self.cached_sma = self.sma_keeper.get();
        self.cached_std = self.calculate_std();
        self.last_cache_timestamp = timestamp;
    }

    /// Calculates the standard deviation from the tick price keeper history
    fn calculate_std(&self) -> f64 {
        let size = self.tick_price_keeper.get_history_prices_size();
        
        if size == 0 {
            return 0.0;
        }

        let mean = self.sma_keeper.get();
        
        // Use all available history or just the period
        let end_index = size as i64;
        let start_index = if size > self.period {
            (size - self.period) as i64
        } else {
            0
        };

        if end_index <= start_index {
            return 0.0;
        }

        let mut total_diff = 0.0;
        let count = (end_index - start_index) as usize;
        
        for i in start_index..end_index {
            // Calculate mid price from bid and ask history
            let bid = self.tick_price_keeper.get_history_bid(i);
            let ask = self.tick_price_keeper.get_history_ask(i);
            let price = (bid + ask) / 2.0;
            let diff = price - mean;
            total_diff += diff * diff;
        }

        if count == 0 {
            return 0.0;
        }

        let variance = total_diff / count as f64;
        variance.sqrt()
    }

    /// Gets the tick price keeper (for advanced usage)
    pub fn get_tick_price_keeper(&self) -> &TickPriceKeeper {
        &self.tick_price_keeper
    }

    /// Gets the SMA keeper (for advanced usage)
    pub fn get_sma_keeper(&self) -> &SmaKeeper {
        &self.sma_keeper
    }
}
