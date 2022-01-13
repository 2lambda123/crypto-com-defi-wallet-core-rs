use std::sync::Arc;

use defi_wallet_core_common::{
    broadcast_tx_sync, build_signed_single_msg_tx, get_account_balance, get_account_details,
    get_single_msg_sign_payload, BalanceApiVersion, CosmosSDKMsg, CosmosSDKTxInfo, HDWallet,
    Network, PublicKeyBytesWrapper, SecretKey, SingleCoin, WalletCoin,
    COMPRESSED_SECP256K1_PUBKEY_SIZE,
};
use wasm_bindgen::prelude::*;
/// wasm utilities
mod utils;

/// HD Wallet wrapper for Wasm
#[wasm_bindgen]
pub struct Wallet {
    wallet: HDWallet,
}

/// Signing key wrapper for Wasm
#[wasm_bindgen]
pub struct PrivateKey {
    key: Arc<SecretKey>,
}

#[wasm_bindgen]
impl PrivateKey {
    /// generate a random signing key
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            key: Arc::new(SecretKey::new()),
        }
    }
}

impl Default for PrivateKey {
    fn default() -> Self {
        Self::new()
    }
}

/// basic supported coins for wasm
/// TODO: re-work with `Network`
/// (wasm only supports C-style enums)
#[wasm_bindgen]
pub enum CoinType {
    /// Crypto.org Chain mainnet
    CryptoOrgMainnet,
    /// Crypto.org Chain testnet
    CryptoOrgTestnet,
    /// Cronos mainnet beta
    CronosMainnet,
    /// Cosmos Hub mainnet
    CosmosHub,
}

impl From<CoinType> for WalletCoin {
    fn from(coin: CoinType) -> Self {
        WalletCoin::CosmosSDK {
            network: match coin {
                CoinType::CryptoOrgMainnet => Network::CryptoOrgMainnet,
                CoinType::CryptoOrgTestnet => Network::CryptoOrgTestnet,
                CoinType::CronosMainnet => Network::CronosMainnet,
                CoinType::CosmosHub => Network::CosmosHub,
            },
        }
    }
}

#[wasm_bindgen]
impl Wallet {
    /// generate a random wallet (with an optional password)
    #[wasm_bindgen(constructor)]
    pub fn new(password: Option<String>) -> Self {
        Self {
            wallet: HDWallet::generate_wallet(password),
        }
    }

    /// return the default address for a given coin type
    #[wasm_bindgen]
    pub fn get_default_address(&self, coin: CoinType) -> Result<String, JsValue> {
        self.wallet
            .get_default_address(coin.into())
            .map_err(|e| JsValue::from_str(&format!("error: {}", e)))
    }

    /// obtain a signing key for a given derivation path
    #[wasm_bindgen]
    pub fn get_key(&self, derivation_path: String) -> Result<PrivateKey, JsValue> {
        let key = self
            .wallet
            .get_key(derivation_path)
            .map_err(|e| JsValue::from_str(&format!("error: {}", e)))?;
        Ok(PrivateKey { key })
    }
}

/// the common transaction data needed for Cosmos SDK transactions
/// (raw duplicate needed for Wasm -- TODO: unify common structures?)
#[wasm_bindgen(getter_with_clone)]
pub struct CosmosSDKTxInfoRaw {
    /// global account number of the sender
    pub account_number: u64,
    /// equivalent of "account nonce"
    pub sequence_number: u64,
    /// the maximum gas limit
    pub gas_limit: u64,
    /// the amount fee to be paid (gas_limit * gas_price)
    pub fee_amount: u64,
    /// the fee's denomination
    pub fee_denom: String,
    /// transaction timeout
    pub timeout_height: u32,
    /// optional memo
    pub memo_note: Option<String>,
    /// the network chain id
    pub chain_id: String,
    /// bech32 human readable prefix
    pub bech32hrp: String,
    /// the coin type to use
    pub coin_type: u32,
}

#[wasm_bindgen]
impl CosmosSDKTxInfoRaw {
    /// constructor for JS -- TODO: some builder API wrapper?
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_number: u64,
        sequence_number: u64,
        gas_limit: u64,
        fee_amount: u64,
        fee_denom: String,
        timeout_height: u32,
        memo_note: Option<String>,
        chain_id: String,
        bech32hrp: String,
        coin_type: u32,
    ) -> Self {
        Self {
            account_number,
            sequence_number,
            gas_limit,
            fee_amount,
            fee_denom,
            timeout_height,
            memo_note,
            chain_id,
            bech32hrp,
            coin_type,
        }
    }
}

impl From<CosmosSDKTxInfoRaw> for CosmosSDKTxInfo {
    fn from(info: CosmosSDKTxInfoRaw) -> Self {
        CosmosSDKTxInfo {
            account_number: info.account_number,
            sequence_number: info.sequence_number,
            gas_limit: info.gas_limit,
            fee_amount: SingleCoin::Other {
                amount: info.fee_amount.to_string(),
                denom: info.fee_denom,
            },
            timeout_height: info.timeout_height,
            memo_note: info.memo_note,
            network: Network::Other {
                chain_id: info.chain_id,
                coin_type: info.coin_type,
                bech32hrp: info.bech32hrp,
            },
        }
    }
}

/// creates the transaction signing payload (`SignDoc`)
/// for MsgSend from the Cosmos SDK bank module
/// wasm-bindgen only supports the C-style enums,
/// hences this duplicate function
#[wasm_bindgen]
pub fn get_single_bank_send_signdoc(
    tx_info: CosmosSDKTxInfoRaw,
    sender_pubkey: Vec<u8>,
    recipient_address: String,
    amount: u64,
    denom: String,
) -> Result<Vec<u8>, JsValue> {
    if sender_pubkey.len() != COMPRESSED_SECP256K1_PUBKEY_SIZE {
        return Err(JsValue::from_str("invalid public key length"));
    }
    let pubkey = PublicKeyBytesWrapper(sender_pubkey);
    get_single_msg_sign_payload(
        tx_info.into(),
        CosmosSDKMsg::BankSend {
            recipient_address,
            amount: SingleCoin::Other {
                amount: format!("{}", amount),
                denom,
            },
        },
        pubkey,
    )
    .map_err(|e| JsValue::from_str(&format!("error: {}", e)))
}

/// creates the signed transaction
/// for MsgSend from the Cosmos SDK bank module
/// wasm-bindgen only supports the C-style enums,
/// hences this duplicate function
#[wasm_bindgen]
pub fn get_single_bank_send_signed_tx(
    tx_info: CosmosSDKTxInfoRaw,
    private_key: PrivateKey,
    recipient_address: String,
    amount: u64,
    denom: String,
) -> Result<Vec<u8>, JsValue> {
    build_signed_single_msg_tx(
        tx_info.into(),
        CosmosSDKMsg::BankSend {
            recipient_address,
            amount: SingleCoin::Other {
                amount: format!("{}", amount),
                denom,
            },
        },
        private_key.key,
    )
    .map_err(|e| JsValue::from_str(&format!("error: {}", e)))
}

#[wasm_bindgen]
pub fn get_nft_issue_denom_signed_tx(
    tx_info: CosmosSDKTxInfoRaw,
    private_key: PrivateKey,
    id: String,
    name: String,
    schema: String,
) -> Result<Vec<u8>, JsValue> {
    build_signed_single_msg_tx(
        tx_info.into(),
        CosmosSDKMsg::NftIssueDenom { id, name, schema },
        private_key.key,
    )
    .map_err(|e| JsValue::from_str(&format!("error: {}", e)))
}

#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn get_nft_mint_signed_tx(
    tx_info: CosmosSDKTxInfoRaw,
    private_key: PrivateKey,
    id: String,
    denom_id: String,
    name: String,
    uri: String,
    data: String,
    recipient: String,
) -> Result<Vec<u8>, JsValue> {
    build_signed_single_msg_tx(
        tx_info.into(),
        CosmosSDKMsg::NftMint {
            id,
            denom_id,
            name,
            uri,
            data,
            recipient,
        },
        private_key.key,
    )
    .map_err(|e| JsValue::from_str(&format!("error: {}", e)))
}

#[wasm_bindgen]
pub fn get_nft_edit_signed_tx(
    tx_info: CosmosSDKTxInfoRaw,
    private_key: PrivateKey,
    id: String,
    denom_id: String,
    name: String,
    uri: String,
    data: String,
) -> Result<Vec<u8>, JsValue> {
    build_signed_single_msg_tx(
        tx_info.into(),
        CosmosSDKMsg::NftEdit {
            id,
            denom_id,
            name,
            uri,
            data,
        },
        private_key.key,
    )
    .map_err(|e| JsValue::from_str(&format!("error: {}", e)))
}

#[wasm_bindgen]
pub fn get_nft_transfer_signed_tx(
    tx_info: CosmosSDKTxInfoRaw,
    private_key: PrivateKey,
    id: String,
    denom_id: String,
    recipient: String,
) -> Result<Vec<u8>, JsValue> {
    build_signed_single_msg_tx(
        tx_info.into(),
        CosmosSDKMsg::NftTransfer {
            id,
            denom_id,
            recipient,
        },
        private_key.key,
    )
    .map_err(|e| JsValue::from_str(&format!("error: {}", e)))
}

#[wasm_bindgen]
pub fn get_nft_burn_signed_tx(
    tx_info: CosmosSDKTxInfoRaw,
    private_key: PrivateKey,
    id: String,
    denom_id: String,
) -> Result<Vec<u8>, JsValue> {
    build_signed_single_msg_tx(
        tx_info.into(),
        CosmosSDKMsg::NftBurn { id, denom_id },
        private_key.key,
    )
    .map_err(|e| JsValue::from_str(&format!("error: {}", e)))
}

/// retrieves the account details (e.g. sequence and account number) for a given address
/// TODO: switch to grpc-web
#[wasm_bindgen]
pub async fn query_account_details(api_url: String, address: String) -> Result<JsValue, JsValue> {
    let account_details = get_account_details(&api_url, &address)
        .await
        .map_err(|e| JsValue::from_str(&format!("error: {}", e)))?;

    Ok(JsValue::from_serde(&account_details)
        .map_err(|e| JsValue::from_str(&format!("error: {}", e)))?)
}

/// retrieves the account balance for a given address and a denom
/// api-version: https://github.com/cosmos/cosmos-sdk/releases/tag/v0.42.11
/// - 0 means before 0.42.11 or 0.44.4
/// - >=1 means after 0.42.11 or 0.44.4
/// TODO: switch to grpc-web
#[wasm_bindgen]
pub async fn query_account_balance(
    api_url: String,
    address: String,
    denom: String,
    api_version: u8,
) -> Result<JsValue, JsValue> {
    let balance_api_version = if api_version == 0 {
        BalanceApiVersion::Old
    } else {
        BalanceApiVersion::New
    };
    let account_details = get_account_balance(&api_url, &address, &denom, balance_api_version)
        .await
        .map_err(|e| JsValue::from_str(&format!("error: {}", e)))?;

    Ok(JsValue::from_serde(&account_details)
        .map_err(|e| JsValue::from_str(&format!("error: {}", e)))?)
}

/// broadcasts a signed tx
#[wasm_bindgen]
pub async fn broadcast_tx(
    tendermint_rpc_url: String,
    raw_signed_tx: Vec<u8>,
) -> Result<JsValue, JsValue> {
    let resp = broadcast_tx_sync(&tendermint_rpc_url, raw_signed_tx)
        .await
        .map_err(|e| JsValue::from_str(&format!("error: {}", e)))?
        .into_result()
        .map_err(|e| JsValue::from_str(&format!("missing_resulgt: {}", e)))?;

    Ok(JsValue::from_serde(&resp).map_err(|e| JsValue::from_str(&format!("error: {}", e)))?)
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
