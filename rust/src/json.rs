//! A small JSON value module with the semantics of the Python
//! json module wherever the loaders' behaviour depends on them.
//!
//! The semantics were probed against Python 3.12 on this machine,
//! and the probes stand in the unit tests below and in the port
//! gate report.
//!
//! A duplicate object key keeps the last value, standing at the
//! position of its first key, as a Python dict holds it. A number
//! is classified from its raw literal. An integer literal is held
//! exactly as a big integer, and an integer literal of more than
//! 4300 digits is refused, as the Python int conversion refuses
//! it. A literal holding a point or an exponent is a float, read
//! with the correctly rounded conversion both runtimes share, so
//! a literal that overflows reads as an infinity and one that
//! underflows reads as a zero. The three constant spellings the
//! Python reader accepts, NaN, Infinity, and minus Infinity, read
//! as those floats. A string is held as a sequence of UTF-16 code
//! units, so an unpaired surrogate is representable, is detected,
//! and is refused by name by the loaders, as the Python loaders
//! refuse it. Nesting past the depth the Python reader reaches is
//! refused as its own error kind, because the Python surfaces
//! part on it. The render command lets the failure propagate and
//! the review command names the file as holding no change set.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;

use crate::pycompat::is_python_space_unit;

/// The most digits an integer literal may hold, as the Python
/// int conversion limit stands in this runtime.
pub const MAX_INT_DIGITS: usize = 4300;

/// The deepest nesting the reader accepts. The Python reader
/// leans on the interpreter stack, so its flip point depends on
/// the route that invokes it. Measured through the installed
/// stickler command, the console script entry the package
/// ships, the reader accepts 9997 open containers and refuses
/// the 9998th on the review and render surfaces alike, for bare
/// arrays, for a field nested in arrays, and for nested
/// objects. The python3 -m stickler.cli route refuses at 9994
/// containers, an artifact of that route's deeper interpreter
/// stack, and the port does not chase it. The limit stands at
/// the console script count.
pub const MAX_DEPTH: usize = 9997;

/// A JSON number, classified from its raw literal.
#[derive(Clone, Debug)]
pub enum JsonNumber {
    /// An integer literal, held exactly at any size.
    Int(BigInt),
    /// A literal holding a point or an exponent, or one of the
    /// three constant spellings, held as a double.
    Float(f64),
}

impl JsonNumber {
    /// Return whether this number is an integer.
    pub fn is_int(&self) -> bool {
        matches!(self, JsonNumber::Int(_))
    }

    /// Return whether this number is finite, read without
    /// conversion. An integer is read by comparison against the
    /// largest finite float, as the seat readers read it, so an
    /// integer with hundreds of digits is refused by the
    /// comparison instead of raising.
    pub fn is_finite(&self) -> bool {
        match self {
            JsonNumber::Int(value) => {
                let max = largest_finite_float_int();
                *value >= -max.clone() && *value <= max
            }
            JsonNumber::Float(value) => value.is_finite(),
        }
    }

    /// Return the exact rational this number stands for, or None
    /// when a float is not finite and so stands for no rational.
    pub fn to_rational(&self) -> Option<BigRational> {
        match self {
            JsonNumber::Int(value) => Some(BigRational::new(value.clone(), BigInt::from(1))),
            JsonNumber::Float(value) => float_to_rational(*value),
        }
    }

    /// Render the number as the Python str of the value renders
    /// it. An integer renders as its digits, and a float renders
    /// in the shortest round trip form.
    pub fn render(&self) -> String {
        match self {
            JsonNumber::Int(value) => value.to_string(),
            JsonNumber::Float(value) => crate::pycompat::repr_float(*value),
        }
    }
}

impl PartialEq for JsonNumber {
    /// Numeric equality as Python reads it, across the integer
    /// and float forms. One is exactly equal to one point zero.
    fn eq(&self, other: &JsonNumber) -> bool {
        match (self.to_rational(), other.to_rational()) {
            (Some(left), Some(right)) => left == right,
            _ => match (self, other) {
                (JsonNumber::Float(left), JsonNumber::Float(right)) => left == right,
                _ => false,
            },
        }
    }
}

/// The largest finite float held as an exact integer. The value
/// is the largest double, read as the integer it stands for.
fn largest_finite_float_int() -> BigInt {
    // The largest double is (2^53 - 1) * 2^971.
    ((BigInt::from(1) << 53) - BigInt::from(1)) << 971
}

/// Read a float as the exact rational it stands for.
///
/// A double is a sign, a mantissa, and a power of two, and the
/// rational is that product read exactly. Not a number and the
/// infinities stand for no rational and return None.
fn float_to_rational(value: f64) -> Option<BigRational> {
    if !value.is_finite() {
        return None;
    }
    if value == 0.0 {
        return Some(BigRational::zero());
    }
    let bits = value.to_bits();
    let negative = bits >> 63 == 1;
    let exponent_bits = ((bits >> 52) & 0x7ff) as i64;
    let fraction = (bits & ((1u64 << 52) - 1)) as i128;
    let (mantissa, exponent) = if exponent_bits == 0 {
        (fraction, -1022 - 52)
    } else {
        (fraction + (1i128 << 52), exponent_bits - 1023 - 52)
    };
    let mut numerator = BigInt::from(mantissa);
    if negative {
        numerator = -numerator;
    }
    if exponent >= 0 {
        Some(BigRational::new(
            numerator << (exponent as usize),
            BigInt::from(1),
        ))
    } else {
        Some(BigRational::new(
            numerator,
            BigInt::from(1) << ((-exponent) as usize),
        ))
    }
}

/// A JSON string, held as a sequence of UTF-16 code units.
///
/// Python strings are code point sequences and a lone surrogate
/// escape in a JSON text stands in the string as a lone surrogate
/// code point. Holding the units keeps such a string
/// representable here, so the loaders can detect it and refuse it
/// by name, as the Python loaders do.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JString(pub Vec<u16>);

impl JString {
    /// Build a JSON string from a Rust string.
    pub fn from_text(text: &str) -> JString {
        JString(text.encode_utf16().collect())
    }

    /// Build a JSON string from raw bytes as the interpreter
    /// builds the argument strings in sys.argv. The bytes decode
    /// as UTF-8 under the surrogateescape error handler the
    /// interpreter decodes arguments with, so each byte that
    /// stands in no valid sequence reads as the surrogate for
    /// that byte. A file name carrying a byte such as 0xFF so
    /// stands in the string as an unpaired surrogate, where the
    /// loaders refuse it by name, as the Python loaders do.
    pub fn from_surrogateescape(bytes: &[u8]) -> JString {
        JString(decode_surrogateescape(bytes))
    }

    /// Encode the string back to raw bytes under the same
    /// handler, as the interpreter encodes a path for the file
    /// system and a printed line for its output stream. A paired
    /// surrogate reads as its character, and an unpaired
    /// surrogate standing for a byte reads back as that byte.
    /// Any other unpaired surrogate reads as its own three byte
    /// form, a case no argument stands in.
    pub fn to_surrogateescape(&self) -> Vec<u8> {
        encode_surrogateescape(&self.0)
    }

    /// Return the Rust string this JSON string stands for, or
    /// None when it holds an unpaired surrogate and so stands
    /// for no printable string.
    pub fn to_rust_string(&self) -> Option<String> {
        String::from_utf16(&self.0).ok()
    }

    /// Return whether the string holds an unpaired surrogate.
    /// Such a string cannot be encoded as UTF-8 text and so
    /// cannot be printed, which is the reading the Python loaders
    /// give it.
    pub fn holds_unpaired_surrogate(&self) -> bool {
        self.to_rust_string().is_none()
    }

    /// Return whether the string holds a line break, a newline
    /// or a carriage return.
    pub fn holds_line_break(&self) -> bool {
        self.0.iter().any(|unit| *unit == 0x0a || *unit == 0x0d)
    }

    /// Return whether the string is blank, holding nothing but
    /// Python whitespace, as the validators read a blank field.
    pub fn is_blank(&self) -> bool {
        self.0.iter().all(|unit| is_python_space_unit(*unit))
    }

    /// Return whether the string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return whether this string equals a Rust string slice,
    /// unit for unit.
    pub fn equals(&self, text: &str) -> bool {
        self.0.iter().copied().eq(text.encode_utf16())
    }
}

/// The shape of one UTF-8 sequence, its length in bytes and the
/// range its second byte must stand in. The ranges are the ones
/// the interpreter's decoder enforces, which refuse the overlong
/// forms and the surrogate forms at the second byte.
fn sequence_shape(lead: u8) -> Option<(usize, u8, u8)> {
    match lead {
        0xc2..=0xdf => Some((2, 0x80, 0xbf)),
        0xe0 => Some((3, 0xa0, 0xbf)),
        0xe1..=0xec => Some((3, 0x80, 0xbf)),
        0xed => Some((3, 0x80, 0x9f)),
        0xee..=0xef => Some((3, 0x80, 0xbf)),
        0xf0 => Some((4, 0x90, 0xbf)),
        0xf1..=0xf3 => Some((4, 0x80, 0xbf)),
        0xf4 => Some((4, 0x80, 0x8f)),
        _ => None,
    }
}

/// Decode bytes under the surrogateescape handler. A valid
/// sequence reads as its character. Where a sequence fails, the
/// lead byte and every continuation byte that validly follows
/// it escape one by one as the surrogates for their bytes, and
/// decoding resumes after them, which is the error span the
/// interpreter's decoder reports for the sequence.
fn decode_surrogateescape(bytes: &[u8]) -> Vec<u16> {
    let mut units = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let lead = bytes[i];
        if lead < 0x80 {
            units.push(lead as u16);
            i += 1;
            continue;
        }
        let Some((length, low, high)) = sequence_shape(lead) else {
            units.push(0xdc00 + lead as u16);
            i += 1;
            continue;
        };
        let mut valid = 0usize;
        if i + 1 < bytes.len() && (low..=high).contains(&bytes[i + 1]) {
            valid = 1;
            while valid < length - 1 {
                match bytes.get(i + 1 + valid) {
                    Some(next) if (0x80..=0xbf).contains(next) => {
                        valid += 1;
                    }
                    _ => break,
                }
            }
        }
        if valid == length - 1 {
            let mut scalar = (lead as u32) & (0x7f >> length);
            for step in 0..valid {
                scalar = (scalar << 6) | (bytes[i + 1 + step] as u32 & 0x3f);
            }
            let character = char::from_u32(scalar).expect("a shaped sequence is a scalar");
            let mut buffer = [0u16; 2];
            units.extend_from_slice(character.encode_utf16(&mut buffer));
            i += length;
        } else {
            for step in 0..=valid {
                units.push(0xdc00 + bytes[i + step] as u16);
            }
            i += valid + 1;
        }
    }
    units
}

/// Encode code units under the surrogateescape handler, the
/// inverse of the decode above. A surrogate pair reads as its
/// character, a lone surrogate standing for a byte reads as
/// that byte, and any other lone surrogate reads as the three
/// bytes of its own value, which no decoded argument holds.
fn encode_surrogateescape(units: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut i = 0usize;
    while i < units.len() {
        let unit = units[i];
        if (0xd800..0xdc00).contains(&unit)
            && i + 1 < units.len()
            && (0xdc00..0xe000).contains(&units[i + 1])
        {
            let scalar =
                0x10000 + (((unit as u32) - 0xd800) << 10) + ((units[i + 1] as u32) - 0xdc00);
            let character = char::from_u32(scalar).expect("a surrogate pair is a scalar");
            let mut buffer = [0u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            i += 2;
            continue;
        }
        if (0xdc80..0xe000).contains(&unit) {
            bytes.push((unit - 0xdc00) as u8);
            i += 1;
            continue;
        }
        if (0xd800..0xe000).contains(&unit) {
            bytes.push(0xe0 | (unit >> 12) as u8);
            bytes.push(0x80 | ((unit >> 6) & 0x3f) as u8);
            bytes.push(0x80 | (unit & 0x3f) as u8);
            i += 1;
            continue;
        }
        let character = char::from_u32(unit as u32).expect("a lone unit is a scalar");
        let mut buffer = [0u8; 4];
        bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
        i += 1;
    }
    bytes
}

/// A JSON value.
#[derive(Debug)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    Str(JString),
    Array(Vec<JsonValue>),
    Object(Vec<(JString, JsonValue)>),
}

/// Cloning is iterative, as the reading and the disposal
/// are. A parsed tree can stand at the deepest nesting the
/// reader accepts, and the derived clone would recurse to
/// that depth and overflow the stack of a small thread. The
/// walk keeps its work on a heap list and assembles each
/// container from the clones of its children standing on a
/// second list, so the depth of the tree never becomes the
/// depth of the call stack on any path that clones a value.
impl Clone for JsonValue {
    fn clone(&self) -> JsonValue {
        enum Work<'a> {
            Value(&'a JsonValue),
            Array(usize),
            Object(Vec<JString>, usize),
        }
        let mut work = vec![Work::Value(self)];
        let mut done: Vec<JsonValue> = Vec::new();
        while let Some(item) = work.pop() {
            match item {
                Work::Value(value) => match value {
                    JsonValue::Null => done.push(JsonValue::Null),
                    JsonValue::Bool(flag) => done.push(JsonValue::Bool(*flag)),
                    JsonValue::Number(number) => done.push(JsonValue::Number(number.clone())),
                    JsonValue::Str(text) => done.push(JsonValue::Str(text.clone())),
                    JsonValue::Array(items) => {
                        work.push(Work::Array(items.len()));
                        for child in items.iter().rev() {
                            work.push(Work::Value(child));
                        }
                    }
                    JsonValue::Object(pairs) => {
                        let keys: Vec<JString> = pairs.iter().map(|pair| pair.0.clone()).collect();
                        work.push(Work::Object(keys, pairs.len()));
                        for pair in pairs.iter().rev() {
                            work.push(Work::Value(&pair.1));
                        }
                    }
                },
                Work::Array(count) => {
                    let children = done.split_off(done.len() - count);
                    done.push(JsonValue::Array(children));
                }
                Work::Object(keys, count) => {
                    let children = done.split_off(done.len() - count);
                    done.push(JsonValue::Object(keys.into_iter().zip(children).collect()));
                }
            }
        }
        done.pop().expect("the walk clones the value it was handed")
    }
}

/// Disposal is iterative, as the reading is. A parsed tree can
/// stand at the deepest nesting the reader accepts, and the
/// derived disposal would recurse to that depth and overflow
/// the stack of a small thread. The children are drained into
/// a work list, so each value is disposed of holding only
/// empty containers and scalars, and the depth of the tree
/// never becomes the depth of the call stack.
impl Drop for JsonValue {
    fn drop(&mut self) {
        let mut work: Vec<JsonValue> = Vec::new();
        self.drain_children(&mut work);
        while let Some(mut value) = work.pop() {
            value.drain_children(&mut work);
        }
    }
}

impl JsonValue {
    /// Move a value's nested containers into the work list,
    /// leaving the value itself holding only empty containers
    /// and scalars.
    fn drain_children(&mut self, work: &mut Vec<JsonValue>) {
        match self {
            JsonValue::Array(items) => work.append(items),
            JsonValue::Object(pairs) => {
                work.extend(pairs.drain(..).map(|pair| pair.1));
            }
            _ => {}
        }
    }

    /// Return whether this value is an object.
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    /// Return the value standing under a key of an object, or
    /// None when this value is no object or holds no such key.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(pairs) => pairs
                .iter()
                .find(|(name, _value)| name.equals(key))
                .map(|(_name, value)| value),
            _ => None,
        }
    }

    /// Return whether an object holds a key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Return the entries of an object, or None for any other
    /// value.
    pub fn as_object(&self) -> Option<&Vec<(JString, JsonValue)>> {
        match self {
            JsonValue::Object(pairs) => Some(pairs),
            _ => None,
        }
    }

    /// Return the items of an array, or None for any other value.
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Return the string this value holds, or None for any other
    /// value.
    pub fn as_jstring(&self) -> Option<&JString> {
        match self {
            JsonValue::Str(text) => Some(text),
            _ => None,
        }
    }

    /// Return the number this value holds, or None for any other
    /// value, booleans included.
    pub fn as_number(&self) -> Option<&JsonNumber> {
        match self {
            JsonValue::Number(number) => Some(number),
            _ => None,
        }
    }
}

impl PartialEq for JsonValue {
    /// Equality as Python reads two parsed values. Objects are
    /// equal when they hold the same keys with equal values, in
    /// whatever order the keys stand.
    fn eq(&self, other: &JsonValue) -> bool {
        match (self, other) {
            (JsonValue::Null, JsonValue::Null) => true,
            (JsonValue::Bool(left), JsonValue::Bool(right)) => left == right,
            (JsonValue::Number(left), JsonValue::Number(right)) => left == right,
            (JsonValue::Str(left), JsonValue::Str(right)) => left == right,
            (JsonValue::Array(left), JsonValue::Array(right)) => left == right,
            (JsonValue::Object(left), JsonValue::Object(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .all(|(key, value)| other.get_key(key).is_some_and(|found| found == value))
            }
            _ => false,
        }
    }
}

impl JsonValue {
    fn get_key(&self, key: &JString) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(pairs) => pairs
                .iter()
                .find(|(name, _value)| name == key)
                .map(|(_name, value)| value),
            _ => None,
        }
    }
}

/// The ways a JSON text fails to read.
///
/// Syntax and the integer digit limit are the failures the
/// Python reader raises as a ValueError. TooDeep is the failure
/// it raises as a RecursionError, and the surfaces part on the
/// two kinds, so the kinds stand apart here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    Syntax,
    TooDeep,
    IntTooLong,
}

/// Parse a JSON text held as a Rust string.
///
/// The text a command reads from a file arrives already decoded
/// as UTF-8, as the Python readers decode it, and is widened to
/// code units here so the string reading matches the module's
/// one reading everywhere.
pub fn parse(text: &str) -> Result<JsonValue, ParseError> {
    let units: Vec<u16> = text.encode_utf16().collect();
    parse_units(&units)
}

/// Parse a JSON text held as UTF-16 code units.
///
/// The chat seat reads a decision out of an envelope content
/// string that already stands as code units, and parses it
/// through this same door.
pub fn parse_units(units: &[u16]) -> Result<JsonValue, ParseError> {
    let mut parser = Parser { units, pos: 0 };
    parser.skip_whitespace();
    let value = parser.parse_document()?;
    parser.skip_whitespace();
    if parser.pos != units.len() {
        return Err(ParseError::Syntax);
    }
    Ok(value)
}

/// One open container on the reader's explicit stack.
///
/// The reader keeps its containers on a heap stack rather than
/// in nested calls, so the depth the Python reader reaches by
/// recursion is reached here without a call stack to overflow.
enum Frame {
    Array(Vec<JsonValue>),
    Object {
        pairs: Vec<(JString, JsonValue)>,
        key: JString,
    },
}

struct Parser<'a> {
    units: &'a [u16],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u16> {
        self.units.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u16> {
        let unit = self.peek();
        if unit.is_some() {
            self.pos += 1;
        }
        unit
    }

    fn skip_whitespace(&mut self) {
        while let Some(unit) = self.peek() {
            if matches!(unit, 0x20 | 0x09 | 0x0a | 0x0d) {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), ParseError> {
        for expected in literal.encode_utf16() {
            if self.bump() != Some(expected) {
                return Err(ParseError::Syntax);
            }
        }
        Ok(())
    }

    /// Parse one whole document. The containers open on the
    /// explicit stack, and a finished value is delivered into
    /// the container waiting for it until none is.
    fn parse_document(&mut self) -> Result<JsonValue, ParseError> {
        let mut stack: Vec<Frame> = Vec::new();
        'read: loop {
            self.skip_whitespace();
            let mut value = match self.peek() {
                Some(0x7b) => {
                    if stack.len() >= MAX_DEPTH {
                        return Err(ParseError::TooDeep);
                    }
                    self.bump();
                    self.skip_whitespace();
                    if self.peek() == Some(0x7d) {
                        self.bump();
                        JsonValue::Object(Vec::new())
                    } else {
                        let key = self.parse_key()?;
                        stack.push(Frame::Object {
                            pairs: Vec::new(),
                            key,
                        });
                        continue 'read;
                    }
                }
                Some(0x5b) => {
                    if stack.len() >= MAX_DEPTH {
                        return Err(ParseError::TooDeep);
                    }
                    self.bump();
                    self.skip_whitespace();
                    if self.peek() == Some(0x5d) {
                        self.bump();
                        JsonValue::Array(Vec::new())
                    } else {
                        stack.push(Frame::Array(Vec::new()));
                        continue 'read;
                    }
                }
                _ => self.parse_scalar()?,
            };
            'deliver: loop {
                match stack.last_mut() {
                    None => return Ok(value),
                    Some(Frame::Array(items)) => {
                        items.push(value);
                        self.skip_whitespace();
                        match self.bump() {
                            Some(0x2c) => continue 'read,
                            Some(0x5d) => {
                                let Frame::Array(items) = stack.pop().expect("frame stands") else {
                                    unreachable!()
                                };
                                value = JsonValue::Array(items);
                                continue 'deliver;
                            }
                            _ => return Err(ParseError::Syntax),
                        }
                    }
                    Some(Frame::Object { pairs, key }) => {
                        let key = key.clone();
                        if let Some(slot) = pairs.iter_mut().find(|(name, _)| *name == key) {
                            slot.1 = value;
                        } else {
                            pairs.push((key, value));
                        }
                        self.skip_whitespace();
                        match self.bump() {
                            Some(0x2c) => {
                                let next = self.parse_key()?;
                                if let Some(Frame::Object { key, .. }) = stack.last_mut() {
                                    *key = next;
                                }
                                continue 'read;
                            }
                            Some(0x7d) => {
                                let Frame::Object { pairs, .. } =
                                    stack.pop().expect("frame stands")
                                else {
                                    unreachable!()
                                };
                                value = JsonValue::Object(pairs);
                                continue 'deliver;
                            }
                            _ => return Err(ParseError::Syntax),
                        }
                    }
                }
            }
        }
    }

    /// Parse the key and colon that open one object entry. The
    /// cursor stands before the key's opening quote.
    fn parse_key(&mut self) -> Result<JString, ParseError> {
        self.skip_whitespace();
        if self.peek() != Some(0x22) {
            return Err(ParseError::Syntax);
        }
        let key = self.parse_string()?;
        self.skip_whitespace();
        if self.bump() != Some(0x3a) {
            return Err(ParseError::Syntax);
        }
        Ok(key)
    }

    /// Parse one value that is not a container.
    fn parse_scalar(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek() {
            Some(0x22) => Ok(JsonValue::Str(self.parse_string()?)),
            Some(0x74) => {
                self.expect_literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(0x66) => {
                self.expect_literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(0x6e) => {
                self.expect_literal("null")?;
                Ok(JsonValue::Null)
            }
            Some(0x4e) => {
                self.expect_literal("NaN")?;
                Ok(JsonValue::Number(JsonNumber::Float(f64::NAN)))
            }
            Some(0x49) => {
                self.expect_literal("Infinity")?;
                Ok(JsonValue::Number(JsonNumber::Float(f64::INFINITY)))
            }
            Some(0x2d) => {
                if self
                    .units
                    .get(self.pos + 1)
                    .is_some_and(|unit| *unit == 0x49)
                {
                    self.bump();
                    self.expect_literal("Infinity")?;
                    Ok(JsonValue::Number(JsonNumber::Float(f64::NEG_INFINITY)))
                } else {
                    self.parse_number()
                }
            }
            Some(unit) if (0x30..=0x39).contains(&unit) => self.parse_number(),
            _ => Err(ParseError::Syntax),
        }
    }

    fn parse_string(&mut self) -> Result<JString, ParseError> {
        self.bump();
        let mut units = Vec::new();
        loop {
            let unit = self.bump().ok_or(ParseError::Syntax)?;
            match unit {
                0x22 => return Ok(JString(units)),
                0x5c => {
                    let escape = self.bump().ok_or(ParseError::Syntax)?;
                    match escape {
                        0x22 => units.push(0x22),
                        0x5c => units.push(0x5c),
                        0x2f => units.push(0x2f),
                        0x62 => units.push(0x08),
                        0x66 => units.push(0x0c),
                        0x6e => units.push(0x0a),
                        0x72 => units.push(0x0d),
                        0x74 => units.push(0x09),
                        0x75 => {
                            let mut value: u16 = 0;
                            for _ in 0..4 {
                                let digit = self.bump().ok_or(ParseError::Syntax)?;
                                let digit = match digit {
                                    0x30..=0x39 => digit - 0x30,
                                    0x41..=0x46 => digit - 0x41 + 10,
                                    0x61..=0x66 => digit - 0x61 + 10,
                                    _ => return Err(ParseError::Syntax),
                                };
                                value = value * 16 + digit;
                            }
                            units.push(value);
                        }
                        _ => return Err(ParseError::Syntax),
                    }
                }
                0x00..=0x1f => return Err(ParseError::Syntax),
                _ => units.push(unit),
            }
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.pos;
        if self.peek() == Some(0x2d) {
            self.bump();
        }
        match self.peek() {
            Some(0x30) => {
                self.bump();
            }
            Some(unit) if (0x31..=0x39).contains(&unit) => {
                while let Some(unit) = self.peek() {
                    if (0x30..=0x39).contains(&unit) {
                        self.bump();
                    } else {
                        break;
                    }
                }
            }
            _ => return Err(ParseError::Syntax),
        }
        let mut is_float = false;
        if self.peek() == Some(0x2e) {
            is_float = true;
            self.bump();
            match self.peek() {
                Some(unit) if (0x30..=0x39).contains(&unit) => {}
                _ => return Err(ParseError::Syntax),
            }
            while let Some(unit) = self.peek() {
                if (0x30..=0x39).contains(&unit) {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        if let Some(unit) = self.peek() {
            if unit == 0x65 || unit == 0x45 {
                is_float = true;
                self.bump();
                if let Some(sign) = self.peek() {
                    if sign == 0x2b || sign == 0x2d {
                        self.bump();
                    }
                }
                match self.peek() {
                    Some(unit) if (0x30..=0x39).contains(&unit) => {}
                    _ => return Err(ParseError::Syntax),
                }
                while let Some(unit) = self.peek() {
                    if (0x30..=0x39).contains(&unit) {
                        self.bump();
                    } else {
                        break;
                    }
                }
            }
        }
        let literal: String = self.units[start..self.pos]
            .iter()
            .map(|unit| *unit as u8 as char)
            .collect();
        if is_float {
            let value: f64 = literal.parse().map_err(|_| ParseError::Syntax)?;
            return Ok(JsonValue::Number(JsonNumber::Float(value)));
        }
        let digits = literal.trim_start_matches('-');
        if digits.len() > MAX_INT_DIGITS {
            return Err(ParseError::IntTooLong);
        }
        let value = BigInt::parse_bytes(literal.as_bytes(), 10).ok_or(ParseError::Syntax)?;
        Ok(JsonValue::Number(JsonNumber::Int(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The decode vectors are the units the interpreter's own
    /// surrogateescape decode of the bytes gives, probed on this
    /// machine. The surrogate forms the UTF-8 decoder refuses,
    /// the truncated forms, and the overlong forms each escape
    /// byte by byte over the longest valid prefix.
    #[test]
    fn surrogateescape_decodes_as_the_interpreter_decodes() {
        let cases: &[(&[u8], &[u16])] = &[
            (b"plain", &[0x70, 0x6c, 0x61, 0x69, 0x6e]),
            (b"\xff", &[0xdcff]),
            (b"\xed\xa0\x80", &[0xdced, 0xdca0, 0xdc80]),
            (b"\xc3\xa9", &[0x00e9]),
            (b"\xf0\x9f\x98\x80", &[0xd83d, 0xde00]),
            (b"\xc3(", &[0xdcc3, 0x28]),
            (b"\xe2\x82", &[0xdce2, 0xdc82]),
            (b"\xe2\x82A", &[0xdce2, 0xdc82, 0x41]),
            (b"\xc0\x80", &[0xdcc0, 0xdc80]),
            (b"\xe0\x80\x80", &[0xdce0, 0xdc80, 0xdc80]),
            (b"\xf4\x90\x80\x80", &[0xdcf4, 0xdc90, 0xdc80, 0xdc80]),
            (b"\xf0\x9f", &[0xdcf0, 0xdc9f]),
            (b"a\x80b", &[0x61, 0xdc80, 0x62]),
            (b"\xe0\xa0\x80", &[0x0800]),
            (b"\xed\x9f\xbf", &[0xd7ff]),
            (b"\xee\x80\x80", &[0xe000]),
        ];
        for (bytes, units) in cases {
            let decoded = JString::from_surrogateescape(bytes);
            assert_eq!(decoded.0, units.to_vec(), "bytes {bytes:?}");
            assert_eq!(decoded.to_surrogateescape(), bytes.to_vec());
        }
    }

    fn int(value: &JsonValue) -> BigInt {
        match value {
            JsonValue::Number(JsonNumber::Int(int)) => int.clone(),
            other => panic!("expected an integer, got {other:?}"),
        }
    }

    fn float(value: &JsonValue) -> f64 {
        match value {
            JsonValue::Number(JsonNumber::Float(float)) => *float,
            other => panic!("expected a float, got {other:?}"),
        }
    }

    #[test]
    fn a_duplicate_key_keeps_the_last_value() {
        let value = parse("{\"a\": 1, \"a\": 2}").unwrap();
        assert_eq!(int(value.get("a").unwrap()), BigInt::from(2));
    }

    #[test]
    fn numbers_classify_from_their_literal() {
        assert!(matches!(
            parse("5").unwrap(),
            JsonValue::Number(JsonNumber::Int(_))
        ));
        assert!(matches!(
            parse("5.0").unwrap(),
            JsonValue::Number(JsonNumber::Float(_))
        ));
        assert!(matches!(
            parse("1e2").unwrap(),
            JsonValue::Number(JsonNumber::Float(_))
        ));
        assert_eq!(int(&parse("-0").unwrap()), BigInt::from(0));
        assert_eq!(float(&parse("-0.0").unwrap()), -0.0);
    }

    #[test]
    fn an_integer_is_held_exactly_at_any_size() {
        let literal = format!("1{}", "0".repeat(400));
        let value = parse(&literal).unwrap();
        assert_eq!(
            int(&value),
            BigInt::parse_bytes(literal.as_bytes(), 10).unwrap()
        );
    }

    #[test]
    fn an_integer_literal_past_the_digit_limit_is_refused() {
        let ok = "9".repeat(MAX_INT_DIGITS);
        assert!(parse(&ok).is_ok());
        let too_long = "9".repeat(MAX_INT_DIGITS + 1);
        assert_eq!(parse(&too_long), Err(ParseError::IntTooLong));
    }

    #[test]
    fn a_float_literal_that_overflows_reads_as_an_infinity() {
        assert_eq!(float(&parse("1e400").unwrap()), f64::INFINITY);
        assert_eq!(float(&parse("-1e400").unwrap()), f64::NEG_INFINITY);
    }

    #[test]
    fn the_constant_spellings_read_as_their_floats() {
        assert!(float(&parse("NaN").unwrap()).is_nan());
        assert_eq!(float(&parse("Infinity").unwrap()), f64::INFINITY);
        assert_eq!(float(&parse("-Infinity").unwrap()), f64::NEG_INFINITY);
    }

    #[test]
    fn an_unpaired_surrogate_is_representable_and_detected() {
        let value = parse(r#""\ud800x""#).unwrap();
        let text = value.as_jstring().unwrap();
        assert!(text.holds_unpaired_surrogate());
        assert!(text.to_rust_string().is_none());
    }

    #[test]
    fn a_surrogate_pair_reads_as_its_character() {
        let value = parse(r#""\ud83d\ude00""#).unwrap();
        let text = value.as_jstring().unwrap();
        assert!(!text.holds_unpaired_surrogate());
        assert_eq!(text.to_rust_string().unwrap(), "\u{1f600}");
    }

    #[test]
    fn malformed_texts_are_refused() {
        for bad in [
            "{'a':1}",
            "{\"a\":1,}",
            "{\"a\":01}",
            "{\"a\":+1}",
            "{\"a\":.5}",
            "{\"a\":1.}",
            "{a:1}",
            "\"unterminated",
            "\"bad \\x escape\"",
            "{\"a\": 1} extra",
            "\"a\u{0001}b\"",
            "\u{feff}{\"a\": 1}",
        ] {
            assert_eq!(parse(bad), Err(ParseError::Syntax), "text {bad:?}");
        }
    }

    #[test]
    fn nesting_past_the_limit_is_its_own_error() {
        let deep = "[".repeat(MAX_DEPTH + 5) + &"]".repeat(MAX_DEPTH + 5);
        assert_eq!(parse(&deep), Err(ParseError::TooDeep));
        let fine = "[".repeat(200) + &"]".repeat(200);
        assert!(parse(&fine).is_ok());
    }

    #[test]
    fn finiteness_reads_by_range_before_any_conversion() {
        let huge = parse(&format!("1{}", "0".repeat(400))).unwrap();
        let huge = huge.as_number().unwrap();
        assert!(!huge.is_finite());
        let boundary = JsonNumber::Int(largest_finite_float_int());
        assert!(boundary.is_finite());
        let past = JsonNumber::Int(largest_finite_float_int() + 1);
        assert!(!past.is_finite());
    }

    #[test]
    fn numeric_equality_crosses_the_int_and_float_forms() {
        assert_eq!(parse("1").unwrap(), parse("1.0").unwrap());
        assert_ne!(parse("1").unwrap(), parse("1.5").unwrap());
    }
}
