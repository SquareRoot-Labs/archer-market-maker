/// Tracks realized volatility using log returns over a rolling window.
///
/// Feed pushes each new price; the tracker maintains a ring buffer and
/// computes the standard deviation of log(p_t / p_{t-1}) on demand.
pub struct VolatilityTracker {
    prices: Vec<f64>,
    head: usize,
    count: usize,
    capacity: usize,
}

impl VolatilityTracker {
    pub fn new(window: usize) -> Self {
        Self {
            prices: vec![0.0; window],
            head: 0,
            count: 0,
            capacity: window,
        }
    }

    /// Record a new price sample.
    pub fn push(&mut self, price: f64) {
        if !price.is_finite() || price <= 0.0 {
            return;
        }
        self.prices[self.head] = price;
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Realized volatility = std dev of log returns over the window.
    /// Returns 0.0 if fewer than 2 samples.
    pub fn realized_vol(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }

        let n_returns = self.count - 1;

        // Walk the ring buffer in chronological order to compute log returns.
        let start = if self.count < self.capacity {
            0
        } else {
            self.head // oldest sample
        };

        let mut sum = 0.0;
        let mut sum_sq = 0.0;

        let mut prev = self.prices[start % self.capacity];
        for i in 1..self.count {
            let idx = (start + i) % self.capacity;
            let cur = self.prices[idx];
            let lr = (cur / prev).ln();
            sum += lr;
            sum_sq += lr * lr;
            prev = cur;
        }

        let mean = sum / n_returns as f64;
        let variance = (sum_sq / n_returns as f64) - mean * mean;
        variance.max(0.0).sqrt()
    }

    /// Realized vol expressed in basis points (1 bps = 0.0001).
    pub fn realized_vol_bps(&self) -> f64 {
        self.realized_vol() * 10_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tracker_returns_zero() {
        let t = VolatilityTracker::new(300);
        assert_eq!(t.realized_vol(), 0.0);
    }

    #[test]
    fn single_price_returns_zero() {
        let mut t = VolatilityTracker::new(300);
        t.push(100.0);
        assert_eq!(t.realized_vol(), 0.0);
    }

    #[test]
    fn constant_price_returns_zero() {
        let mut t = VolatilityTracker::new(300);
        for _ in 0..50 {
            t.push(100.0);
        }
        assert!(t.realized_vol() < 1e-15);
    }

    #[test]
    fn known_volatility() {
        let mut t = VolatilityTracker::new(10);
        let prices = [100.0, 101.0, 99.5, 100.5, 102.0, 101.0, 100.0, 99.0, 100.0, 101.0];
        for p in &prices {
            t.push(*p);
        }
        let vol = t.realized_vol();
        assert!(vol > 0.0, "vol should be positive for varying prices");
        assert!(vol < 0.05, "vol should be reasonable for ~1% moves");
    }

    #[test]
    fn wraps_around_ring_buffer() {
        let mut t = VolatilityTracker::new(5);
        // Push 10 prices — only last 5 should be kept
        for i in 1..=10 {
            t.push(100.0 + i as f64);
        }
        assert_eq!(t.count, 5);
        let vol = t.realized_vol();
        assert!(vol > 0.0);
    }

    #[test]
    fn ignores_invalid_prices() {
        let mut t = VolatilityTracker::new(300);
        t.push(100.0);
        t.push(f64::NAN);
        t.push(-5.0);
        t.push(0.0);
        t.push(f64::INFINITY);
        assert_eq!(t.count, 1);
    }
}
