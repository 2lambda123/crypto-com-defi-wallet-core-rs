use ethers::prelude::{Chain, Eip1559TransactionRequest, ParseChainError, ProviderError};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::{
    prelude::{Address, LocalWallet, Signer, TransactionRequest, U256},
    utils::{parse_units, ConversionError},
};
use std::{str::FromStr, sync::Arc};

use crate::{SecretKey, WalletCoin};
use ethers::prelude::abi;
use ethers::types::H160;
use serde::{Deserialize, Serialize};

/// Possible errors from Ethereum transaction construction and broadcasting
#[derive(Debug, thiserror::Error)]
pub enum EthError {
    #[error("Converting from hexadecimal failed")]
    HexConversion,
    #[error("Converting from decimal failed: {0}")]
    ParseError(ConversionError),
    #[error("Invalid node Web3 connection URL")]
    NodeUrl,
    #[error("Transaction sending failed")]
    SendTxFail,
    #[error("Transaction sending failed: {0}")]
    BroadcastTxFail(ProviderError),
    #[error("Transaction dropped from the mempool")]
    MempoolDrop,
    #[error("Failed to obtain an account balance")]
    BalanceFail,
    #[error("Async Runtime error")]
    AsyncRuntimeError,
    #[error("Contract call error")]
    ContractError,
    #[error("Signature error")]
    SignatureError,
    #[error("Chainid error: {0}")]
    ChainidError(ParseChainError),
}

/// Ethereum networks
/// the string conversion is from: https://github.com/gakonst/ethers-rs/blob/4fd9c7800ee9afd5395d8c7b8652d788b9e80f35/ethers-core/src/types/chain.rs#L130
/// e.g. "mainnet" == Ethereum mainnet
pub enum EthNetwork {
    Known { name: String },
    Custom { chain_id: u64, legacy: bool },
}

impl EthNetwork {
    /// returns the chain id and if the chain needs the legacy
    /// transaction request
    pub fn to_chain_params(self) -> Result<(u64, bool), EthError> {
        match self {
            EthNetwork::Known { name } => {
                let chain = Chain::from_str(&name).map_err(EthError::ChainidError)?;
                Ok((chain as u64, chain.is_legacy()))
            }
            EthNetwork::Custom { chain_id, legacy } => Ok((chain_id, legacy)),
        }
    }
}

/// The gas/native token amount
/// in decimal notation
pub enum EthAmount {
    /// 10^-18 ETH
    WeiDecimal {
        amount: String,
    },
    /// 10^-9 ETH
    GweiDecimal {
        amount: String,
    },
    EthDecimal {
        amount: String,
    },
}

impl TryInto<U256> for EthAmount {
    type Error = ConversionError;

    fn try_into(self) -> Result<U256, Self::Error> {
        match self {
            EthAmount::WeiDecimal { amount } => parse_units(amount, "wei"),
            EthAmount::GweiDecimal { amount } => parse_units(amount, "gwei"),
            EthAmount::EthDecimal { amount } => parse_units(amount, "ether"),
        }
    }
}

/// constructs a simple transfer of Eth/native token on a given network
pub fn construct_simple_eth_transfer_tx(
    from_hex: &str,
    to_hex: &str,
    amount: EthAmount,
    legacy_tx: bool,
    chain_id: u64,
) -> Result<TypedTransaction, EthError> {
    let from = Address::from_str(from_hex).map_err(|_| EthError::HexConversion)?;
    let to = Address::from_str(to_hex).map_err(|_| EthError::HexConversion)?;
    let amount: U256 = amount.try_into().map_err(EthError::ParseError)?;
    if legacy_tx {
        Ok(TransactionRequest::pay(to, amount)
            .from(from)
            .chain_id(chain_id)
            .into())
    } else {
        Ok(Eip1559TransactionRequest::new()
            .to(to)
            .value(amount)
            .from(from)
            .chain_id(chain_id)
            .into())
    }
}

/// constructs an unsigned simple transfer of Eth/native token on a given network
pub fn construct_unsigned_eth_tx(
    from_hex: &str,
    to_hex: &str,
    amount: EthAmount,
    network: EthNetwork,
    legacy_tx: bool,
) -> Result<Vec<u8>, EthError> {
    let (chain_id, legacy) = network.to_chain_params()?;

    let tx =
        construct_simple_eth_transfer_tx(from_hex, to_hex, amount, legacy || legacy_tx, chain_id)?;
    Ok(tx.rlp().to_vec())
}

/// A common information for ethereum transactions
pub struct EthTxInfo {
    /// the destination address as a hexadecimal string
    pub to_address: String,
    /// the amount to send
    pub amount: EthAmount,
    /// the nonce as a decimal string
    pub nonce: String,
    /// the gas limit as a decimal string
    pub gas_limit: String,
    /// the gas price to pay
    pub gas_price: EthAmount,
    /// optional data
    pub data: Option<Vec<u8>>,
    /// use the legacy tx format (even if the chain supports EIP-1559)
    pub legacy_tx: bool,
}

/// builds a signed ethereum transaction given the inputs
pub fn build_signed_eth_tx(
    tx_info: EthTxInfo,
    network: EthNetwork,
    secret_key: Arc<SecretKey>,
) -> Result<Vec<u8>, EthError> {
    let (chain_id, legacy) = network.to_chain_params()?;

    let from_address = WalletCoin::Ethereum
        .derive_address(&secret_key.get_signing_key())
        .map_err(|_| EthError::HexConversion)?;
    let mut tx: TypedTransaction = construct_simple_eth_transfer_tx(
        &from_address,
        &tx_info.to_address,
        tx_info.amount,
        tx_info.legacy_tx || legacy,
        chain_id,
    )?;
    tx.set_nonce(
        U256::from_dec_str(&tx_info.nonce)
            .map_err(|e| EthError::ParseError(ConversionError::FromDecStrError(e)))?,
    );
    tx.set_gas(
        U256::from_dec_str(&tx_info.gas_limit)
            .map_err(|e| EthError::ParseError(ConversionError::FromDecStrError(e)))?,
    );
    let gas_price: U256 = tx_info.gas_price.try_into().map_err(EthError::ParseError)?;
    tx.set_gas_price(gas_price);
    if let Some(data) = tx_info.data {
        tx.set_data(data.into());
    }
    let wallet = LocalWallet::from(secret_key.get_signing_key()).with_chain_id(chain_id);
    let sig = wallet.sign_hash(tx.sighash(), false);
    let signed_tx = &tx.rlp_signed(&sig);
    Ok(signed_tx.to_vec())
}

///
pub struct Contract {
    abi_contract: abi::Contract,
}

impl Contract {
    pub fn new(data: &str) -> Self {
        Self {
            abi_contract: abi::Contract::load(data.as_bytes()).unwrap(),
        }
    }

    pub fn encode_input(&self, function_name: &str, tokens: Vec<Token>) -> Vec<u8> {
        let fun = self.abi_contract.function(function_name).unwrap();
        let tokens: Vec<abi::token::Token> = tokens.into_iter().map(Into::into).collect();
        fun.encode_input(&tokens).unwrap()
    }
}

///
#[derive(Serialize, Deserialize)]
pub enum Token {
    Address(H160),
    FixedBytes(Vec<u8>),
    Bytes(Vec<u8>),
    Int(U256),
    Uint(U256),
    Bool(bool),
    String(String),
    FixedArray(Vec<Token>),
    Array(Vec<Token>),
    Tuple(Vec<Token>),
}

impl Token {
    pub fn build_address(address_str: &str) -> Self {
        Self::Address(Address::from_str(address_str).unwrap())
    }

    pub fn build_int(u256_str: &str) -> Self {
        Self::Int(U256::from_str(u256_str).unwrap())
    }

    pub fn build_uint(u256_str: &str) -> Self {
        Self::Uint(U256::from_str(u256_str).unwrap())
    }
}

impl From<Token> for abi::token::Token {
    fn from(token: Token) -> Self {
        match token {
            Token::Address(val) => abi::token::Token::Address(val),
            Token::FixedBytes(val) => abi::token::Token::FixedBytes(val),
            Token::Bytes(val) => abi::token::Token::Bytes(val),
            Token::Int(val) => abi::token::Token::Int(val),
            Token::Uint(val) => abi::token::Token::Uint(val),
            Token::Bool(val) => abi::token::Token::Bool(val),
            Token::String(val) => abi::token::Token::String(val),
            Token::FixedArray(vec) => {
                abi::token::Token::FixedArray(vec.into_iter().map(|val| val.into()).collect())
            }
            Token::Array(vec) => {
                abi::token::Token::FixedArray(vec.into_iter().map(|val| val.into()).collect())
            }
            Token::Tuple(vec) => {
                abi::token::Token::Tuple(vec.into_iter().map(|val| val.into()).collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;
    use std::sync::Arc;

    use ethers::utils::rlp::Rlp;

    use crate::*;

    #[test]
    fn eth_tx_works() {
        let tx_raw = construct_unsigned_eth_tx(
            "0x2c600e0a72b3ae39e9b27d2e310b180abe779368",
            "0x2c600e0a72b3ae39e9b27d2e310b180abe779368",
            EthAmount::EthDecimal {
                amount: "1".to_string(),
            },
            EthNetwork::Known {
                name: "cronos".into(),
            },
            false,
        )
        .expect("ok signed tx");
        assert!(Rlp::new(&tx_raw).payload_info().is_ok());
    }

    #[test]
    fn eth_signing_works() {
        let secret_key = SecretKey::new();
        let tx_info = EthTxInfo {
            to_address: "0x2c600e0a72b3ae39e9b27d2e310b180abe779368".to_string(),
            amount: EthAmount::EthDecimal {
                amount: "1".to_string(),
            },
            nonce: "0".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: EthAmount::WeiDecimal {
                amount: "7".to_string(),
            },
            data: Some(vec![]),
            legacy_tx: false,
        };
        let tx_raw = build_signed_eth_tx(
            tx_info,
            EthNetwork::Known {
                name: "cronos".into(),
            },
            Arc::new(secret_key),
        )
        .expect("ok signed tx");
        assert!(Rlp::new(&tx_raw).payload_info().is_ok());
    }

    use crate::{SecretKey, WalletCoin};
    use ethers::abi::token::Token::{Address, Uint};
    use ethers::abi::Contract;
    use ethers::types::H160;
    use ethers::types::U256;
    use ethers::utils::hex;

    #[test]
    fn eth_tx_test() {
        // check normal tx
        let hex = "24e585759e492f5e810607c82c202476c22c5876b10247ebf8b2bb7f75dbed2e";
        let secret_key =
            SecretKey::from_hex(hex.to_owned()).expect("Failed to construct Secret Key from hex");
        println!(
            "{}",
            secret_key
                .to_address(WalletCoin::Ethereum)
                .expect("address error")
        );
        let tx_info = EthTxInfo {
            to_address: "0x4592d8f8d7b001e72cb26a73e4fa1806a51ac79d".to_string(),
            amount: EthAmount::EthDecimal {
                amount: "1".to_string(),
            },
            nonce: "0".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: EthAmount::WeiDecimal {
                amount: "1000".to_string(),
            },
            data: Some(vec![]),
            legacy_tx: true,
        };

        let tx_raw = build_signed_eth_tx(
            tx_info,
            EthNetwork::Custom {
                chain_id: 0,
                legacy: true,
            },
            Arc::new(secret_key),
        )
        .expect("ok signed tx");
        assert_eq!(hex::encode(tx_raw),"f869808203e8825208944592d8f8d7b001e72cb26a73e4fa1806a51ac79d880de0b6b3a7640000801ba01997d312edfb72eea35788c9241eb8a693a23730920149468eda7a114e66f570a063aaa8bb4cec6a129d378487e93fea759782b741109751f8a235b479814289c4");
    }

    #[test]
    fn eth_contract_test() {
        let b = "[{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"initialSupply\",\"type\":\"uint256\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"Approval\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"Transfer\",\"type\":\"event\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"}],\"name\":\"allowance\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"}],\"name\":\"approve\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"balanceOf\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"decimals\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"subtractedValue\",\"type\":\"uint256\"}],\"name\":\"decreaseAllowance\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"addedValue\",\"type\":\"uint256\"}],\"name\":\"increaseAllowance\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"symbol\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"totalSupply\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"}],\"name\":\"transfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"}],\"name\":\"transferFrom\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]".as_bytes();
        let contract = Contract::load(b).expect("abi error");
        let func = contract.function("transfer").expect("fun error");
        let raw = func
            .encode_input(&[
                Address(
                    H160::from_str("0x2c600e0a72b3ae39e9b27d2e310b180abe779368")
                        .expect("address error"),
                ),
                Uint(U256::from(100)),
            ])
            .expect("encode error");
        assert_eq!("a9059cbb0000000000000000000000002c600e0a72b3ae39e9b27d2e310b180abe7793680000000000000000000000000000000000000000000000000000000000000064", hex::encode(raw.clone()));

        // check contract tx
        let hex = "24e585759e492f5e810607c82c202476c22c5876b10247ebf8b2bb7f75dbed2e";
        let secret_key =
            SecretKey::from_hex(hex.to_owned()).expect("Failed to construct Secret Key from hex");
        println!(
            "{}",
            secret_key
                .to_address(WalletCoin::Ethereum)
                .expect("address error")
        );
        let tx_info = EthTxInfo {
            to_address: "0x4592d8f8d7b001e72cb26a73e4fa1806a51ac79d".to_string(),
            amount: EthAmount::EthDecimal {
                amount: "1".to_string(),
            },
            nonce: "0".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: EthAmount::WeiDecimal {
                amount: "1000".to_string(),
            },
            data: Some(raw),
            legacy_tx: true,
        };

        let tx_raw = build_signed_eth_tx(
            tx_info,
            EthNetwork::Custom {
                chain_id: 0,
                legacy: true,
            },
            Arc::new(secret_key),
        )
        .expect("ok signed tx");
        assert_eq!(hex::encode(tx_raw),"f8ae808203e8825208944592d8f8d7b001e72cb26a73e4fa1806a51ac79d880de0b6b3a7640000b844a9059cbb0000000000000000000000002c600e0a72b3ae39e9b27d2e310b180abe77936800000000000000000000000000000000000000000000000000000000000000641ba095845d357e85e871c56a4f2a5cb0418f38c2275ea223c79336e64cb4f28c423ea07e1a148e3131bd7a47eb85c336d79c55b953a4f04dc349236256e0c62c3d4754");
    }
}
