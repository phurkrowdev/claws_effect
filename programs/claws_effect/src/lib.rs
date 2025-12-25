use anchor_lang::prelude::*;

declare_id!("6MDt7Xsgo7TAXmj41U6xdTgM1Yrou9Ksd49rLWJPxoiG");

#[program]
pub mod claws_effect {
    use super::*;

		pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        total_supply: u16,
        tier_caps: [u16; 7],
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.config;

        let sum: u32 = tier_caps.iter().map(|&x| x as u32).sum();
        require!(sum == total_supply as u32, ClawsError::TierCapsDoNotSumToTotal);

        cfg.version = Config::VERSION;
        cfg.bump = ctx.bumps.config;
        cfg.authority = ctx.accounts.authority.key();

        cfg.is_frozen = false;
        cfg.is_finalized = false;

        cfg.total_supply = total_supply;
        cfg.minted_total = 0;

        cfg.tier_caps = tier_caps;
        cfg.tier_minted = [0u16; 7];

        Ok(())
    }

    pub fn mint_seed(ctx: Context<MintSeed>, tier: u8) -> Result<()> {
        let cfg = &mut ctx.accounts.config;

        require!(!cfg.is_frozen, ClawsError::MintFrozen);
        require!(tier < 7, ClawsError::InvalidTier);

        let i = tier as usize;

        require!(cfg.minted_total < cfg.total_supply, ClawsError::SoldOut);
        require!(cfg.tier_minted[i] < cfg.tier_caps[i], ClawsError::TierSoldOut);

        cfg.minted_total += 1;
        cfg.tier_minted[i] += 1;

        Ok(())
    }

    pub fn freeze_mint_permanent(ctx: Context<FreezeMint>) -> Result<()> {
        let cfg = &mut ctx.accounts.config;

        require!(ctx.accounts.authority.key() == cfg.authority, ClawsError::Unauthorized);
        require!(!cfg.is_finalized, ClawsError::ScarcityFinalized);

        cfg.is_frozen = true;
        Ok(())
    }

    pub fn finalize_scarcity_law(ctx: Context<FinalizeScarcity>) -> Result<()> {
        let cfg = &mut ctx.accounts.config;

        require!(ctx.accounts.authority.key() == cfg.authority, ClawsError::Unauthorized);
        require!(!cfg.is_finalized, ClawsError::AlreadyFinalized);

        cfg.is_finalized = true;
        cfg.authority = Pubkey::default();

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Config::SPACE,
        seeds = [Config::SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintSeed<'info> {
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [Config::SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct FreezeMint<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [Config::SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct FinalizeScarcity<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [Config::SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[account]
pub struct Config {
    pub version: u8,
    pub bump: u8,

    pub authority: Pubkey,
    pub is_frozen: bool,
    pub is_finalized: bool,

    pub total_supply: u16,
    pub minted_total: u16,

    pub tier_caps: [u16; 7],
    pub tier_minted: [u16; 7],
}

impl Config {
    pub const VERSION: u8 = 1;
    pub const SEED: &'static [u8] = b"config";

    // 8  discriminator
    // 2  version + bump
    // 32 authority
    // 2  bools
    // 4  supply counters
    // 28 tier arrays
    pub const SPACE: usize = 8 + 68;
}

#[error_code]
pub enum ClawsError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Minting is frozen")]
    MintFrozen,
    #[msg("Invalid tier")]
    InvalidTier,
    #[msg("Sold out")]
    SoldOut,
    #[msg("Tier sold out")]
    TierSoldOut,
    #[msg("Scarcity finalized")]
    ScarcityFinalized,
    #[msg("Already finalized")]
    AlreadyFinalized,
    #[msg("Tier caps must sum to total supply")]
    TierCapsDoNotSumToTotal,
}

