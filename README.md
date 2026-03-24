# Archer Market Maker

[Archer](https://archer.exchange) is a fully on-chain order book exchange on Solana that eliminates adverse selection faced by market makers through sovereign maker books, parametric pricing, and pro-rata execution. Instead of a single shared order book, each market maker owns their own on-chain book — enabling zero write-lock contention, O(1) repricing, and incentives that reward depth over speed. [Read more about how Archer works](https://x.com/mmdhrumil/status/2026301400158810390).

A simple market maker for the Archer Exchange.

Places bid and ask orders on an Archer on-chain orderbook using Binance WebSocket prices as a reference, with optional cross-tick synthetic pricing. Designed to be **easy to understand** and **a starting point** for building your own strategy.

## How It Works

The bot runs a loop every 200ms:

1. **Fetch price** — streams live best bid/ask via Binance WebSocket
2. **Compute quotes** — places 8 bid/ask levels at volatility-adjusted bps offsets from mid
3. **Send transaction** — picks the cheapest Solana instruction type to update the on-chain book

```
Binance WebSocket                     Archer Exchange
  (live book ticker)                   (on-chain orderbook)
       │                                      ▲
       ▼                                      │
  ┌──────────┐      ┌──────────┐      ┌────────────────┐
  │  Feed    │ ──▶  │  Engine  │ ──▶  │  TX Sender     │
  │ (stream) │      │ (loop)   │      │ (fire & forget)│
  └──────────┘      └──────────┘      └────────────────┘
                         │
                    Strategy
                 (vol-adjusted spreads)
```

### What gets placed on the book

Spreads widen automatically when volatility is high. The strategy tracks realized volatility (standard deviation of log returns) over the last 300 price samples and scales all spread levels by a multiplier:

```
  multiplier = max(1.0, realized_vol / baseline_vol)    (capped at vol_max_multiplier)
```

In calm markets (vol at or below baseline), spreads stay as configured. When vol rises above baseline, all levels widen proportionally:

```
  Asks:  mid + 25 bps × vol_mult  ─── Level 8
         mid + 20 bps × vol_mult  ─── Level 7
         mid + 15 bps × vol_mult  ─── Level 6
         mid + 12 bps × vol_mult  ─── Level 5
         mid + 10 bps × vol_mult  ─── Level 4
         mid +  7 bps × vol_mult  ─── Level 3
         mid +  5 bps × vol_mult  ─── Level 2
         mid +  2 bps × vol_mult  ─── Level 1 (tightest)
  ────── Mid price ──────────────────────────────
  Bids:  mid -  2 bps × vol_mult  ─── Level 1 (tightest)
         mid -  5 bps × vol_mult  ─── Level 2
         mid -  7 bps × vol_mult  ─── Level 3
         mid - 10 bps × vol_mult  ─── Level 4
         mid - 12 bps × vol_mult  ─── Level 5
         mid - 15 bps × vol_mult  ─── Level 6
         mid - 20 bps × vol_mult  ─── Level 7
         mid - 25 bps × vol_mult  ─── Level 8
```

Each level quotes an equal share of your deposited inventory.

### CU Optimization

Solana transactions cost compute units. The bot detects what changed since last cycle and picks the cheapest instruction:

| Instruction | CU Cost | When |
|-------------|---------|------|
| `UpdateMidPrice` | ~200 | Price moved but level structure unchanged (most cycles) |
| `UpdateBook` | ~4,500 | Level sizes or count changed |
| `ClearBook` | ~180 | Shutdown, error, or stale feed |

In practice, **~90% of cycles use the cheap mid-only path**, saving ~85% of CU.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs) 1.85+
- [Solana CLI](https://docs.anza.xyz/cli/install)
- An RPC endpoint ([Helius](https://helius.dev), [Triton](https://triton.one), or [QuickNode](https://quicknode.com))
- A funded Solana wallet

### 1. Build

```bash
git clone https://github.com/ArcherExchange/archer-market-maker.git
cd archer-market-maker
cargo build --release
```

### 2. Configure

Edit `config/default.toml`:

```toml
[market]
market_pubkey = "YOUR_MARKET_PUBKEY"
maker_keypair_path = "~/.config/solana/id.json"

[connection]
rpc_url = "https://mainnet.helius-rpc.com?api-key=YOUR_KEY"

[feed]
binance_symbol = "SOLUSDT"
# Optional: derive a synthetic pair via cross-tick division
# cross_symbol = "BTCUSDT"   # price = SOLUSDT / BTCUSDT
```

### 3. Initialize and deposit

```bash
# Create your maker book on-chain (one-time)
cargo run --release -- init

# Deposit tokens (example: 5 SOL + 750 USDC)
cargo run --release -- deposit --base 5.0 --quote 750.0
```

### 4. Run

```bash
# Dry run first (no real transactions)
cargo run --release -- run --shadow

# Run for real
cargo run --release -- run
```

### 5. Stop

```bash
# Ctrl+C — clears the book on shutdown

# Or emergency kill from another terminal
cargo run --release -- kill
```

## CLI Commands

```
archer-market-maker <COMMAND>

  run       Start the market maker
  init      Initialize maker book on-chain (one-time)
  deposit   Deposit base + quote tokens
  withdraw  Withdraw all funds
  kill      Emergency: clear all orders immediately
  status    Print on-chain book state
```

## Configuration

All settings in `config/default.toml`:

| Section | Key | Default | Description |
|---------|-----|---------|-------------|
| `market` | `market_pubkey` | — | Archer market public key |
| `market` | `maker_keypair_path` | — | Path to Solana keypair |
| `connection` | `rpc_url` | — | Solana RPC endpoint |
| `feed` | `binance_symbol` | — | Binance symbol (e.g. `SOLUSDT`) |
| `feed` | `cross_symbol` | `""` | Cross pair for synthetic pricing (e.g. `BTCUSDT`) |
| `feed` | `binance_ws_url` | `wss://stream.binance.com:9443/ws` | Binance WebSocket endpoint |
| `feed` | `staleness_timeout_ms` | `5000` | Pull quotes if feed stale |
| `strategy` | `spread_levels_bps` | `[2,5,7,10,12,15,20,25]` | Base bps offset per level |
| `strategy` | `inventory_pct` | `80` | % of inventory to quote |
| `strategy` | `vol_window` | `300` | Rolling window size (price samples) for volatility |
| `strategy` | `vol_baseline_bps` | `5.0` | Per-sample vol (bps) at which spreads are unchanged |
| `strategy` | `vol_max_multiplier` | `5.0` | Maximum spread multiplier from vol scaling |
| `execution` | `loop_interval_ms` | `200` | Engine cycle time |
| `execution` | `priority_fee_microlamports` | `100` | Solana priority fee |
| `execution` | `shadow_mode` | `false` | Dry run mode |
| `monitoring` | `log_level` | `info` | Log verbosity |

## Project Structure

```
src/
├── main.rs          CLI + orchestration
├── config.rs        TOML config
├── feed.rs          Binance WebSocket price feed (with cross-tick support)
├── strategy.rs      Vol-adjusted spread levels + CU optimization
├── volatility.rs    Realized vol tracker (log returns, ring buffer)
├── engine.rs        Core loop: price → strategy → TX
├── state.rs         Shared atomic state
├── tx.rs            Fire-and-forget TX sender
└── archer/          Self-contained Archer protocol client
    ├── types.rs     On-chain account layouts (MakerBook, MarketStateHeader)
    ├── config.rs    MarketConfig with conversion factors
    ├── math.rs      Price/lot conversions + book update builder
    ├── ix_builder.rs  Instruction builders for all maker operations
    ├── accounts.rs  Account parsing + balance helpers
    └── client.rs    High-level RPC client
```

## Adding Your Own Strategy

Edit `strategy.rs`. The `compute()` method takes a mid price and inventory, returns a `QuoteDecision`. The engine and TX layers don't change.

Ideas to try:
- Lean quotes based on inventory (shift mid toward the side you want to offload)
- Add multiple price sources and take the median

## License

Apache-2.0
