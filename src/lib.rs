extern crate byteorder;
#[macro_use]
extern crate nom;

use nom::{IResult, le_u8, le_u16, le_u32, le_u64};

/// Magic Bytes of the Contact Header
const HEADER_MAGIC: [u8; 4] = [0x50, 0x43, 0x48, 0x21];  // PCH!

#[derive(Debug)]
pub struct PitchHeader {
    pub tag: Option<String>,
    pub item_count: u32,
    pub contents: Vec<IndexEntry>,
}

#[derive(Debug)]
pub struct IndexEntry {
    size: u64,
    name: String,
}

impl PitchHeader {
    pub fn new(tag: String) -> PitchHeader {
        PitchHeader { tag: Some(tag), item_count: 0, contents: Vec::new() }
    }

    pub fn deserialize(i: &[u8]) -> IResult<&[u8], PitchHeader> {
        pitch_header(i)
    }
}

named!(version< &[u8], u8>, map_opt!(le_u8,
    |x: u8| -> Option<u8> {
        match x {
        0x01 => Some(0x01),
        _ => None,
        }}));

named!(parse_length_string_u8< &[u8], Option<String> >,
    do_parse!(
        size: le_u8 >>
        raw_eid: take!(size) >>
        (match size {
            0 => None,
            _ => Some(String::from_utf8(raw_eid.to_vec()).unwrap()),
        })
));

fn index_entry(i: &[u8]) -> IResult<&[u8], Option<IndexEntry>> {
    let (i, length) = le_u16(i).unwrap();
    if length == 0 {
        return IResult::Done(&i, None);
    }
    let (i, file_size) = le_u64(i).unwrap();
    let length: usize = length as usize;
    let name = String::from_utf8(i[..length].to_vec()).unwrap();
    IResult::Done(&i[length..], Some(IndexEntry { name, size: file_size }))
}

fn index_entries(i: &[u8]) -> IResult<&[u8], Vec<IndexEntry>> {
    let mut contents = Vec::new();
    let mut i = i;
    loop {
        match index_entry(i) {
            IResult::Error(err) => { return IResult::Error(err); }
            IResult::Incomplete(needed) => { return IResult::Incomplete(needed); }
            IResult::Done(i_n, None) => {
                i = i_n;
                break;
            }
            IResult::Done(i_n, Some(item)) => {
                i = i_n;
                contents.push(item);
            }
        }
    }
    IResult::Done(&i, contents)
}

named!(pitch_header<PitchHeader>,
    do_parse!(
        return_error!(nom::ErrorKind::Custom(257), tag!(HEADER_MAGIC)) >>
        version: return_error!(nom::ErrorKind::Custom(258), version) >>
        item_count: le_u32 >>
        tag: parse_length_string_u8 >>
        contents: index_entries >>
        (PitchHeader {
            tag: tag,
            item_count,
            contents: contents,
            })));

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
