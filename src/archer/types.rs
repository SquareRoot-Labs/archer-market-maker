use bytemuck::{Pod, Zeroable};
use solana_sdk::pubkey::Pubkey;

pub const PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("Archer8kgiavM61GyusMzaaS2ft5sALtNsD1HxkUPMhy");

pub const MAKER_BOOK_DISCRIMINATOR: [u8; 8] = *b"ACHRMKR1";
pub const MARKET_STATE_DISCRIMINATOR: [u8; 8] = *b"ACHRMKT1";
pub const MAKER_BOOK_SEED: &[u8] = b"maker";
pub const MAX_LEVELS: usize = 16;

pub const IX_INITIALIZE_MAKER_BOOK: u8 = 6;
pub const IX_UPDATE_BOOK: u8 = 7;
pub const IX_UPDATE_MID_PRICE: u8 = 8;
pub const IX_CLEAR_BOOK: u8 = 9;
pub const IX_MAKER_DEPOSIT: u8 = 11;
pub const IX_MAKER_WITHDRAW: u8 = 12;
pub const IX_UPDATE_EXPIRY_IN_SLOTS: u8 = 30;

pub const MAKER_LEVEL_SIZE: usize = core::mem::size_of::<MakerLevel>();

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct MakerLevel {
    pub size_in_base_lots: u64,
    pub price_offset_ticks: i64,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MakerBook {
    pub discriminator: [u8; 8],
    pub maker: Pubkey,
    pub market: Pubkey,
    pub delegate: Pubkey,
    pub mid_price_ticks: u64,
    pub quote_delta_per_tick: u64,
    pub min_reference_price: u64,
    pub quote_locked: u64,
    pub quote_free: u64,
    pub base_locked: u64,
    pub base_free: u64,
    pub status: u8,
    pub maker_book_bump: u8,
    pub sync_spread_ticks: u16,
    pub _status_padding: [u8; 4],
    pub last_updated_sequence_number: u64,
    pub total_bid_base_lots: u64,
    pub tick_conversion_num: u64,
    pub tick_conversion_den: u64,
    pub bid_levels: [MakerLevel; MAX_LEVELS],
    pub ask_levels: [MakerLevel; MAX_LEVELS],
    pub last_updated_slot: u64,
    pub expiry_in_slots: u64,
    pub _reserved: [u64; 6],
}

impl MakerBook {
    pub fn get_address(market: &Pubkey, maker: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[MAKER_BOOK_SEED, market.as_ref(), maker.as_ref()],
            &PROGRAM_ID,
        )
    }

    pub fn load(data: &[u8]) -> anyhow::Result<&Self> {
        let size = std::mem::size_of::<Self>();
        anyhow::ensure!(data.len() >= size, "MakerBook data too short: {} < {size}", data.len());
        let book: &Self = bytemuck::try_from_bytes(&data[..size])
            .map_err(|e| anyhow::anyhow!("MakerBook bytemuck: {e}"))?;
        anyhow::ensure!(book.discriminator == MAKER_BOOK_DISCRIMINATOR, "Invalid MakerBook discriminator");
        Ok(book)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MarketStateHeader {
    pub discriminator: [u8; 8],
    pub market_id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub admin: Pubkey,
    pub base_atoms_per_base_lot: u64,
    pub quote_atoms_per_quote_lot: u64,
    pub tick_size_in_quote_atoms_per_base_unit: u64,
    pub raw_base_units_per_base_unit: u64,
    pub uncollected_fees_quote_lots: u64,
    pub collected_fees_quote_lots: u64,
    pub maker_fee_ppm: i32,
    pub taker_fee_ppm: i32,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub status: u8,
    pub mode: u8,
    pub market_bump: u8,
    pub sync_fee_multiplier: u8,
    pub min_async_delay_slots: u16,
    pub max_async_delay_slots: u16,
    pub limit_order_fee_ppm: u32,
}

unsafe impl Pod for MarketStateHeader {}
unsafe impl Zeroable for MarketStateHeader {}

impl MarketStateHeader {
    pub fn load(data: &[u8]) -> anyhow::Result<&Self> {
        let size = std::mem::size_of::<Self>();
        anyhow::ensure!(data.len() >= size, "MarketState data too short: {} < {size}", data.len());
        let header: &Self = bytemuck::try_from_bytes(&data[..size])
            .map_err(|e| anyhow::anyhow!("MarketState bytemuck: {e}"))?;
        anyhow::ensure!(header.discriminator == MARKET_STATE_DISCRIMINATOR, "Invalid MarketState discriminator");
        Ok(header)
    }
}
