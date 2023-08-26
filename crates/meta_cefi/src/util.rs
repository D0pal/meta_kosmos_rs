use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::cefi_service::KeyedOrderBook;

pub fn to_decimal(input: f64) -> Decimal {
    Decimal::from_f64(input).unwrap()
}
