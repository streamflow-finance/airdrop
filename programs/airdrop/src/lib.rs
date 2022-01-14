use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, SetAuthority, Token, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

//Defines the program's ID. This should be used at the root of all Anchor based programs.
declare_id!("Ek6Jpdv5iEEDLXTVQ8UFcntms3DT2ewHtzzwH2R5MpvN");

#[program]
pub mod airdrop {
    use super::*;

    const PDA_SEED: &[u8] = b"streamflow-airdrop";

    pub fn initialize_airdrop(
        ctx: Context<InitializeAirdrop>,
        airdrop_amount: u64,
        withdraw_amount: u64,
    ) -> ProgramResult {
        msg!("Initializing airdrop...");

        ctx.accounts.airdrop_account.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts
            .airdrop_account
            .initializer_deposit_token_account = *ctx
            .accounts
            .initializer_deposit_token_account
            .to_account_info()
            .key;

        ctx.accounts.airdrop_account.airdrop_token_account =
            *ctx.accounts.airdrop_token_account.to_account_info().key;
            msg!("ok 1");
        ctx.accounts.airdrop_account.withdraw_amount = withdraw_amount;
        msg!("ok 2");
        let (pda, _bump_seed) = Pubkey::find_program_address(&[PDA_SEED], ctx.program_id);
        let seeds = &[&PDA_SEED[..], &[_bump_seed]];
        msg!("ok 3");
        token::transfer(
            ctx.accounts
                .transfer_amount_to_airdrop()
                .with_signer(&[&seeds[..]]),
            airdrop_amount,
        )?;
        msg!("ok 4");
        // Transfer initializer token account ownership to PDA
        token::set_authority(ctx.accounts.into(), AuthorityType::AccountOwner, Some(pda))?;
        Ok(())
    }

    pub fn get_airdrop(ctx: Context<GetAirdrop>) -> ProgramResult {
        let (_pda, bump_seed) = Pubkey::find_program_address(&[PDA_SEED], ctx.program_id);
        let seeds = &[&PDA_SEED[..], &[bump_seed]];

        msg!("Giving an airdrop...");

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

        msg!("Canceling airdrop! Refunding airdrop initializer...");

        token::transfer(
            ctx.accounts
                .refund_to_initilizer()
                .with_signer(&[&seeds[..]]),
            ctx.accounts.airdrop_token_account.amount,
        )?;

        ctx.accounts
            .into_set_close_airdrop_context()
            .with_signer(&[&seeds[..]]);

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

        #[account(init, payer = initializer, space = AirdropAccount::LEN)]
        pub airdrop_account: Account<'info, AirdropAccount>,

        #[account(mut)]
        pub airdrop_token_account: Account<'info, TokenAccount>,

        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }

    #[derive(Accounts)]
    pub struct GetAirdrop<'info> {
        #[account(signer)]
        pub taker: AccountInfo<'info>,
        #[account(init_if_needed, associated_token::mint = mint, associated_token::authority = taker, payer = taker)]
        pub taker_receive_token_account: Account<'info, TokenAccount>,
        #[account(mut)]
        pub airdrop_account: Account<'info, AirdropAccount>,
        pub mint: Account<'info, Mint>,
        #[account(mut)]
        pub airdrop_token_account: Account<'info, TokenAccount>,
        pub pda_account: AccountInfo<'info>,
        pub token_program: Program<'info, Token>,
        pub associated_token_program: Program<'info, AssociatedToken>,
        pub system_program: Program<'info, System>,
        pub rent: Sysvar<'info, Rent>,
    }

    #[derive(Accounts)]
    pub struct CancelAirdrop<'info> {
        #[account(mut, signer)]
        pub initializer: AccountInfo<'info>,
        #[account(
        mut,
        constraint = airdrop_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        )]
        pub initializer_deposit_token_account: Account<'info, TokenAccount>,
        pub pda_account: AccountInfo<'info>,
        #[account(
    mut,
    constraint = airdrop_account.initializer_key == *initializer.key,
    close = initializer
    )]
        pub airdrop_account: Account<'info, AirdropAccount>,

        #[account(mut)]
        pub airdrop_token_account: Account<'info, TokenAccount>,

        pub token_program: AccountInfo<'info>,
    }

    #[account]
    pub struct AirdropAccount {
        pub initializer_key: Pubkey,
        pub initializer_deposit_token_account: Pubkey,
        pub airdrop_token_account: Pubkey,
        pub withdraw_amount: u64,
    }

    impl AirdropAccount {
        pub const LEN: usize = 32 + 32 + 32 + 8;
    }

    impl<'info> From<&mut InitializeAirdrop<'info>>
        for CpiContext<'_, '_, '_, 'info, SetAuthority<'info>>
    {
        fn from(accounts: &mut InitializeAirdrop<'info>) -> Self {
            let cpi_accounts = SetAuthority {
                account_or_mint: accounts.airdrop_token_account.to_account_info().clone(),
                current_authority: accounts.airdrop_account.to_account_info().clone(),
            };
            let cpi_program = accounts.token_program.to_account_info();
            CpiContext::new(cpi_program, cpi_accounts)
        }
    }

    impl<'info> InitializeAirdrop<'info> {
        fn transfer_amount_to_airdrop(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
            let cpi_accounts = Transfer {
                from: self
                    .initializer_deposit_token_account
                    .to_account_info()
                    .clone(),
                to: self.airdrop_token_account.to_account_info().clone(),
                authority: self.initializer.clone(),
            };
            let cpi_program = self.token_program.to_account_info();
            CpiContext::new(cpi_program, cpi_accounts)
        }
    }

    impl<'info> GetAirdrop<'info> {
        fn into_transfer_to_taker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
            let cpi_accounts = Transfer {
                from: self.airdrop_token_account.to_account_info().clone(),
                to: self.taker_receive_token_account.to_account_info().clone(),
                authority: self.pda_account.to_account_info().clone(),
            };
            let cpi_program = self.token_program.to_account_info();
            CpiContext::new(cpi_program, cpi_accounts)
        }
    }

    impl<'info> CancelAirdrop<'info> {
        fn refund_to_initilizer(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
            let cpi_accounts = Transfer {
                from: self.airdrop_token_account.to_account_info().clone(),
                to: self
                    .initializer_deposit_token_account
                    .to_account_info()
                    .clone(),
                authority: self.pda_account.to_account_info().clone(),
            };
            let cpi_program = self.token_program.to_account_info();
            CpiContext::new(cpi_program, cpi_accounts)
        }
    }

    impl<'info> CancelAirdrop<'info> {
        fn into_set_close_airdrop_context(
            &self,
        ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
            let cpi_accounts = SetAuthority {
                account_or_mint: self.initializer.to_account_info().clone(),
                current_authority: self.pda_account.clone(),
            };
            CpiContext::new(self.token_program.clone(), cpi_accounts)
        }
    }
}
