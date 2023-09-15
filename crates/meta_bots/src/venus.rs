use chrono::prelude::*;
use ethers::prelude::*;
use meta_address::{enums::Asset, TokenInfo};
use meta_cefi::bitfinex::wallet::TradeExecutionUpdate;
use meta_common::enums::{CexExchange, DexExchange, Network};
use meta_dex::DexService;
use meta_integration::Lark;
use meta_model::{ArbitrageOutcome, ArbitrageSummary};
use meta_util::ether::get_network_scan_url;
use rust_decimal::Decimal;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Debug, Clone, Default)]
pub struct CexTradeInfo {
    pub venue: CexExchange,
    pub trade_info: Option<TradeExecutionUpdate>,
}

#[derive(Debug, Clone, Default)]
pub struct DexTradeInfo {
    pub network: Network,
    pub venue: DexExchange,
    pub tx_hash: Option<TxHash>,
    pub base_token_info: TokenInfo,
    pub quote_token_info: TokenInfo,
    pub v3_fee: Option<u32>,
    pub created: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ArbitragePair {
    pub datetime: DateTime<Utc>,
    pub base: Asset,
    pub quote: Asset,
    pub cex: CexTradeInfo,
    pub dex: DexTradeInfo,
}

pub type CID = u128; //client order id

#[derive(Debug)]
pub struct CexInstruction {
    pub venue: CexExchange,
    pub amount: Decimal,
    pub base_asset: Asset,
    pub quote_asset: Asset,
}

#[derive(Debug)]
pub struct DexInstruction {
    pub network: Network,
    pub venue: DexExchange,
    pub amount: Decimal,
    pub base_token: TokenInfo,
    pub quote_token: TokenInfo,
    pub fee: u32,
    pub recipient: Address,
}

#[derive(Debug)]
pub struct ArbitrageInstruction {
    pub cex: CexInstruction,
    pub dex: DexInstruction,
}

pub async fn check_arbitrage_status(
    map: Arc<RwLock<BTreeMap<CID, ArbitragePair>>>,
) -> Option<(bool, CID, ArbitragePair)> {
    info!("start check arbitrage status");
    let mut _g = map.read().await;
    let mut iter = _g.iter();

    let mut pending_status_tx_count = 0;
    let time = chrono::Utc::now();
    loop {
        let cur = iter.next();
        if cur.is_none() {
            break None;
        } else {
            let (key, val) = cur.unwrap();
            // tx sent, but still unknonw
            if val.dex.tx_hash.is_none() {
                info!("current time {:?}, created {:?}", time, val.dex.created);
                if time.signed_duration_since(val.dex.created).abs().num_seconds() > 1 {
                    pending_status_tx_count += 1;
                }
            }

            if pending_status_tx_count >= 2 {
                break Some((true, CID::default(), ArbitragePair::default()));
            }

            if val.cex.trade_info.is_some() && val.dex.tx_hash.is_some() {
                break Some((false, *key, val.clone()));
            } else {
                continue;
            }
        }
    }
}

pub async fn notify_arbitrage_result(
    arbitrage_map: Arc<RwLock<BTreeMap<CID, ArbitragePair>>>,
    lark: &Lark,
    provider: Arc<Provider<Ws>>,
    cid: CID,
    arbitrage_info: &ArbitragePair,
) {
    {
        let mut _g = arbitrage_map.write().await;
        _g.remove(&cid);
    }

    let dex_service =
        DexService::new(provider.clone(), arbitrage_info.dex.network, arbitrage_info.dex.venue);
    let dex_trade_info = arbitrage_info.dex.clone();
    let cex_trade_info = arbitrage_info.cex.clone();
    let hash = dex_trade_info.tx_hash.unwrap();
    let parsed_tx_ret = dex_service
        .analyze_v3_tx(
            hash,
            dex_trade_info.base_token_info.clone(),
            dex_trade_info.quote_token_info.clone(),
            dex_trade_info.v3_fee.unwrap(),
        )
        .await;
    match parsed_tx_ret {
        Ok(parsed_tx) => {
            let mut cex_outcome = ArbitrageOutcome::default();
            if let Some(info) = cex_trade_info.trade_info {
                cex_outcome.price = info.exec_price;
                cex_outcome.base_amount = info.exec_amount;
                cex_outcome.quote_amount = info
                    .exec_price
                    .saturating_mul(cex_outcome.base_amount)
                    .saturating_mul(Decimal::NEGATIVE_ONE);
                cex_outcome.fee_token = info.fee_currency.unwrap().parse::<Asset>().unwrap();
                cex_outcome.fee_amount = info.fee.unwrap();
                cex_outcome.id = cid.to_string();
            }

            let base_token = dex_trade_info.base_token_info.token;
            let quote_token = dex_trade_info.quote_token_info.token;
            let mut dex_outcome = ArbitrageOutcome::default();
            dex_outcome.base_amount =
                *parsed_tx.trade.get(&base_token).unwrap_or(&Decimal::default());
            dex_outcome.quote_amount =
                *parsed_tx.trade.get(&quote_token).unwrap_or(&Decimal::default());
            dex_outcome.price = dex_outcome
                .base_amount
                .checked_div(dex_outcome.quote_amount)
                .unwrap_or(Decimal::default())
                .abs();
            dex_outcome.fee_token = parsed_tx.fee.fee_token.into();
            dex_outcome.fee_amount = parsed_tx.fee.amount;
            dex_outcome.id = get_network_scan_url(dex_trade_info.network, hash);
            dex_outcome.network = Some(dex_trade_info.network);
            let summary = ArbitrageSummary {
                datetime: arbitrage_info.datetime.to_rfc3339(),
                base: arbitrage_info.base,
                quote: arbitrage_info.quote,
                cex: cex_outcome,
                dex: dex_outcome,
            };
            lark.send_arbitrage_summary(summary).await;
        }
        Err(e) => error!("error in analyze v3 tx {:?}", e),
    }
}
