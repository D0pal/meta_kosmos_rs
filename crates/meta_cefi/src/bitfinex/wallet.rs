use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletSnapshot {
    pub wallet_type: String, // exchange, margin
    pub currency: String,
    pub balance: Decimal,
    pub un_settled_interest: Decimal,       //	float	Unsettled interest
    pub balance_available: Option<Decimal>, //	float / null	Amount not tied up in active orders, positions or funding (null if the value has not yet been calculated).pub
    pub description: Option<String>,        //	string	Description of the ledger entry
    pub meta: Option<Value>, //	Provides info on the reason for the wallet update, if available.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PositionSnapshot {
    pub symbol: String, // Pair (tBTCUSD, …).
    pub status: String, // Status (ACTIVE, CLOSED).
    pub amount: Decimal,
    pub base_price: Decimal,
    pub margin_funding: Decimal,
    pub margin_funding_type: i32,
    pub pl: Decimal,
    pub pl_perc: Decimal,
    pub price_liq: Decimal,
    pub leverage: Decimal,
    pub position_id: i32,
    pub mts_create: u64,
    pub mts_update: u64,
    pub position_type: i32,
    pub collateral: Decimal,
    pub collateral_min: Decimal,
    pub meta: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundingCreditSnapshot {
    pub credit_id: i32,
    pub symbol: String,  //	The currency of the credit (fUSD, etc)
    pub side: i32, //	int	1 if you are the lender, 0 if you are both the lender and borrower, -1 if you're the borrower
    pub mts_create: u64, //	int	Millisecond Time Stamp when the credit was created
    pub mts_update: u64, //	int	Millisecond Time Stamp when the credit was created
    pub amount: Decimal, //	float	Amount the credit is for
                   // FLAGS	object	future params object (stay tuned)
                   // STATUS	string	Credit Status: ACTIVE, EXECUTED, PARTIALLY FILLED, CANCELED
                   // RATE	float	Rate of the credit
                   // PERIOD	int	Period of the credit
                   // MTS_OPENING	int	Millisecond Time Stamp when the funding was opened
                   // MTS_LAST_PAYOUT	int	Millisecond Time Stamp when the last payout was received
                   // NOTIFY	int	0 if false, 1 if true
                   // HIDDEN	int	0 if false, 1 if true
                   // RENEW	int	0 if false, 1 if true
                   // RATE_REAL	float	the calculated rate for FRR and FRRDELTAFIX
                   // NO_CLOSE	int	0 if false, 1 if true (whether the funding should be closed when the position is closed)
                   // POSITION_PAIR	string	The pair of the position that the funding is used for
}

// unknown, needs to check
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BU {
    // bu
    pub a: Decimal,
    pub b: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewOrderOnReq {
    pub mts: u64,                //	int	Millisecond Time Stamp of the update
    pub msg_type: String, //	string	Purpose of notification ('on-req', 'oc-req', 'uca', 'fon-req', 'foc-req')
    pub message_id: Option<u64>, // 	int	unique ID of the message
    pub id: Option<u64>,  //ID	int	Order ID
    pub order: Order,
    pub a: Option<String>,
    pub status: String, // STATUS	string	Status of the notification; it may vary over time (SUCCESS, ERROR, FAILURE, ...)
    pub text: String,   // TEXT	string	Text of the notification
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub gid: u64,       //	int	Group ID
    pub cid: u64,       //	int	Client Order ID
    pub symbol: String, //	string	Pair (tBTCUSD, …)

    pub mts_create: u64,      //	int	Millisecond timestamp of creation
    pub mts_update: u64,      //	int	Millisecond timestamp of update
    pub amount: Decimal,      //	float	Positive means buy, negative means sell.
    pub amount_orig: Decimal, //	float	Original amount

    pub order_type: String, // 	string	The type of the order: LIMIT, EXCHANGE LIMIT, MARKET, EXCHANGE MARKET, STOP, EXCHANGE STOP, STOP LIMIT, EXCHANGE STOP LIMIT, TRAILING STOP, EXCHANGE TRAILING STOP, FOK, EXCHANGE FOK, IOC, EXCHANGE IOC.
    pub type_prev: Option<String>, //	string	Previous order type
    pub a: Option<String>,
    pub b: Option<String>,

    pub mts_tif: u64, //	int	Millisecond timestamp of Time-In-Force: automatic order cancellation
    pub order_status: String, //	string	Order Status: ACTIVE
    pub c: Option<String>,
    pub d: Option<String>,

    pub price: Decimal,           //	float	Price
    pub price_avg: Decimal,       //	float	Average price
    pub price_traling: Decimal,   //	float	The trailing price
    pub price_aux_limit: Decimal, // PRICE_AUX_LIMIT	float	Auxiliary Limit price (for STOP LIMIT)

    pub e: Option<String>,
    pub f: Option<String>,
    pub g: Option<String>,
    pub h: u64,

    pub hiddern: i32, // HIDDEN	int	0 if false, 1 if true
    pub i: Option<String>,
    pub j: Option<String>,
    pub k: Option<String>,

    // pub placed_id: i32, // PLACED_ID	int	If another order caused this order to be placed (OCO) this will be that other order's ID
    pub routing: String, // ROUTING	string	indicates origin of action: BFX, ETHFX, API>BFX, API>ETHFX
    // pub flags: i32,     // FLAGS	int	See https://docs.bitfinex.com/v2/docs/flag-values.
    pub l: Option<String>,
    pub m: Option<String>,
    pub meta: Value, // META	json string	Additional meta information about the order ( $F7 = IS_POST_ONLY (0 if false, 1 if true), $F33 = Leverage (int))
                     // pub code: Option<i32>, //  CODE	null or integer	Work in progress
                     //
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeEvent {
    pub id: u64,
    pub symbol: String,
    pub gid: u64,
    pub cid: u64,

    pub amount: Decimal,
    pub qty: Decimal,
    pub order_type: String,
    pub amt: Decimal,

    pub quantity: Decimal,
    pub a: Option<String>,
    pub b: Option<String>,
    pub c: u64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcEvent {  // oc

    // [125271920288,0,1693153165935,\"tARBUSD\",
    pub id: u64,
    pub gid: u64,       //	int	Group ID
    pub cid: u64,       //	int	Client Order ID
    pub symbol: String, //	string	Pair (tBTCUSD, …)
 // 1693153166200,1693153166202,0,1,
    pub mts_create: u64,      //	int	Millisecond timestamp of creation
    pub mts_update: u64,      //	int	Millisecond timestamp of update
    pub amount: Decimal,      //	float	Positive means buy, negative means sell.
    pub amount_orig: Decimal, //	float	Original amount
 // \"EXCHANGE MARKET\",null,null,null,
    pub order_type: String, // 	string	The type of the order: LIMIT, EXCHANGE LIMIT, MARKET, EXCHANGE MARKET, STOP, EXCHANGE STOP, STOP LIMIT, EXCHANGE STOP LIMIT, TRAILING STOP, EXCHANGE TRAILING STOP, FOK, EXCHANGE FOK, IOC, EXCHANGE IOC.
    pub type_prev: Option<String>, //	string	Previous order type
    pub a: Option<String>,
    pub b: Option<String>,

    // 0,\"EXECUTED @ 0.95902(1.0)\",null,null,
    pub o: i32,
    pub text: String,
    pub p: Option<String>,
    pub q: Option<String>,

    // 0.9591,0.95902,0,0,
    pub qnt: Decimal, //	int	Millisecond timestamp of Time-In-Force: automatic order cancellation
    pub amt: Decimal, //	string	Order Status: ACTIVE
    pub c: i32,
    pub d: i32,

    // null,null,null,0,
    pub e: Option<String>,
    pub f: Option<String>,
    pub g: Option<String>,
    pub h: u64,

    // 0,null,null,null,\"API>BFX\",null,null,{}]
    pub hiddern: i32, // HIDDEN	int	0 if false, 1 if true
    pub i: Option<String>,
    pub j: Option<String>,
    pub k: Option<String>,

    // pub placed_id: i32, // PLACED_ID	int	If another order caused this order to be placed (OCO) this will be that other order's ID
    pub routing: String, // ROUTING	string	indicates origin of action: BFX, ETHFX, API>BFX, API>ETHFX
    // pub flags: i32,     // FLAGS	int	See https://docs.bitfinex.com/v2/docs/flag-values.
    pub l: Option<String>,
    pub m: Option<String>,
    pub meta: Value, // META	json string	Additional meta information about the order ( $F7 = IS_POST_ONLY (0 if false, 1 if true), $F33 = Leverage (int))
                     // pub code: Option<i32>, //  CODE	null or integer	Work in progress
                     //

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TuEvent {  // tu

    pub id: u64,
    pub symbol: String,
    pub gid: u64,
    pub cid: u64,
    pub amount_orig: Decimal,
    pub amount_exec: Decimal,
    pub trade_type : String,
    pub amount: Decimal,
    pub amount_real: Decimal,
    pub fee: Decimal,
    pub asset: String,
    pub created: u64
}