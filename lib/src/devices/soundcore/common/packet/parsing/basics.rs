use nom::{
    IResult, Parser,
    bytes::complete::take,
    combinator::{map, map_opt},
    error::{ContextError, ParseError, context},
    multi::many0,
    number::complete::le_u8,
};

pub fn take_bool<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], bool, E> {
    map(le_u8, |value| value == 1).parse_complete(input)
}

/// Parses a single Tag-Length-Value field: one tag byte, one length byte, then
/// `length` value bytes. The returned value slice borrows from the input.
///
/// Newer Soundcore devices (e.g. the "D" series such as the Liberty 5 Pro Max)
/// encode their state update packet as a sequence of these fields instead of the
/// fixed byte layout used by older "A" series devices.
pub fn take_tlv_field<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (u8, &'a [u8]), E> {
    context("tlv field", |input| {
        let (input, tag) = le_u8(input)?;
        let (input, length) = le_u8(input)?;
        let (input, value) = take(length as usize)(input)?;
        Ok((input, (tag, value)))
    })
    .parse_complete(input)
}

/// Parses a sequence of TLV fields until the input is exhausted (or a partial
/// field is encountered), returning each `(tag, value)` pair in order.
pub fn take_tlv_fields<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Vec<(u8, &'a [u8])>, E> {
    context("tlv fields", many0(take_tlv_field)).parse_complete(input)
}

pub fn take_str<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    len: usize,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'a str, E> {
    move |input| map_opt(take(len), |bytes| std::str::from_utf8(bytes).ok()).parse_complete(input)
}
