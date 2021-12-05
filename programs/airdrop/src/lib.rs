use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

//Defines the program's ID. This should be used at the root of all Anchor based programs.
declare_id!("Mg6PaZpoDXkFvimZpWWK9WBeZ8FCfcGkgG76zSSsMn1");

#[program]
pub mod airdrop {
    use super::*;

    const PDA_SEED: &[u8] = b"streamflow-airdrop";

    // Initializer creates airdrop using his token account
    // Until initializer cancels the airdrop, PDA is owner of initializer token account
    pub fn initialize_airdrop(
        ctx: Context<InitializeAirdrop>,
        airdrop_amount: u64,
        withdraw_amount: u64,
    ) -> ProgramResult {
        ctx.accounts.airdrop_account.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts
            .airdrop_account
            .initializer_deposit_token_account = *ctx
            .accounts
            .initializer_deposit_token_account
            .to_account_info()
            .key;

        ctx.accounts.airdrop_account.airdrop_amount = airdrop_amount;
        ctx.accounts.airdrop_account.withdraw_amount = withdraw_amount;

        let (pda, _bump_seed) = Pubkey::find_program_address(&[PDA_SEED], ctx.program_id);

        // Transfer initializer token account ownership to PDA
        token::set_authority(ctx.accounts.into(), AuthorityType::AccountOwner, Some(pda))?;
        Ok(())
    }

    pub fn get_airdrop(ctx: Context<GetAirdrop>) -> ProgramResult {
        let (_pda, bump_seed) = Pubkey::find_program_address(&[PDA_SEED], ctx.program_id);
        let seeds = &[&PDA_SEED[..], &[bump_seed]];

        msg!("Giving an airdrop...");

        //todo implement a blacklist so taker can't call program multiple times?

        token::transfer(
            ctx.accounts
                .into_transfer_to_taker_context()
                .with_signer(&[&seeds[..]]),
            ctx.accounts.airdrop_account.withdraw_amount,
        )?;

        msg!("Taker got airdrop successfully!");

        Ok(())
    }

    pub fn cancel_airdrop(ctx: Context<CancelAirdrop>) -> ProgramResult {
        let (_pda, bump_seed) = Pubkey::find_program_address(&[PDA_SEED], ctx.program_id);
        let seeds = &[&PDA_SEED[..], &[bump_seed]];

        msg!("Canceling airdrop! Giving ownership back to initializer...");

        //todo check if anything else is needed to cancel airdrop for good
        //todo check if airdrop could be paused and then continued...

        token::set_authority(
            ctx.accounts
                .into_set_authority_context()
                .with_signer(&[&seeds[..]]),
            AuthorityType::AccountOwner,
            Some(ctx.accounts.airdrop_account.initializer_key),
        )?;

        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(airdrop_amount: u64)]
    pub struct InitializeAirdrop<'info> {
        #[account(signer)]
        pub initializer: AccountInfo<'info>,
        #[account(
        mut,
        constraint = initializer_deposit_token_account.amount >= airdrop_amount
    )]
        pub initializer_deposit_token_account: Account<'info, TokenAccount>,
        #[account(init, payer = initializer, space = 8 + AirdropAccount::LEN)]
        pub airdrop_account: Account<'info, AirdropAccount>,
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }

    #[derive(Accounts)]
    pub struct GetAirdrop<'info> {
        #[account(signer)]
        pub taker: AccountInfo<'info>,
        #[account(mut)]
        pub taker_receive_token_account: Account<'info, TokenAccount>,
        #[account(mut)]
        pub pda_deposit_token_account: Account<'info, TokenAccount>,
        #[account(mut)]
        pub initializer_main_account: AccountInfo<'info>,
        #[account(
        mut,
        constraint = airdrop_account.initializer_deposit_token_account == *pda_deposit_token_account.to_account_info().key,
        constraint = airdrop_account.initializer_key == *initializer_main_account.key,
        close = initializer_main_account
    )]
        pub airdrop_account: Account<'info, AirdropAccount>,
        pub pda_account: AccountInfo<'info>,
        pub token_program: Program<'info, Token>,
    }

    #[derive(Accounts)]
    pub struct CancelAirdrop<'info> {
        pub initializer: AccountInfo<'info>,
        #[account(mut)]
        pub pda_deposit_token_account: Account<'info, TokenAccount>,
        pub pda_account: AccountInfo<'info>,
        #[account(
    mut,
    constraint = airdrop_account.initializer_key == *initializer.key,
    constraint = airdrop_account.initializer_deposit_token_account == *pda_deposit_token_account.to_account_info().key,
    close = initializer
    )]
        pub airdrop_account: ProgramAccount<'info, AirdropAccount>,
        pub token_program: AccountInfo<'info>,
    }

    #[account]
    pub struct AirdropAccount {
        pub initializer_key: Pubkey,
        pub initializer_deposit_token_account: Pubkey,
        pub airdrop_amount: u64,
        pub withdraw_amount: u64,
    }

    impl AirdropAccount {
        pub const LEN: usize = 32 + 32 + 8 + 8;
    }

    impl<'info> From<&mut InitializeAirdrop<'info>>
        for CpiContext<'_, '_, '_, 'info, SetAuthority<'info>>
    {
        fn from(accounts: &mut InitializeAirdrop<'info>) -> Self {
            let cpi_accounts = SetAuthority {
                account_or_mint: accounts
                    .initializer_deposit_token_account
                    .to_account_info()
                    .clone(),
                current_authority: accounts.initializer.clone(),
            };
            let cpi_program = accounts.token_program.to_account_info();
            CpiContext::new(cpi_program, cpi_accounts)
        }
    }

    impl<'info> GetAirdrop<'info> {
        fn into_transfer_to_taker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
            let cpi_accounts = Transfer {
                from: self.pda_deposit_token_account.to_account_info().clone(),
                to: self.taker_receive_token_account.to_account_info().clone(),
                authority: self.pda_account.clone(),
            };
            let cpi_program = self.token_program.to_account_info();
            CpiContext::new(cpi_program, cpi_accounts)
        }
    }

    impl<'info> CancelAirdrop<'info> {
        fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
            let cpi_accounts = SetAuthority {
                account_or_mint: self.pda_deposit_token_account.to_account_info().clone(),
                current_authority: self.pda_account.clone(),
            };
            let cpi_program = self.token_program.to_account_info();
            CpiContext::new(cpi_program, cpi_accounts)
        }
    }
}
