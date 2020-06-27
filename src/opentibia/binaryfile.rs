use std::io;
use crate::helpers::ReadExt;

#[derive(Debug)]
pub struct Node {
    pub kind: u8,
    pub data: Vec<u8>,
    pub children: Vec<Node>,
}

impl Node {
    const ESCAPE: u8 = 0xFD;
    const START: u8 = 0xFE;
    const END: u8 = 0xFF;

    pub fn deserialize(r: &mut dyn io::Read, skip_start: bool) -> io::Result<Node> {
        if !skip_start {
            let data = r.read_byte()?;

            if data != Node::START {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected start of a node"));
            }
        }

        let kind = r.read_byte()?;

        let mut data = Vec::new();
        let mut children = Vec::new();

        loop {
            let b = r.read_byte()?;

            match b {
                Node::START => children.push(Node::deserialize(r, true)?),
                Node::END => break,
                Node::ESCAPE => data.push(r.read_byte()?),
                _ => data.push(b),
            }
        }

        data.shrink_to_fit();
        children.shrink_to_fit();

        Ok(Node {
            kind: kind,
            data: data,
            children: children,
        })
    }
}

pub fn streaming_parser<R, F>(mut r: R, skip_start: bool, mut callback: F) -> io::Result<()>
    where F: FnMut(u8, &[u8]) -> io::Result<bool>,
          R: io::Read
{
    if !skip_start {
        let data = r.read_byte()?;

        if data != Node::START {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected start of a node"));
        }
    }

    let mut kind = r.read_byte()?;
    let mut data = Vec::new();

    loop {
        let b = match r.read_byte() {
            Ok(b) => b,
            Err(ref a) if a.kind() == io::ErrorKind::UnexpectedEof => {
                // The last node is not yet processed at this point.
                // We don't care about the result since this is at EOF
                callback(kind, &data)?;
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        match b {
            Node::START => {
                let callback_result = callback(kind, &data)?;
                data.clear();

                // Stop parsing if callback returned false
                if !callback_result {
                    return Ok(());
                }

                kind = r.read_byte()?;
            }

            Node::END => (),
            Node::ESCAPE => data.push(r.read_byte()?),
            _ => data.push(b),
        }
    }
}
