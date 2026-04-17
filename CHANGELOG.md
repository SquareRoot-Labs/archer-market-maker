# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2026-04-17]

### Added
- `set-expiry` CLI command. Calls the on-chain `UpdateExpiryInSlots` instruction (discriminator `30`) to set `MakerBook.expiry_in_slots`. `--slots 0` disables the aggregator's expiry-skip check.
- `MakerBook` now decodes the new trailing fields `last_updated_slot`, `expiry_in_slots`, and reserved padding added by the on-chain layout resize.

### Changed
- Maker deposit/withdraw instructions now pass `market_account` as readonly, matching the on-chain program's updated account requirements.
- Bumped compute-unit limits: `UpdateMidPrice` 750 → 850, `UpdateBook` 5500 → 5600.
- README CU table updated to reflect the real per-instruction budgets used by the engine (`~180` / `~400` / `~5,000`).
