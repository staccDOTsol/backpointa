use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TransferChecked}};
use anchor_lang::solana_program::program_pack::Pack;
use spl_token::solana_program::{program::invoke, system_instruction};
use std::str::FromStr;

declare_id!("7XvN8FDBHMusbJR5tpwQBhDxperdbKmRA9YTM2yPNTJW");

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Instruction")]
    InvalidInstruction,
    #[msg("Insufficient funds for the transaction.")]
    InsufficientFundsForTransaction,
}


#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum TokenWrapInstruction {
    /// Creates a wrapped token mint
    /// Accounts and data expected by this instruction are outlined in the comment.
    CreateMint {
        idempotent: bool,
    },

    /// Wraps tokens
    /// Accounts and data expected by this instruction are outlined in the comment.
    Wrap {
        amount: u64,
    },

    /// Unwraps tokens
    /// Accounts and data expected by this instruction are outlined in the comment.
    Unwrap {
        amount: u64,
    },
}

// Context for `CreateMint` instruction
#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(init, payer = funder, 
        mint::decimals = unwrapped_mint.decimals,
        mint::authority = wrapped_mint_backpointer,
        mint::freeze_authority = wrapped_mint_backpointer,
    )]
    pub wrapped_mint: Box<InterfaceAccount<'info, Mint>>,
    pub unwrapped_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: 
    #[account(init, payer = funder, space = 8 + 32,
        seeds = [b"backpointa", unwrapped_mint.key().as_ref(), token_program_wrapped.key().as_ref()],
        bump
    )]
    pub wrapped_mint_backpointer: Account<'info, Backpointer>,
    /// CHECK: 
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: 
    pub token_program_wrapped: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    #[account(mut, constraint = to_service_account.key == &Pubkey::from_str("7ihN8QaTfNoDTRTQGULCzbUT3PHwPDTu5Brcu4iT2paP").unwrap())]
    /// CHECK: 
    pub to_service_account: AccountInfo<'info>,
}

// Context for `Wrap` instruction
#[derive(Accounts)]
pub struct Wrap<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(mut)]
    pub unwrapped_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(init_if_needed, payer = funder, associated_token::authority = wrapped_mint_backpointer, associated_token::mint = unwrapped_mint)]
    pub escrow: Box<InterfaceAccount<'info, TokenAccount>>,
    pub unwrapped_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub wrapped_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: 
    #[account(
        seeds = [b"backpointa", unwrapped_mint.key().as_ref(), token_program_wrapped.key().as_ref()],
        bump
    )]
    pub wrapped_mint_backpointer: Account<'info, Backpointer>,
    #[account(init_if_needed,
        payer = funder,
        associated_token::authority = funder,
        associated_token::mint = wrapped_mint
    )]
    pub recipient_wrapped_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: 
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: 
    pub token_program_wrapped: UncheckedAccount<'info>,
    #[account(mut, constraint = to_service_account.key == &Pubkey::from_str("7ihN8QaTfNoDTRTQGULCzbUT3PHwPDTu5Brcu4iT2paP").unwrap())]
    /// CHECK: 
    pub to_service_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub associated_token_program : Program<'info, AssociatedToken>,
}

// Context for `Unwrap` instruction
#[derive(Accounts)]
pub struct Unwrap<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(mut)]
    pub wrapped_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub wrapped_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: 
    #[account(
        seeds = [b"backpointa", unwrapped_mint.key().as_ref(), token_program_wrapped.key().as_ref()],
        bump
    )]
    pub wrapped_mint_backpointer: Account<'info, Backpointer>,
    #[account(mut)]
    pub escrow: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut
    )]
    pub recipient_unwrapped_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub unwrapped_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: 
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: 
    pub token_program_wrapped: UncheckedAccount<'info>,
    #[account(mut, constraint = to_service_account.key == &Pubkey::from_str("7ihN8QaTfNoDTRTQGULCzbUT3PHwPDTu5Brcu4iT2paP").unwrap())]
    /// CHECK: 
    pub to_service_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub associated_token_program : Program<'info, AssociatedToken>,
}
#[account]
pub struct Backpointer {
    pub wrapped_mint: Pubkey,
}


fn transfer_service_fee_lamports<'a>(
    funder_account: &AccountInfo<'a>,
    to_service_account: &AccountInfo<'a>,
    service_fee_lamports: u64,
) -> Result<()> {
    let transfer_instruction = system_instruction::transfer(
        funder_account.key, 
        to_service_account.key, 
        service_fee_lamports,
    );
    invoke(
        &transfer_instruction,
        &[
            funder_account.clone(),
            to_service_account.clone(),
        ],
    )?;
    Ok(())
}

#[program]
pub mod backpointa {
    use super::*;

    pub fn wrap(ctx: Context<Wrap>, amount: u64) -> Result<()> {
        // First, deserialize the unwrapped_mint to access its decimals
        // Extract a service fee of 0.001 SOL (1 SOL = 1 billion lamports) for performing this instruction
        let service_fee_lamports = 1_000_000; // 0.001 SOL in lamports

        transfer_service_fee_lamports(
            &ctx.accounts.funder,
            &ctx.accounts.to_service_account,
            service_fee_lamports,
        )?;
        let ai = ctx.accounts.unwrapped_mint.to_account_info();
        let unwrapped_mint_data = &ai.try_borrow_data()?;
        let unwrapped_mint = spl_token_2022::state::Mint::unpack(&unwrapped_mint_data)?;

        // Now, you can access the `decimals` field of the mint
        let decimals = unwrapped_mint.decimals;
        let seeds = &[b"backpointa", ctx.accounts.unwrapped_mint.to_account_info().key.as_ref(), ctx.accounts.token_program.to_account_info().key.as_ref(), &[ctx.bumps.wrapped_mint_backpointer]];
        let signer = &[&seeds[..]];

        // Proceed with the wrapping logic, ensuring the amounts are scaled correctly according to the decimals
        // Example: Transfer tokens from the user's account to the escrow
        let transfer_to_escrow_cpi_accounts = TransferChecked {
            from: ctx.accounts.unwrapped_token_account.to_account_info(),
            to: ctx.accounts.escrow.to_account_info(),
            authority: ctx.accounts.funder.to_account_info(),
            mint: ctx.accounts.unwrapped_mint.to_account_info(),
        };
        let transfer_to_escrow_cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program_wrapped.to_account_info(),
            transfer_to_escrow_cpi_accounts,
            signer,
        );
        token_interface::transfer_checked(transfer_to_escrow_cpi_context, amount, decimals)?;

        // Mint wrapped tokens to the recipient's account
        // Ensure to consider the `decimals` for the amount being minted if necessary

        let mint_to_recipient_cpi_accounts = token_interface::MintTo {
            mint: ctx.accounts.wrapped_mint.to_account_info(),
            to: ctx.accounts.recipient_wrapped_token_account.to_account_info(),
            authority: ctx.accounts.wrapped_mint_backpointer.to_account_info(),
        };
        let mint_to_recipient_cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program_wrapped.to_account_info(),
            mint_to_recipient_cpi_accounts,
            signer,
        );
        token_interface::mint_to(mint_to_recipient_cpi_context, amount)?;


        Ok(())
    }
    pub fn create_mint(ctx: Context<CreateMint>, idempotent: bool) -> Result<()> {
        let backpointer = &mut ctx.accounts.wrapped_mint_backpointer;
        backpointer.wrapped_mint = *ctx.accounts.wrapped_mint.to_account_info().key;
        // Additional logic for idempotency and initializing backpointer can be added here

        Ok(())
    }

    pub fn unwrap(ctx: Context<Unwrap>, amount: u64) -> Result<()> {
        // Business logic for unwrapping tokens

        let seeds = &[b"backpointa", 
            ctx.accounts.unwrapped_mint.to_account_info().key.as_ref(), 
            ctx.accounts.token_program_wrapped.to_account_info().key.as_ref(), 
            &[ctx.bumps.wrapped_mint_backpointer]
        ];
        let signer = &[&seeds[..]];
            // Example: Burn wrapped tokens from the user's account
            let burn_cpi_accounts = token_interface::Burn {
                from: ctx.accounts.wrapped_token_account.to_account_info(),
                mint: ctx.accounts.wrapped_mint.to_account_info(),
                authority: ctx.accounts.funder.to_account_info(),
            };
            let burn_cpi_context = CpiContext::new_with_signer(
                ctx.accounts.token_program_wrapped.to_account_info(),
                burn_cpi_accounts,
                signer,
            );
            token_interface::burn(burn_cpi_context, amount)?;
            
        

        // Transfer equivalent amount of unwrapped tokens from the escrow to the recipient
        // Similar logic using `token::transfer` can be implemented here
        
        let ai = ctx.accounts.unwrapped_mint.to_account_info();
        let unwrapped_mint_data = &ai.try_borrow_data()?;
        let unwrapped_mint = spl_token_2022::state::Mint::unpack(&unwrapped_mint_data)?;

        // Now, you can access the `decimals` field of the mint
        let decimals = unwrapped_mint.decimals;
        let transfer_from_escrow_cpi_accounts = TransferChecked {
            from: ctx.accounts.escrow.to_account_info(),
            to: ctx.accounts.recipient_unwrapped_token_account.to_account_info(),
            authority: ctx.accounts.wrapped_mint_backpointer.to_account_info(),
            mint: ctx.accounts.unwrapped_mint.to_account_info(),
        };
        let transfer_from_escrow_cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program_wrapped.to_account_info(),
            transfer_from_escrow_cpi_accounts,
            signer,
        );
        token_interface::transfer_checked(transfer_from_escrow_cpi_context, amount, decimals)?;


        Ok(())
    }

}