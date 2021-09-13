use std::io::Write;

use anchor_lang::prelude::*;
use anchor_lang::Key;

use crate::state::*;

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(zero)]
    pub market: Loader<'info, Market>,
}

/// Initialize a new empty market with a given owner.
pub fn handler(
    ctx: Context<InitializeMarket>,
    owner: Pubkey,
    quote_currency: String,
    quote_token_mint: Pubkey,
) -> ProgramResult {
    let market_address = ctx.accounts.market.key();
    let initial_seeds = &[ctx.accounts.market.to_account_info().key.as_ref()];

    let mut market = ctx.accounts.market.load_init()?;

    let (authority, authority_seed) = Pubkey::find_program_address(initial_seeds, ctx.program_id);

    market.version = 0;
    market.owner = owner;
    market.market_authority = authority;
    market.authority_seed = market_address;
    market.authority_bump_seed = [authority_seed];
    market.quote_token_mint = quote_token_mint;
    (&mut market.quote_currency[..]).write(quote_currency.as_bytes())?;

    msg!("market initialized with currency {}", quote_currency);

    Ok(())
}
