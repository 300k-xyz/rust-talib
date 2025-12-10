use crate::trade_price_keeper::TradePriceKeeper;

/// Trade side constants
pub const BUY: bool = true;
pub const SELL: bool = false;

/// Calculates the standard deviation (not variance, despite the name) of prices
/// in the given range from the TradePriceKeeper.
/// 
/// # Arguments
/// * `price_keeper` - The TradePriceKeeper containing price history
/// * `start_index` - Starting index (can be negative for reverse indexing)
/// * `end_index` - Ending index (can be negative for reverse indexing)
/// * `mean` - The mean value to use for variance calculation
/// 
/// # Returns
/// The standard deviation (square root of variance)
/// 
/// # Panics
/// Panics if end_index <= start_index
pub fn get_variance(
    price_keeper: &TradePriceKeeper,
    start_index: i64,
    end_index: i64,
    mean: f64,
) -> f64 {
    let size = price_keeper.get_history_prices_size();
    
    if size == 0 {
        return 0.0;
    }

    // Convert negative indices to positive
    let start = if start_index < 0 {
        (size as i64 + start_index) as usize
    } else {
        start_index as usize
    };
    
    let end = if end_index < 0 {
        (size as i64 + end_index) as usize
    } else {
        end_index as usize
    };

    if end <= start {
        panic!("get_variance end_index <= start_index");
    }

    let mut total_diff = 0.0;
    
    for index in start..end {
        let price = price_keeper.get_history_price(index as i64);
        let diff = price - mean;
        total_diff += diff * diff;
    }

    let variance = total_diff / (end - start) as f64;
    variance.sqrt()
}
