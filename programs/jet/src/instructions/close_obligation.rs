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
use anchor_spl::token::{self, CloseAccount, Transfer};
            
use crate::state::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CloseObligation<'info> {
    /// The relevant market
    #[account(has_one = market_authority)]
    pub market: Loader<'info, Market>,

    /// The market's authority account
    pub market_authority: AccountInfo<'info>,
    
    /// The account that stores the obligation notes, such as loans and collaterals, to be closed.
    #[account(mut,
              seeds = [
                  b"obligation".as_ref(),
                  market.key().as_ref(),
                  borrower.key.as_ref()
              ],
              has_one = owner,
              bump = bump)]
    pub obligation: Loader<'info, Obligation>,

    /// The user/authority that is responsible for owning this obligation.
    #[account(mut, signer)]
    pub owner: AccountInfo<'info>,

    #[account(address = token::ID)]
    pub token_program: AccountInfo<'info>,

    /// Marks the account as being closed at the end of the instruction’s execution, 
    /// sending the rent exemption lamports to the specified .
    #[account(mut, close = "JPv1rCqrhagNNmJVM5J1he7msQ5ybtvE1nNuHpDHMNU")]
    pub close_obligation: Loader<'info, Obligation>,
}

impl<'info> CloseObligation<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            Transfer {
                from: self.obligation.to_account_info(),
                to: self.owner.to_account_info(),
                authority: self.market_authority.clone(),
            },
        )
    }
    
    fn close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            CloseAccount {
                account: self.obligation.to_account_info(),
                destination: self.close_obligation.to_account_info(),
                authority: self.market_authority.clone(),
            },
        )
    }
}

// TODO: handler for close obligation transfer ownership from owner to jet program

///Close an account that tracks a portfolio of collateral deposits and loans.
pub fn handler(ctx: Context<CloseObligation>, _bump: u8) -> ProgramResult {
    let obligation = ctx.accounts.obligation.load()?;


    //  TODO: verify if loans is empty & collateral is empty, then proceed with closing the obligation account
    let collateral_remaining = token::accessor::amount(&ctx.accounts.obligation.collateral)?;
    let loan_remaining = token::accessor::amount(&ctx.accounts.obligation.loan)?;

    /// TODO: comment for doc
    if collateral_remaining.len > 0 && loan_remaining.len > 0 {
        // TODO: verify if loans is empty & collateral is empty, then proceed 
        // (need to write this function if doesn't exist)
        verify_ability_obligation_withdraw()?;
        
        // TODO: double check: transfer CPI context
        token::transfer(
            ctx.accounts
            .transfer_context()
            .with_signer(&[&market.authority_seeds()])
        )?;
        
    }
    
    // TODO: double check: Account should now be empty, so we can close it out
    token::close_account(
        ctx.accounts
        .close_context()
        .with_signer(&[&market.authority_seeds()]),
    )?;
    
    msg!("closeed obligation account");
    Ok(())
}



