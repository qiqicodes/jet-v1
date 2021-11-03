pub mod close_deposit_account {
    use anchor_lang::prelude::*;
    use anchor_lang::Key;
    use anchor_spl::token::{self, Burn, CloseAccount, Transfer};
    use crate::state::*;
    use crate::Rounding;
    # [instruction (bump : u8)]
    pub struct CloseDepositAccount<'info> {
        /// The relevant market this deposit is for
        # [account (has_one = market_authority)]
        pub market: Loader<'info, Market>,
        /// The market's authority account
        pub market_authority: AccountInfo<'info>,
        /// The reserve deposited into
        # [account (mut , has_one = market , has_one = vault , has_one = deposit_note_mint)]
        pub reserve: Loader<'info, Reserve>,
        /// The reserve's vault where any tokens to withdraw will be transferred from
        #[account(mut)]
        pub vault: AccountInfo<'info>,
        /// The mint for the deposit notes
        #[account(mut)]
        pub deposit_note_mint: AccountInfo<'info>,
        /// The user/authority that owns the deposits
        #[account(mut, signer)]
        pub depositor: AccountInfo<'info>,
        /// The account that stores the deposit notes, to be closed
        # [account (mut , seeds = [b"deposits" . as_ref () , reserve . key () . as_ref () , depositor . key . as_ref ()] , bump = bump)]
        pub deposit_account: AccountInfo<'info>,
        /// The account to receive any remaining tokens still deposited
        #[account(mut)]
        pub receiver_account: AccountInfo<'info>,
        # [account (address = anchor_spl :: token :: ID)]
        pub token_program: AccountInfo<'info>,
    }
    #[automatically_derived]
    impl<'info> anchor_lang::Accounts<'info> for CloseDepositAccount<'info>
    where
        'info: 'info,
    {
        #[inline(never)]
        fn try_accounts(
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            accounts: &mut &[anchor_lang::solana_program::account_info::AccountInfo<'info>],
            ix_data: &[u8],
        ) -> std::result::Result<Self, anchor_lang::solana_program::program_error::ProgramError>
        {
            let mut ix_data = ix_data;
            struct __Args {
                bump: u8,
            }
            impl borsh::ser::BorshSerialize for __Args
            where
                u8: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.bump, writer)?;
                    Ok(())
                }
            }
            impl borsh::de::BorshDeserialize for __Args
            where
                u8: borsh::BorshDeserialize,
            {
                fn deserialize(
                    buf: &mut &[u8],
                ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                    Ok(Self {
                        bump: borsh::BorshDeserialize::deserialize(buf)?,
                    })
                }
            }
            let __Args { bump } = __Args::deserialize(&mut ix_data)
                .map_err(|_| anchor_lang::__private::ErrorCode::InstructionDidNotDeserialize)?;
            let market: anchor_lang::Loader<Market> =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let market_authority: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let reserve: anchor_lang::Loader<Reserve> =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let vault: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let deposit_note_mint: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let depositor: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let deposit_account: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let receiver_account: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let token_program: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            if &market.load()?.market_authority != market_authority.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if !reserve.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if &reserve.load()?.market != market.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if &reserve.load()?.vault != vault.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if &reserve.load()?.deposit_note_mint != deposit_note_mint.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if !vault.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if !deposit_note_mint.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if !depositor.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if true {
                if !depositor.to_account_info().is_signer {
                    return Err(anchor_lang::__private::ErrorCode::ConstraintSigner.into());
                }
            }
            let __program_signer = Pubkey::create_program_address(
                &[
                    b"deposits".as_ref(),
                    reserve.key().as_ref(),
                    depositor.key.as_ref(),
                    &[bump],
                ][..],
                program_id,
            )
            .map_err(|_| anchor_lang::__private::ErrorCode::ConstraintSeeds)?;
            if deposit_account.to_account_info().key != &__program_signer {
                return Err(anchor_lang::__private::ErrorCode::ConstraintSeeds.into());
            }
            if !deposit_account.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if !receiver_account.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if token_program.to_account_info().key != &anchor_spl::token::ID {
                return Err(anchor_lang::__private::ErrorCode::ConstraintAddress.into());
            }
            Ok(CloseDepositAccount {
                market,
                market_authority,
                reserve,
                vault,
                deposit_note_mint,
                depositor,
                deposit_account,
                receiver_account,
                token_program,
            })
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountInfos<'info> for CloseDepositAccount<'info>
    where
        'info: 'info,
    {
        fn to_account_infos(
            &self,
        ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
            let mut account_infos = ::alloc::vec::Vec::new();
            account_infos.extend(self.market.to_account_infos());
            account_infos.extend(self.market_authority.to_account_infos());
            account_infos.extend(self.reserve.to_account_infos());
            account_infos.extend(self.vault.to_account_infos());
            account_infos.extend(self.deposit_note_mint.to_account_infos());
            account_infos.extend(self.depositor.to_account_infos());
            account_infos.extend(self.deposit_account.to_account_infos());
            account_infos.extend(self.receiver_account.to_account_infos());
            account_infos.extend(self.token_program.to_account_infos());
            account_infos
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountMetas for CloseDepositAccount<'info> {
        fn to_account_metas(
            &self,
            is_signer: Option<bool>,
        ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
            let mut account_metas = ::alloc::vec::Vec::new();
            account_metas.extend(self.market.to_account_metas(None));
            account_metas.extend(self.market_authority.to_account_metas(None));
            account_metas.extend(self.reserve.to_account_metas(None));
            account_metas.extend(self.vault.to_account_metas(None));
            account_metas.extend(self.deposit_note_mint.to_account_metas(None));
            account_metas.extend(self.depositor.to_account_metas(Some(true)));
            account_metas.extend(self.deposit_account.to_account_metas(None));
            account_metas.extend(self.receiver_account.to_account_metas(None));
            account_metas.extend(self.token_program.to_account_metas(None));
            account_metas
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::AccountsExit<'info> for CloseDepositAccount<'info>
    where
        'info: 'info,
    {
        fn exit(
            &self,
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
        ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
            anchor_lang::AccountsExit::exit(&self.reserve, program_id)?;
            anchor_lang::AccountsExit::exit(&self.vault, program_id)?;
            anchor_lang::AccountsExit::exit(&self.deposit_note_mint, program_id)?;
            anchor_lang::AccountsExit::exit(&self.depositor, program_id)?;
            anchor_lang::AccountsExit::exit(&self.deposit_account, program_id)?;
            anchor_lang::AccountsExit::exit(&self.receiver_account, program_id)?;
            Ok(())
        }
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
    /// instead of an `AccountInfo`. This is useful for clients that want
    /// to generate a list of accounts, without explicitly knowing the
    /// order all the fields should be in.
    ///
    /// To access the struct in this module, one should use the sibling
    /// `accounts` module (also generated), which re-exports this.
    pub(crate) mod __client_accounts_close_deposit_account {
        use super::*;
        use anchor_lang::prelude::borsh;
        pub struct CloseDepositAccount {
            pub market: anchor_lang::solana_program::pubkey::Pubkey,
            pub market_authority: anchor_lang::solana_program::pubkey::Pubkey,
            pub reserve: anchor_lang::solana_program::pubkey::Pubkey,
            pub vault: anchor_lang::solana_program::pubkey::Pubkey,
            pub deposit_note_mint: anchor_lang::solana_program::pubkey::Pubkey,
            pub depositor: anchor_lang::solana_program::pubkey::Pubkey,
            pub deposit_account: anchor_lang::solana_program::pubkey::Pubkey,
            pub receiver_account: anchor_lang::solana_program::pubkey::Pubkey,
            pub token_program: anchor_lang::solana_program::pubkey::Pubkey,
        }
        impl borsh::ser::BorshSerialize for CloseDepositAccount
        where
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.market, writer)?;
                borsh::BorshSerialize::serialize(&self.market_authority, writer)?;
                borsh::BorshSerialize::serialize(&self.reserve, writer)?;
                borsh::BorshSerialize::serialize(&self.vault, writer)?;
                borsh::BorshSerialize::serialize(&self.deposit_note_mint, writer)?;
                borsh::BorshSerialize::serialize(&self.depositor, writer)?;
                borsh::BorshSerialize::serialize(&self.deposit_account, writer)?;
                borsh::BorshSerialize::serialize(&self.receiver_account, writer)?;
                borsh::BorshSerialize::serialize(&self.token_program, writer)?;
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::ToAccountMetas for CloseDepositAccount {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.market,
                        false,
                    ),
                );
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.market_authority,
                        false,
                    ),
                );
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.reserve,
                    false,
                ));
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.vault, false,
                ));
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.deposit_note_mint,
                    false,
                ));
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.depositor,
                    true,
                ));
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.deposit_account,
                    false,
                ));
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.receiver_account,
                    false,
                ));
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.token_program,
                        false,
                    ),
                );
                account_metas
            }
        }
    }
    impl<'info> CloseDepositAccount<'info> {
        fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
            CpiContext::new(
                self.token_program.clone(),
                Transfer {
                    from: self.vault.to_account_info(),
                    to: self.receiver_account.to_account_info(),
                    authority: self.market_authority.clone(),
                },
            )
        }
        fn note_burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
            CpiContext::new(
                self.token_program.clone(),
                Burn {
                    to: self.deposit_account.to_account_info(),
                    mint: self.deposit_note_mint.to_account_info(),
                    authority: self.market_authority.clone(),
                },
            )
        }
        fn close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
            CpiContext::new(
                self.token_program.clone(),
                CloseAccount {
                    account: self.deposit_account.to_account_info(),
                    destination: self.depositor.to_account_info(),
                    authority: self.market_authority.clone(),
                },
            )
        }
    }
    /// Close an account that stores deposit notes
    pub fn handler(ctx: Context<CloseDepositAccount>, _bump: u8) -> ProgramResult {
        let market = ctx.accounts.market.load()?;
        let notes_remaining = token::accessor::amount(&ctx.accounts.deposit_account)?;
        if notes_remaining > 0 {
            market.verify_ability_deposit_withdraw()?;
            let mut reserve = ctx.accounts.reserve.load_mut()?;
            let clock = Clock::get()?;
            let reserve_info = market.reserves().get_cached(reserve.index, clock.slot);
            let tokens_to_withdraw =
                reserve_info.deposit_notes_to_tokens(notes_remaining, Rounding::Down);
            reserve.withdraw(tokens_to_withdraw, notes_remaining);
            token::transfer(
                ctx.accounts
                    .transfer_context()
                    .with_signer(&[&market.authority_seeds()]),
                tokens_to_withdraw,
            )?;
            token::burn(
                ctx.accounts
                    .note_burn_context()
                    .with_signer(&[&market.authority_seeds()]),
                notes_remaining,
            )?;
        }
        token::close_account(
            ctx.accounts
                .close_context()
                .with_signer(&[&market.authority_seeds()]),
        )?;
        ::solana_program::log::sol_log("closed deposit account");
        Ok(())
    }
}
