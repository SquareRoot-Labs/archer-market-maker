use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::config::FeedSettings;
use crate::state::{SharedState, now_us};
use crate::volatility::VolatilityTracker;

#[derive(Deserialize)]
struct CoinbaseTicker {
    price: String,
}

pub async fn run_feed(
    state: Arc<SharedState>,
    config: FeedSettings,
    vol_window: usize,
    cancel: CancellationToken,
) {
    let url = format!(
        "https://api.coinbase.com/v2/prices/{}/spot",
        config.coinbase_product_id
    );
    let interval = Duration::from_millis(config.poll_interval_ms);
    let client = reqwest::Client::new();
    let mut vol_tracker = VolatilityTracker::new(vol_window);

    tracing::info!(
        product = %config.coinbase_product_id,
        poll_ms = config.poll_interval_ms,
        vol_window,
        "Price feed starting"
    );

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return,
            _ = tokio::time::sleep(interval) => {}
        }

        match fetch_price(&client, &url).await {
            Ok(price) => {
                state.mid_price.store(price, Relaxed);
                state.price_timestamp_us.store(now_us(), Relaxed);

                vol_tracker.push(price);
                state.volatility_bps.store(vol_tracker.realized_vol_bps(), Relaxed);

                if !state.feed_alive.load(Relaxed) {
                    state.feed_alive.store(true, Relaxed);
                    tracing::info!(price, "Feed connected");
                }
            }
            Err(e) => {
                tracing::warn!("Price fetch failed: {e}");
                state.feed_alive.store(false, Relaxed);
            }
        }
    }
}

#[derive(Deserialize)]
struct CoinbaseResponse {
    data: CoinbaseTicker,
}

async fn fetch_price(client: &reqwest::Client, url: &str) -> anyhow::Result<f64> {
    let resp: CoinbaseResponse = client
        .get(url)
        .header("User-Agent", "archer-market-maker")
        .send()
        .await?
        .json()
        .await?;
    let price: f64 = resp.data.price.parse()?;
    anyhow::ensure!(price > 0.0 && price.is_finite(), "invalid price: {price}");
    Ok(price)
}
