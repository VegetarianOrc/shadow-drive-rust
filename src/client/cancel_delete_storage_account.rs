use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction::UnmarkDeleteAccount;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};

use super::ShadowDriveClient;
use crate::{
    constants::{PROGRAM_ADDRESS, STORAGE_CONFIG_PDA, TOKEN_MINT},
    derived_addresses::stake_account,
    models::*,
};

impl<T> ShadowDriveClient<T>
where
    T: Signer + Send + Sync,
{
    /// Unmarks a [`StorageAccount`](crate::models::StorageAccount) for deletion from the Shadow Drive.
    /// To prevent deletion, this method must be called before the end of the Solana epoch in which `delete_storage_account` is called.
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that you want to unmark for deletion.
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
    /// let cancel_delete_storage_account_response = shdw_drive_client
    ///     .cancel_delete_storage_account(&storage_account_key)
    ///     .await?;
    /// ```
    pub async fn cancel_delete_storage_account(
        &self,
        storage_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let wallet = &self.wallet;
        let wallet_pubkey = wallet.pubkey();

        let selected_account = self.get_storage_account(storage_account_key).await?;
        let stake_account = stake_account(&storage_account_key).0;

        let accounts = shdw_drive_accounts::UnmarkDeleteAccount {
            storage_config: *STORAGE_CONFIG_PDA,
            storage_account: *storage_account_key,
            stake_account,
            owner: selected_account.owner_1,
            token_mint: TOKEN_MINT,
            system_program: system_program::ID,
        };

        let args = UnmarkDeleteAccount {};

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
