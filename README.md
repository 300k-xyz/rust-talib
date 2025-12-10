# rust-talib

A Rust implementation of technical analysis indicators library, providing efficient and memory-safe tools for financial data analysis.

## Features

This library provides implementations of common technical analysis indicators:

- **ATR (Average True Range)** - Measures market volatility
- **Bollinger Bands** - Volatility indicator with upper and lower bands
- **KDJ** - Stochastic oscillator variant for momentum analysis
- **MACD** - Moving Average Convergence Divergence indicator
- **RSI (Relative Strength Index)** - Momentum oscillator
- **Stochastic Oscillator** - Momentum indicator comparing closing price to price range
- **SMA (Simple Moving Average)** - Basic moving average calculation
- **Min/Max Keeper** - Efficient tracking of minimum and maximum values in a sliding window

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-talib = "0.1.0"
```

Or use it directly from git:

```toml
[dependencies]
rust-talib = { git = "https://github.com/300k-xyz/rust-talib" }
```

## Usage

### ATR (Average True Range)

```rust
use rust_talib::atr_keeper::AtrKeeper;

let mut atr = AtrKeeper::new(14, 60).unwrap();
atr.add(110.0, 100.0, 105.0); // high, low, close
atr.add(115.0, 105.0, 110.0);
let current_atr = atr.get();
```

### Bollinger Bands

```rust
use rust_talib::bollinger_band_keeper::BollingerBandKeeper;

let mut bb = BollingerBandKeeper::with_window(20, 2.0, None);
bb.add(100.0);
bb.add(101.0);
bb.add(102.0);

let is_above = bb.is_above_upper_band(105.0);
let is_below = bb.is_below_lower_band(95.0);
let is_inside = bb.is_inside_band(101.0);
```

### KDJ Indicator

```rust
use rust_talib::kdj_keeper::KdjKeeper;

let mut kdj = KdjKeeper::new(9, 3, 3); // fast_k, slow_k, slow_d
kdj.add(110.0, 100.0, 105.0).unwrap(); // high, low, close
let (k, d, j) = kdj.get();
let j_centered = kdj.get_j_centered();
```

### MACD

```rust
use rust_talib::macd_keeper::MacdKeeper;

let mut macd = MacdKeeper::new(26, 12, 9, 20, None); // slow, fast, dea, divergence_window
macd.add(100.0);
macd.add(101.0);
let has_cross = macd.check_cross();
let divergence = macd.check_divergence();
```

### RSI (Relative Strength Index)

```rust
use rust_talib::rsi_keeper::RsiKeeper;

let mut rsi = RsiKeeper::with_period(14);
rsi.add(100.0);
rsi.add(101.0);
rsi.add(102.0);
let current_rsi = rsi.get();
let prev_rsi = rsi.get_prev();
```

### Stochastic Oscillator

```rust
use rust_talib::stochastic_oscillator_keeper::StochasticOscillatorKeeper;

let mut stoch = StochasticOscillatorKeeper::new(14, 3); // k_period, d_period
stoch.add(100.0).unwrap();
stoch.add(101.0).unwrap();
let percent_k = stoch.get_percent_k();
let percent_d = stoch.get_percent_d();
let is_overbought = stoch.is_overbought();
let is_oversold = stoch.is_oversold();
```

### SMA (Simple Moving Average)

```rust
use rust_talib::sma_keeper::SmaKeeper;

let mut sma = SmaKeeper::new(20, 1000, 0.0); // period, time_gap_ms, initial_sma
sma.add(1000, 100.0); // timestamp, value
sma.add(2000, 101.0);
let current_sma = sma.get();
```

### Min/Max Keeper

```rust
use rust_talib::min_max_keeper::MinMaxKeeper;

let mut mm = MinMaxKeeper::with_capacity(10, 0.0001); // period, target_range
mm.add(100.0).unwrap();
mm.add(101.0).unwrap();
let max = mm.get_max();
let min = mm.get_min();
let mid = mm.get_mid();
```

## API Documentation

### AtrKeeper

- `new(period: usize, candle_period: usize) -> Result<Self, String>` - Create new ATR keeper
- `add(high: f64, low: f64, close: f64)` - Add new price data
- `get() -> f64` - Get current ATR value
- `peek_next(high: f64, low: f64) -> f64` - Preview next ATR value
- `fluctuant_index(day_average_atr: &HashMap<usize, f64>) -> f64` - Calculate fluctuant index

### BollingerBandKeeper

- `new() -> Self` - Create empty keeper
- `with_window(window_size: usize, std_dev_multiplier: f64, window_values: Option<Vec<f64>>) -> Self` - Create with parameters
- `add(value: f64)` - Add new price
- `is_above_upper_band(value: f64) -> bool` - Check if value is above upper band
- `is_below_lower_band(value: f64) -> bool` - Check if value is below lower band
- `is_inside_band(value: f64) -> bool` - Check if value is inside bands

### KdjKeeper

- `new(period_fast_k: usize, period_slow_k: usize, period_slow_d: usize) -> Self` - Create new keeper
- `add(high: f64, low: f64, close: f64) -> Result<(), String>` - Add price data
- `get() -> (f64, f64, f64)` - Get (K, D, J) values
- `get_j_centered() -> f64` - Get centered J value
- `is_over_bought_sold(over_bought_thresh: f64, over_sold_thresh: f64) -> f64` - Check overbought/oversold
- `is_cross_golden_death(cross_golden_thresh: f64, cross_death_thresh: f64) -> f64` - Check golden/death cross
- `is_peak_bottom(peak_thresh: f64, bottom_thresh: f64) -> f64` - Check peak/bottom

### MacdKeeper

- `new(slow_period: usize, fast_period: usize, dea_period: usize, divergen_wind: usize, prices: Option<Vec<f64>>) -> Self` - Create new keeper
- `add(price: f64)` - Add new price
- `check_cross() -> bool` - Check for MACD line cross
- `check_divergence() -> f64` - Check for divergence

### RsiKeeper

- `new() -> Self` - Create empty keeper
- `with_period(max_len: usize) -> Self` - Create with period
- `add(price: f64)` - Add new price
- `get() -> f64` - Get current RSI
- `get_prev() -> f64` - Get previous RSI

### StochasticOscillatorKeeper

- `new(k_period: usize, d_period: usize) -> Self` - Create new keeper
- `add(value: f64) -> Result<(), String>` - Add new price
- `get_percent_k() -> f64` - Get %K value
- `get_percent_d() -> f64` - Get %D value
- `is_overbought() -> bool` - Check if overbought (>80)
- `is_oversold() -> bool` - Check if oversold (<20)

## Testing

Run all tests:

```bash
cargo test
```

Run tests for a specific module:

```bash
cargo test --lib atr_keeper
```

## Performance

All indicators are designed for efficient real-time calculation with:
- O(1) or O(log n) complexity for most operations
- Minimal memory allocation
- Sliding window implementations using `VecDeque`

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Repository

https://github.com/300k-xyz/rust-talib

