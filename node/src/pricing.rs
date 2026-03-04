//! Provider auto-pricing — market-adaptive fee adjustment.
//!
//! NODE-015: Automatically adjusts a provider's inference price based on
//! local utilization, network-wide market signals, and configurable bounds.
//!
//! Strategy: EMA-based price tracking of market clearing prices combined
//! with local utilization pressure. When utilization is high, price rises
//! toward the market ceiling; when low, price drops toward the floor to
//! attract jobs. Supports multiple pricing strategies.

use std::collections::VecDeque;

/// Pricing strategy selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PricingStrategy {
    /// Fixed price — never adjusts.
    Fixed,
    /// Track market EMA and adjust based on utilization.
    MarketAdaptive,
    /// Aggressive undercut — price slightly below market EMA.
    Undercut,
    /// Premium — price above market for high-reputation providers.
    Premium,
}

/// Configuration for the auto-pricer.
#[derive(Debug, Clone)]
pub struct PricingConfig {
    /// Minimum price (floor) — never go below this.
    pub floor: u128,
    /// Maximum price (ceiling) — never exceed this.
    pub ceiling: u128,
    /// Initial price before any market data.
    pub initial_price: u128,
    /// EMA smoothing factor in basis points (0–10000).
    /// Higher = more responsive to recent prices.
    pub ema_alpha_bps: u64,
    /// Utilization target as percentage (0–100).
    /// Below target → lower price; above → raise price.
    pub utilization_target_pct: u64,
    /// Price adjustment speed in basis points per epoch.
    /// How fast price moves toward the target.
    pub adjustment_rate_bps: u64,
    /// Undercut margin in basis points (for Undercut strategy).
    pub undercut_margin_bps: u64,
    /// Premium markup in basis points (for Premium strategy).
    pub premium_markup_bps: u64,
    /// Strategy to use.
    pub strategy: PricingStrategy,
    /// Maximum market price samples to retain.
    pub max_samples: usize,
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            floor: 100,
            ceiling: 100_000,
            initial_price: 1_000,
            ema_alpha_bps: 2000,  // α = 0.2
            utilization_target_pct: 70,
            adjustment_rate_bps: 500, // 5% per epoch
            undercut_margin_bps: 300, // 3% below market
            premium_markup_bps: 1500, // 15% above market
            strategy: PricingStrategy::MarketAdaptive,
            max_samples: 1000,
        }
    }
}

/// A market price observation (clearing price from a completed job).
#[derive(Debug, Clone, Copy)]
pub struct PriceSample {
    pub epoch: u64,
    pub price: u128,
}

/// The auto-pricer state machine.
#[derive(Debug)]
pub struct AutoPricer {
    config: PricingConfig,
    /// Current computed price.
    current_price: u128,
    /// EMA of observed market prices.
    market_ema: Option<u128>,
    /// Recent price samples for statistics.
    samples: VecDeque<PriceSample>,
    /// Current utilization (0–10000 basis points).
    utilization_bps: u64,
    /// Total jobs completed (for statistics).
    jobs_completed: u64,
    /// Total revenue earned.
    total_revenue: u128,
    /// Last epoch we adjusted price.
    last_adjustment_epoch: u64,
}

impl AutoPricer {
    pub fn new(config: PricingConfig) -> Self {
        let initial = config.initial_price.clamp(config.floor, config.ceiling);
        Self {
            current_price: initial,
            market_ema: None,
            samples: VecDeque::new(),
            utilization_bps: 0,
            jobs_completed: 0,
            total_revenue: 0,
            last_adjustment_epoch: 0,
            config,
        }
    }

    /// Get the current recommended price.
    pub fn price(&self) -> u128 {
        self.current_price
    }

    /// Get the market EMA if enough data.
    pub fn market_ema(&self) -> Option<u128> {
        self.market_ema
    }

    /// Get statistics.
    pub fn stats(&self) -> PricerStats {
        PricerStats {
            current_price: self.current_price,
            market_ema: self.market_ema,
            utilization_bps: self.utilization_bps,
            jobs_completed: self.jobs_completed,
            total_revenue: self.total_revenue,
            sample_count: self.samples.len(),
        }
    }

    /// Feed a market price observation (e.g., clearing price of a nearby job).
    pub fn observe_market_price(&mut self, sample: PriceSample) {
        // Update EMA
        let alpha = self.config.ema_alpha_bps as u128;
        let inv_alpha = 10000u128.saturating_sub(alpha);
        self.market_ema = Some(match self.market_ema {
            Some(prev) => {
                // new_ema = α * sample + (1 - α) * prev
                let weighted_new = sample.price.saturating_mul(alpha);
                let weighted_old = prev.saturating_mul(inv_alpha);
                weighted_new.saturating_add(weighted_old) / 10000
            }
            None => sample.price,
        });

        // Store sample
        self.samples.push_back(sample);
        while self.samples.len() > self.config.max_samples {
            self.samples.pop_front();
        }
    }

    /// Update utilization metric.
    /// `active_jobs` / `capacity` expressed as 0–10000 bps.
    pub fn update_utilization(&mut self, active_jobs: u32, capacity: u32) {
        self.utilization_bps = if capacity == 0 {
            0
        } else {
            ((active_jobs as u64) * 10000 / (capacity as u64)).min(10000)
        };
    }

    /// Record a completed job (for revenue tracking).
    pub fn record_completion(&mut self, price_paid: u128) {
        self.jobs_completed += 1;
        self.total_revenue = self.total_revenue.saturating_add(price_paid);
    }

    /// Run the pricing adjustment for the current epoch.
    /// Returns the new price.
    pub fn adjust(&mut self, current_epoch: u64) -> u128 {
        if self.config.strategy == PricingStrategy::Fixed {
            return self.current_price;
        }

        // Avoid adjusting multiple times in same epoch
        if current_epoch <= self.last_adjustment_epoch {
            return self.current_price;
        }
        self.last_adjustment_epoch = current_epoch;

        let target_price = self.compute_target_price();
        let rate = self.config.adjustment_rate_bps as u128;

        // Move current_price toward target_price by adjustment_rate
        self.current_price = if self.current_price < target_price {
            let delta = target_price.saturating_sub(self.current_price);
            let step = delta.saturating_mul(rate) / 10000;
            self.current_price.saturating_add(step.max(1))
        } else if self.current_price > target_price {
            let delta = self.current_price.saturating_sub(target_price);
            let step = delta.saturating_mul(rate) / 10000;
            self.current_price.saturating_sub(step.max(1))
        } else {
            self.current_price
        };

        // Clamp to bounds
        self.current_price = self.current_price.clamp(self.config.floor, self.config.ceiling);
        self.current_price
    }

    /// Compute the ideal target price given strategy, utilization, and market.
    fn compute_target_price(&self) -> u128 {
        let base = self.market_ema.unwrap_or(self.config.initial_price);

        // Apply strategy modifier
        let strategy_price = match self.config.strategy {
            PricingStrategy::Fixed => return self.current_price,
            PricingStrategy::MarketAdaptive => base,
            PricingStrategy::Undercut => {
                let margin = base.saturating_mul(self.config.undercut_margin_bps as u128) / 10000;
                base.saturating_sub(margin)
            }
            PricingStrategy::Premium => {
                let markup = base.saturating_mul(self.config.premium_markup_bps as u128) / 10000;
                base.saturating_add(markup)
            }
        };

        // Apply utilization pressure
        let target_bps = self.config.utilization_target_pct * 100; // convert pct to bps
        if self.utilization_bps > target_bps {
            // Over-utilized: push price up proportionally
            let excess = self.utilization_bps - target_bps;
            let boost = strategy_price.saturating_mul(excess as u128) / 10000;
            strategy_price.saturating_add(boost)
        } else {
            // Under-utilized: push price down proportionally
            let deficit = target_bps - self.utilization_bps;
            let discount = strategy_price.saturating_mul(deficit as u128) / 10000;
            strategy_price.saturating_sub(discount)
        }
    }

    /// Get price percentiles from recent samples.
    pub fn percentiles(&self) -> Option<PricePercentiles> {
        if self.samples.is_empty() {
            return None;
        }
        let mut prices: Vec<u128> = self.samples.iter().map(|s| s.price).collect();
        prices.sort();
        let n = prices.len();
        Some(PricePercentiles {
            p10: prices[n / 10],
            p25: prices[n / 4],
            p50: prices[n / 2],
            p75: prices[n * 3 / 4],
            p90: prices[n * 9 / 10],
            min: prices[0],
            max: prices[n - 1],
            count: n,
        })
    }
}

/// Price distribution statistics.
#[derive(Debug, Clone)]
pub struct PricePercentiles {
    pub p10: u128,
    pub p25: u128,
    pub p50: u128,
    pub p75: u128,
    pub p90: u128,
    pub min: u128,
    pub max: u128,
    pub count: usize,
}

/// Summary stats from the pricer.
#[derive(Debug, Clone)]
pub struct PricerStats {
    pub current_price: u128,
    pub market_ema: Option<u128>,
    pub utilization_bps: u64,
    pub jobs_completed: u64,
    pub total_revenue: u128,
    pub sample_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_pricer() -> AutoPricer {
        AutoPricer::new(PricingConfig::default())
    }

    #[test]
    fn test_initial_price() {
        let p = default_pricer();
        assert_eq!(p.price(), 1_000);
        assert_eq!(p.market_ema(), None);
    }

    #[test]
    fn test_initial_price_clamped_to_floor() {
        let config = PricingConfig {
            floor: 500,
            initial_price: 100,
            ..Default::default()
        };
        let p = AutoPricer::new(config);
        assert_eq!(p.price(), 500);
    }

    #[test]
    fn test_initial_price_clamped_to_ceiling() {
        let config = PricingConfig {
            ceiling: 800,
            initial_price: 5000,
            ..Default::default()
        };
        let p = AutoPricer::new(config);
        assert_eq!(p.price(), 800);
    }

    #[test]
    fn test_market_ema_first_observation() {
        let mut p = default_pricer();
        p.observe_market_price(PriceSample { epoch: 1, price: 2000 });
        assert_eq!(p.market_ema(), Some(2000));
    }

    #[test]
    fn test_market_ema_converges() {
        let mut p = default_pricer();
        // Feed constant price, EMA should converge
        for i in 0..50 {
            p.observe_market_price(PriceSample { epoch: i, price: 5000 });
        }
        // With α=0.2, after many samples EMA → 5000
        assert_eq!(p.market_ema(), Some(5000));
    }

    #[test]
    fn test_fixed_strategy_no_change() {
        let config = PricingConfig {
            strategy: PricingStrategy::Fixed,
            initial_price: 777,
            ..Default::default()
        };
        let mut p = AutoPricer::new(config);
        p.observe_market_price(PriceSample { epoch: 1, price: 5000 });
        p.update_utilization(10, 10); // 100% utilized
        let price = p.adjust(1);
        assert_eq!(price, 777);
    }

    #[test]
    fn test_price_rises_when_overutilized() {
        let mut p = default_pricer();
        p.observe_market_price(PriceSample { epoch: 0, price: 1000 });
        p.update_utilization(9, 10); // 90% → above 70% target
        let new_price = p.adjust(1);
        assert!(new_price > 1000, "price should rise: {new_price}");
    }

    #[test]
    fn test_price_drops_when_underutilized() {
        let mut p = default_pricer();
        p.observe_market_price(PriceSample { epoch: 0, price: 1000 });
        p.update_utilization(2, 10); // 20% → well below 70% target
        let new_price = p.adjust(1);
        assert!(new_price < 1000, "price should drop: {new_price}");
    }

    #[test]
    fn test_undercut_strategy() {
        let config = PricingConfig {
            strategy: PricingStrategy::Undercut,
            undercut_margin_bps: 1000, // 10% below
            ..Default::default()
        };
        let mut p = AutoPricer::new(config);
        // Feed market price, set utilization at target
        for i in 0..20 {
            p.observe_market_price(PriceSample { epoch: i, price: 2000 });
        }
        p.update_utilization(7, 10); // exactly at 70% target
        // Run many adjustments to converge
        let mut price = 0;
        for epoch in 1..200 {
            price = p.adjust(epoch);
        }
        // Should converge near 1800 (2000 - 10%)
        assert!(price >= 1700 && price <= 1900, "undercut price: {price}");
    }

    #[test]
    fn test_premium_strategy() {
        let config = PricingConfig {
            strategy: PricingStrategy::Premium,
            premium_markup_bps: 2000, // 20% above
            ..Default::default()
        };
        let mut p = AutoPricer::new(config);
        for i in 0..20 {
            p.observe_market_price(PriceSample { epoch: i, price: 2000 });
        }
        p.update_utilization(7, 10);
        let mut price = 0;
        for epoch in 1..200 {
            price = p.adjust(epoch);
        }
        // Should converge near 2400 (2000 + 20%)
        assert!(price >= 2300 && price <= 2500, "premium price: {price}");
    }

    #[test]
    fn test_price_clamped_to_floor() {
        let config = PricingConfig {
            floor: 500,
            initial_price: 600,
            ..Default::default()
        };
        let mut p = AutoPricer::new(config);
        p.observe_market_price(PriceSample { epoch: 0, price: 100 });
        p.update_utilization(0, 10); // 0% utilization → massive downward pressure
        for epoch in 1..100 {
            p.adjust(epoch);
        }
        assert!(p.price() >= 500, "price below floor: {}", p.price());
    }

    #[test]
    fn test_price_clamped_to_ceiling() {
        let config = PricingConfig {
            ceiling: 5000,
            initial_price: 4000,
            ..Default::default()
        };
        let mut p = AutoPricer::new(config);
        p.observe_market_price(PriceSample { epoch: 0, price: 50000 });
        p.update_utilization(10, 10); // 100% utilization
        for epoch in 1..100 {
            p.adjust(epoch);
        }
        assert!(p.price() <= 5000, "price above ceiling: {}", p.price());
    }

    #[test]
    fn test_revenue_tracking() {
        let mut p = default_pricer();
        p.record_completion(1000);
        p.record_completion(2000);
        p.record_completion(1500);
        let stats = p.stats();
        assert_eq!(stats.jobs_completed, 3);
        assert_eq!(stats.total_revenue, 4500);
    }

    #[test]
    fn test_percentiles() {
        let mut p = default_pricer();
        for i in 1..=100 {
            p.observe_market_price(PriceSample { epoch: i, price: i as u128 * 10 });
        }
        let pct = p.percentiles().unwrap();
        assert_eq!(pct.min, 10);
        assert_eq!(pct.max, 1000);
        assert_eq!(pct.count, 100);
        assert_eq!(pct.p50, 510); // index 50 of 100 → price 510
    }

    #[test]
    fn test_no_double_adjust_same_epoch() {
        let mut p = default_pricer();
        p.observe_market_price(PriceSample { epoch: 0, price: 5000 });
        p.update_utilization(9, 10);
        let first = p.adjust(1);
        let second = p.adjust(1); // same epoch
        assert_eq!(first, second);
    }

    #[test]
    fn test_utilization_zero_capacity() {
        let mut p = default_pricer();
        p.update_utilization(5, 0);
        assert_eq!(p.utilization_bps, 0);
    }

    #[test]
    fn test_sample_eviction() {
        let config = PricingConfig {
            max_samples: 5,
            ..Default::default()
        };
        let mut p = AutoPricer::new(config);
        for i in 0..10 {
            p.observe_market_price(PriceSample { epoch: i, price: i as u128 * 100 });
        }
        assert_eq!(p.samples.len(), 5);
        // Oldest should be evicted — first remaining is epoch 5
        assert_eq!(p.samples.front().unwrap().epoch, 5);
    }
}
