/// From the Verilator documentation: "Data representing 'bit' of 1-8 packed
/// bits."
pub type CData = u8;

/// From the Verilator documentation: "Data representing 'bit' of 9-16
/// packed bits"
pub type SData = u16;

/// From the Verilator documentation: "Data representing 'bit' of 17-32
/// packed bits."
pub type IData = u32;

/// From the Verilator documentation: "Data representing 'bit' of 33-64
/// packed bits."
pub type QData = u64;

/// From the Verilator documentation: "Data representing one element of
/// WData array."
pub type EData = u32;

/// From the Verilator documentation: "Data representing >64 packed bits
/// (used as pointer)."
#[deprecated(
    note = "Verilator 5.052 or later no longer uses this type (#7642)"
)]
pub type WData = EData;

/// From the Verilator documentation: "'bit' of >64 packed bits as array
/// input to a function."
pub type WDataInP = *const EData;

/// From the Verilator documentation: "'bit' of >64 packed bits as array
/// output from a function."
pub type WDataOutP = *mut EData;
