use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction::RequestDeleteAccount;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};

use super::ShadowDriveClient;
use crate::{
    constants::{PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT},
    models::*,
};

impl<T> ShadowDriveClient<T>
where
    T: Signer + Send + Sync,
{
    /// Marks a [`StorageAccount`](crate::models::StorageAccount) for deletion from the Shadow Drive.
    /// If an account is marked for deletion, all files within the account will be deleted as well.
    /// Any stake remaining in the [`StorageAccount`](crate::models::StorageAccount) will be refunded to the creator.
    /// Accounts marked for deletion are deleted at the end of each Solana epoch.
    /// Marking a [`StorageAccount`](crate::models::StorageAccount) for deletion can be undone with `cancel_delete_storage_account`,
    /// but this must be done before the end of the Solana epoch.
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that you want to mark for deletion.
    /// # Example
    ///
    /// ```
    /// # use shadow_drive_rust::{ShadowDriveClient, derived_addresses::storage_account};
    /// # use solana_client::rpc_client::RpcClient;
    /// # use solana_sdk::{
    /// # pubkey::Pubkey,
    /// # signature::Keypair,
    /// # signer::{keypair::read_keypair_file, Signer},
    /// # };
    /// #
    /// # let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    /// # let user_pubkey = keypair.pubkey();
    /// # let rpc_client = RpcClient::new("https://ssc-dao.genesysgo.net");
    /// # let shdw_drive_client = ShadowDriveClient::new(keypair, rpc_client);
    /// # let (storage_account_key, _) = storage_account(&user_pubkey, 0);
    /// #
    /// let delete_storage_account_response = shdw_drive_client
    ///     .delete_storage_account(&storage_account_key)
    ///     .await?;
    /// ```
    pub async fn delete_storage_account(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let wallet = &self.wallet;
        let wallet_pubkey = wallet.pubkey();

        let selected_account = self.get_storage_account(storage_account_key).await?;

        let accounts = shdw_drive_accounts::RequestDeleteAccount {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            owner: selected_account.owner_1,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
        };

        let args = RequestDeleteAccount {};

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let txn = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&wallet_pubkey),
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash()?,
        );

        let txn_result = self.rpc_client.send_and_confirm_transaction(&txn)?;

        Ok(ShdwDriveResponse {
            txid: txn_result.to_string(),
        })
    }
}
