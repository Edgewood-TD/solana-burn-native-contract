use solana_program::program::{invoke_signed, invoke};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    msg,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{Sysvar, rent::Rent},
    self,
};
use solana_program::borsh::try_from_slice_unchecked;
use borsh::{BorshDeserialize, BorshSerialize,BorshSchema};
use spl_token;
use spl_associated_token_account;
use spl_token_metadata;


// Declare and export the program's entrypoint
entrypoint!(process_instruction);

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
enum StakeInstruction{
    GenerateVault,
    Stake,
    AddToWhitelist{
        #[allow(dead_code)]
        price:u64,
    },
}



#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct RateData{
    price: u64,
}

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let instruction: StakeInstruction = try_from_slice_unchecked(instruction_data).unwrap();
    let vault_word = "vault";
    let whitelist_word = "whitelist";
    let admin = "HRqXXua5SSsr1C7pBWhtLxjD9HcreNd4ZTKJD7em7mtP".parse::<Pubkey>().unwrap();
    let reward_mint = "89wT74E9QPfzpJxyW8csuvYj3hjevX1jJmcRqMnEgUSp".parse::<Pubkey>().unwrap();
    let burn_add="6t15n9xHdt1uZg6mYxCZdE6uhiBhDqKn7puXQMA4k8XV".parse::<Pubkey>().unwrap();

    match instruction{
       
        StakeInstruction::AddToWhitelist{price}=>{
            let payer = next_account_info(accounts_iter)?;
            let candy_machine_info = next_account_info(accounts_iter)?;
            let whitelist_info = next_account_info(accounts_iter)?;
            let sys_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;

            let rent = &Rent::from_account_info(rent_info)?;

            if *payer.key!=admin||!payer.is_signer{
                //unauthorized access
                return Err(ProgramError::Custom(0x31));
            }

            let (data_address,data_address_bump) = Pubkey::find_program_address(&[whitelist_word.as_bytes(), &candy_machine_info.key.to_bytes()], &program_id);
            if *whitelist_info.key!=data_address{
                //wrong whitelist_info
                return Err(ProgramError::Custom(0x32));
            }

            // if candy_machine_info.owner.to_string() != "cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ" {
            //     // msg!("invalid candy machine");
            //     return Err(ProgramError::Custom(0x33));
            // }

            let size = 8;
            if whitelist_info.owner!=program_id{
                let required_lamports = rent
                .minimum_balance(size as usize)
                .max(1)
                .saturating_sub(whitelist_info.lamports());
                invoke(
                    &system_instruction::transfer(payer.key, &data_address, required_lamports),
                    &[
                        payer.clone(),
                        whitelist_info.clone(),
                        sys_info.clone(),
                    ],
                )?;
                invoke_signed(
                    &system_instruction::allocate(&data_address, size),
                    &[
                        whitelist_info.clone(),
                        sys_info.clone(),
                    ],
                    &[&[whitelist_word.as_bytes(), &candy_machine_info.key.to_bytes(), &[data_address_bump]]],
                )?;

                invoke_signed(
                    &system_instruction::assign(&data_address, program_id),
                    &[
                        whitelist_info.clone(),
                        sys_info.clone(),
                    ],
                    &[&[whitelist_word.as_bytes(), &candy_machine_info.key.to_bytes(), &[data_address_bump]]],
                )?;
            }

            let rate_struct = RateData{
                price,
            };
            rate_struct.serialize(&mut &mut whitelist_info.data.borrow_mut()[..])?;
        },


        
        StakeInstruction::Stake=>{
            let payer = next_account_info(accounts_iter)?;
            let mint = next_account_info(accounts_iter)?;
            let metadata_account_info = next_account_info(accounts_iter)?;
            
            let vault_info = next_account_info(accounts_iter)?;
            let source = next_account_info(accounts_iter)?;
            let destination = next_account_info(accounts_iter)?;

            let token_program = next_account_info(accounts_iter)?;
            let sys_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let token_assoc = next_account_info(accounts_iter)?;
            
            let stake_data_info = next_account_info(accounts_iter)?;
            let whitelist_info = next_account_info(accounts_iter)?;
            let burn_account_add = next_account_info(accounts_iter)?;
            let payer_reward_holder_info = next_account_info(accounts_iter)?;
            let vault_reward_holder_info = next_account_info(accounts_iter)?;
            let reward_mint_info = next_account_info(accounts_iter)?;

            // let clock = Clock::get()?;
            if *burn_account_add.key!=burn_add{
                //msg!("Wrong wallet address for burning");
                return Err(ProgramError::Custom(0x536));
            }
            if *token_program.key!=spl_token::id(){
                //wrong token_info
                return Err(ProgramError::Custom(0x345));
            }

            // let rent = &Rent::from_account_info(rent_info)?;
            let ( stake_data, _stake_data_bump ) = Pubkey::find_program_address(&[&mint.key.to_bytes()], &program_id);
            let payer_reward_holder = spl_associated_token_account::get_associated_token_address(payer.key, &reward_mint);
            let vault_reward_holder = spl_associated_token_account::get_associated_token_address(vault_info.key, &reward_mint);

            if !payer.is_signer{
                //unauthorized access
                return Err(ProgramError::Custom(0x11));
            }

            if stake_data!=*stake_data_info.key{
                //msg!("invalid stake_data account!");
                return Err(ProgramError::Custom(0x10));
            }

            if &Pubkey::find_program_address(&["metadata".as_bytes(), &spl_token_metadata::ID.to_bytes(), &mint.key.to_bytes()], &spl_token_metadata::ID).0 != metadata_account_info.key {
                msg!("invalid metadata account!");
                return Err(ProgramError::Custom(0x03));
            }

            let metadata = spl_token_metadata::state::Metadata::from_account_info(metadata_account_info)?;
            let creators = metadata.data.creators.unwrap();
            let cndy = creators.first().unwrap();
            let candy_machine = cndy.address;


            // if candy_machine != *candy_machine_info.key {
            //     //msg!("Wrong candy machine");
            //     return Err(ProgramError::Custom(0x04));
            // }

            let (wl_data_address,_wl_data_address_bump) = Pubkey::find_program_address(&[whitelist_word.as_bytes(), &candy_machine.to_bytes()], &program_id);

            if payer_reward_holder!=*payer_reward_holder_info.key{
                //wrong payer_reward_holder_info
                return Err(ProgramError::Custom(0x62));
            }

            if vault_reward_holder!=*vault_reward_holder_info.key{
                //wrong vault_reward_holder_info
                return Err(ProgramError::Custom(0x63));
            }
            if *whitelist_info.key != wl_data_address{
                // wrong whitelist_info
                return Err(ProgramError::Custom(0x900));
            }

            if whitelist_info.owner != program_id{
                // candy machine is not whitelisted
                return Err(ProgramError::Custom(0x902));
            }

            let wl_rate_data = if let Ok(data) = RateData::try_from_slice(&whitelist_info.data.borrow()){
                data.price
            } else {
                // can't deserialize rate data
                return Err(ProgramError::Custom(0x901));
            };

            // if candy_machine_info.owner.to_string() != "cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ" {
            //     // msg!("invalid candy machine");
            //     return Err(ProgramError::Custom(0x05));
            // }

            if !cndy.verified{
                //msg!("address is not verified");
                return Err(ProgramError::Custom(0x06));
            }

            let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&vault_word.as_bytes()], &program_id);
            if vault != *vault_info.key{
                //msg!("Wrong vault");
                return Err(ProgramError::Custom(0x07));
            }

            if &spl_associated_token_account::get_associated_token_address(payer.key, mint.key) != source.key {
                // msg!("Wrong source");
                return Err(ProgramError::Custom(0x08));
            }

            if &spl_associated_token_account::get_associated_token_address(&burn_add, mint.key) != destination.key{
                //msg!("Wrong destination");
                return Err(ProgramError::Custom(0x09));
            }

            let reward=wl_rate_data*1000000000;
            if payer_reward_holder_info.owner != token_program.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,
                        payer.key,
                        reward_mint_info.key,
                    ),
                    &[
                        payer.clone(), 
                        payer_reward_holder_info.clone(), 
                        payer.clone(),
                        reward_mint_info.clone(),
                        sys_info.clone(),
                        token_program.clone(),
                        rent_info.clone(),
                        token_assoc.clone(),
                    ],
                    
                )?;
            }

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program.key,
                    vault_reward_holder_info.key,
                    payer_reward_holder_info.key,
                    vault_info.key,
                    &[],
                    reward,
                )?,
                &[
                    vault_reward_holder_info.clone(),
                    payer_reward_holder_info.clone(),
                    vault_info.clone(), 
                    token_program.clone()
                ],
                &[&[&vault_word.as_bytes(), &[_vault_bump]]],
            )?;
            if destination.owner != token_program.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,
                        burn_account_add.key,
                        mint.key,
                    ),
                    &[
                        payer.clone(), 
                        destination.clone(), 
                        burn_account_add.clone(),
                        mint.clone(),
                        sys_info.clone(),
                        token_program.clone(),
                        rent_info.clone(),
                        token_assoc.clone(),
                    ],
                )?;
            }
            invoke(
                &spl_token::instruction::transfer(
                    token_program.key,
                    source.key,
                    destination.key,
                    payer.key,
                    &[],
                    1,
                )?,
                &[
                    source.clone(),
                    destination.clone(),
                    payer.clone(), 
                    token_program.clone()
                ],
            )?;

        },

        StakeInstruction::GenerateVault=>{
            let payer = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let pda = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;

            let rent = &Rent::from_account_info(rent_info)?;

            let (vault_pda, vault_bump_seed) =
                Pubkey::find_program_address(&[vault_word.as_bytes()], &program_id);
            
            if pda.key!=&vault_pda{
                //msg!("Wrong account generated by client");
                return Err(ProgramError::Custom(0x00));
            }

            if pda.owner!=program_id{
                let size = 16;
           
                let required_lamports = rent
                .minimum_balance(size as usize)
                .max(1)
                .saturating_sub(pda.lamports());

                invoke(
                    &system_instruction::transfer(payer.key, &vault_pda, required_lamports),
                    &[
                        payer.clone(),
                        pda.clone(),
                        system_program.clone(),
                    ],
                )?;

                invoke_signed(
                    &system_instruction::allocate(&vault_pda, size),
                    &[
                        pda.clone(),
                        system_program.clone(),
                    ],
                    &[&[vault_word.as_bytes(), &[vault_bump_seed]]],
                )?;

                invoke_signed(
                    &system_instruction::assign(&vault_pda, program_id),
                    &[
                        pda.clone(),
                        system_program.clone(),
                    ],
                    &[&[vault_word.as_bytes(), &[vault_bump_seed]]],
                )?;
            }

            if *payer.key!=admin||!payer.is_signer{
                //unauthorized access
                return Err(ProgramError::Custom(0x02));
            }

            
        }
    };
        
    Ok(())
}


