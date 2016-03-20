use std::convert;
use std::io::{self, Error};

use helpers::ReadExt;

const NODE_ESCAPE: u8 = 0xFD;
const NODE_START: u8 = 0xFE;
const NODE_END: u8 = 0xFF;

#[derive(Debug)]
pub struct Node {
    pub kind: u8,
    pub data: Vec<u8>,
    pub children: Vec<Node>
}

impl Node {
    pub fn deserialize(r: &mut io::Read, skip_start: bool) -> io::Result<Node> {
        if !skip_start {
            let data = try!(r.read_byte());

            if data != NODE_START {
                let invalid_data_error: io::Error = io::Error::new(io::ErrorKind::Other, "unexpected data");
                return Err(invalid_data_error)
            }
        }

        let kind = try!(r.read_byte());

        let mut data = Vec::new();
        let mut children = Vec::new();

        loop {
            let b = try!(r.read_byte());

            match b {
                NODE_START => children.push(try!(Node::deserialize(r, true))),
                NODE_END => break,
                NODE_ESCAPE => data.push(try!(r.read_byte())),
                _ => data.push(b)
            }
        }

        data.shrink_to_fit();
        children.shrink_to_fit();

        Ok(Node {
            kind: kind,
            data: data,
            children: children
        })
    }
}

pub fn streaming_parser<F>(r: &mut io::Read, skip_start: bool, mut callback: F) -> io::Result<()>
    where F: FnMut(u8, &[u8]) -> io::Result<bool> {

    if !skip_start {
        let data = try!(r.read_byte());

        if data != NODE_START {
            let invalid_data_error: io::Error = io::Error::new(io::ErrorKind::Other, "unexpected data");
            return Err(invalid_data_error)
        }
    }

    let mut kind = try!(r.read_byte());
    let mut data = Vec::new();

    loop {
        let b = match r.read_byte() {
            Ok(b) => b,
            Err(ref a) if a.kind() == io::ErrorKind::UnexpectedEof => { 
                // The last node is not yet processed at this point.
                // We don't care about the result since this is at EOF.
                try!(callback(kind, &data[..]));
                return Ok(())
            },
            Err(err) => return Err(err)
        };

        match b {
            NODE_START => {
                let callback_result = try!(callback(kind, &data[..]));
                data.clear();

                // Stop parsing if callback returned false
                if !callback_result {
                    return Ok(())
                }

                kind = try!(r.read_byte());
            },

            NODE_END => (),
            NODE_ESCAPE => data.push(try!(r.read_byte())),
            _ => data.push(b)
        }
    }
}
