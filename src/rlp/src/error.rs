#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    /// Data has additional bytes at the end of the valid RLP fragment.
    RlpIsTooBig,
    /// Data has too few bytes for valid RLP.
    RlpIsTooShort,
    /// Expect an encoded list, RLP was something else.
    RlpExpectedToBeList,
    /// Expect encoded data, RLP was something else.
    RlpExpectedToBeData,
    /// Expected a different size list.
    RlpIncorrectListLen,
    /// Data length number has a prefixed zero byte, invalid for numbers.
    RlpDataLenWithZeroPrefix,
    /// List length number has a prefixed zero byte, invalid for numbers.
    RlpListLenWithZeroPrefix,
    /// Non-canonical (longer than necessary) representation used for data or list.
    RlpInvalidIndirection,
    /// Declared length is inconsistent with data specified after.
    RlpInconsistentLengthAndData,
    /// Declared length is invalid and results in overflow
    RlpInvalidLength,
    /// Custom rlp decoding error.
    Custom(&'static str),
}