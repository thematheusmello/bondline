use crate::{Error, MAX_INPUT_BYTES};

/// Decode signed decimal digits with an optional, nonempty all-zero fractional part.
/// No floating-point conversion or rounding is used. Exponents are not supported.
pub fn parse_integer(value: &str) -> Result<i64, Error> {
    if value.len() > MAX_INPUT_BYTES {
        return Err(Error::InputTooLarge);
    }
    let value = value.trim_matches([' ', '\t', '\r', '\n']);
    let integer = if let Some((integer, fraction)) = value.split_once('.') {
        if fraction.is_empty() || fraction.bytes().any(|byte| byte != b'0') {
            return Err(Error::InvalidInteger);
        }
        integer
    } else {
        value
    };
    integer.parse().map_err(|_| Error::InvalidInteger)
}

/// Source identity only: zero means unassigned; nonzero IDs can be container-scoped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(u32);

impl ObjectId {
    pub const fn get(self) -> u32 {
        self.0
    }
    pub const fn is_assigned(self) -> bool {
        self.0 != 0
    }
}

impl std::str::FromStr for ObjectId {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        u32::try_from(parse_integer(value)?)
            .map(Self)
            .map_err(|_| Error::InvalidObjectId)
    }
}
