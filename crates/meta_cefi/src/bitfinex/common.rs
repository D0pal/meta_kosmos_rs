pub static P0: &'static str = "P0";
pub static P1: &'static str = "P1";
pub static P2: &'static str = "P2";
pub static P3: &'static str = "P3";
pub static R0: &'static str = "R0";

pub const CONF_FLAG_SEQ_ALL: u32 = 65536u32; // Adds sequence numbers to each event. This allows you to see if you are experiencing package loss or if you are receiving messages in a different order than they were sent from our server BETA FEATURE
pub const CONF_OB_CHECKSUM: u32 = 131072u32; // Enable checksum for every book iteration. Checks the top 25 entries for each side of book. Checksum is a signed int.
