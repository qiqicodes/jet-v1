// SPDX-License-Identifier: AGPL-3.0-or-later

// Copyright (C) 2021 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token::{self, Burn, CloseAccount, Transfer};

use crate::state::*;
use crate::Rounding;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CloseLoanAccount<'info> {
    /// The relevant market this loan is for
    #[account(has_one = market_authority)]
    pub market: Loader<'info, Market>,

    /// The market's authority account
    pub market_authority: AccountInfo<'info>,

    /// The obligation the loan account is used for
    #[account(mut,
              has_one = market,
              has_one = owner)]
    pub obligation: Loader<'info, Obligation>,

    /// The reserve that the collateral comes from
    #[account(has_one = market,
              has_one = vault,
              has_one = deposit_note_mint)]
    pub reserve: Loader<'info, Reserve>,

    /// The reserve's vault where any tokens to withdraw will be transferred from
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    /// The mint for the loan notes being used as loan
    pub loan_note_mint: AccountInfo<'info>,

    /// The user/authority that owns the loan
    #[account(mut, signer)]
    pub owner: AccountInfo<'info>,

    /// The account that will store the loan notes
    #[account(mut,
              seeds = [
                  b"loan".as_ref(),
                  reserve.key().as_ref(),
                  obligation.key().as_ref(),
                  owner.key.as_ref()
              ],
              bump = bump)]
    pub loan_account: AccountInfo<'info>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,

    // TODO: needs debug
    #[account(mut, close = owner)]
    pub closing_account: ProgramAccount<'info, anchor_spl::token::ID>,
}

// TODO: Double Check: 
impl<'info> CloseLoanAccount<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            Transfer {
                from: self.vault.to_account_info(),
                to: self.owner.to_account_info(),
                authority: self.market_authority.clone(),
            },
        )
    }

    fn note_burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            Burn {
                to: self.owner.to_account_info(),
                mint: self.loan_note_mint.to_account_info(),
                authority: self.market_authority.clone(),
            },
        )
    }

    fn close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            CloseAccount {
                account: self.loan_account.to_account_info(),
                destination: self.owner.to_account_info(),
                authority: self.market_authority.clone(),
            },
        )
    }
}

// TODO: Close an account that can be used to store loan notes to represent debt in an obligation
pub fn handler(ctx: Context<CloseLoanAccount>, _bump: u8) -> ProgramResult {
    let obligation = ctx.accounts.obligation.load()?;
    let market = ctx.accounts.market.load()?;

    //  TODO: verify if any loan note is in loan account, if no, then proceed with closing the obligation account.
    // Pay all loans before we can close
    let notes_remaining = token::accessor::amount(&ctx.accounts.loan_account)?;

    /// TODO: comment for doc
    if notes_remaining > 0 {
        // TODO: verify if loans is empty, then proceed 
        // (need to write this function if doesn't exist)
        verify_loan_empty()?;

        let mut reserve = ctx.accounts.reserve.load_mut()?;
        let clock = Clock::get()?;

        // TODO: work this to fit loan_notes
        let reserve_info = market.reserves().get_cached(reserve.index, clock.slot);
        let tokens_to_withdraw =
            reserve_info.deposit_notes_to_tokens(notes_remaining, Rounding::Down);

        reserve.withdraw(tokens_to_withdraw, notes_remaining);

        // TODO: Double check:
        token::transfer(
            ctx.accounts
                .transfer_context()
                .with_signer(&[&market.authority_seeds()]),
            tokens_to_withdraw,
        )?;

        // TODO: Double check:
        token::burn(
            ctx.accounts
                .note_burn_context()
                .with_signer(&[&market.authority_seeds()]),
            notes_remaining,
        )?;
    }


    // Account should now be empty, so we can close it out
    token::close_account(
        ctx.accounts
            .close_context()
            .with_signer(&[&market.authority_seeds()]),
    )?;    

    msg!("closed loan account");
    Ok(())
}
