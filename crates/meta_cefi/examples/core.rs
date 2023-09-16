use core_affinity::CoreId;
use once_cell::sync::Lazy;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CORE_IDS: Vec<CoreId> = core_affinity::get_core_ids().unwrap();
}

fn main() {
    println!("core id: {:?}", CORE_IDS[1]);
}
