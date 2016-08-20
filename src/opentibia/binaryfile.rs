use std::io;
use helpers::ReadExt;

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

    pub fn deserialize(r: &mut io::Read, skip_start: bool) -> io::Result<Node> {
        if !skip_start {
            let data = try!(r.read_byte());

            if data != Node::START {
                let invalid_data_error: io::Error = io::Error::new(io::ErrorKind::InvalidInput,
                                                                   "expected start of a node");
                return Err(invalid_data_error);
            }
        }

        let kind = try!(r.read_byte());

        let mut data = Vec::new();
        let mut children = Vec::new();

        loop {
            let b = try!(r.read_byte());

            match b {
                Node::START => children.push(try!(Node::deserialize(r, true))),
                Node::END => break,
                Node::ESCAPE => data.push(try!(r.read_byte())),
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
        let data = try!(r.read_byte());

        if data != Node::START {
            let invalid_data_error = io::Error::new(io::ErrorKind::InvalidInput, "expected start of a node");
            return Err(invalid_data_error);
        }
    }

    let mut kind = try!(r.read_byte());
    let mut data = Vec::new();

    loop {
        let b = match r.read_byte() {
            Ok(b) => b,
            Err(ref a) if a.kind() == io::ErrorKind::UnexpectedEof => {
                // The last node is not yet processed at this point.
                // We don't care about the result since this is at EOF
                try!(callback(kind, &data));
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        match b {
            Node::START => {
                let callback_result = try!(callback(kind, &data));
                data.clear();

                // Stop parsing if callback returned false
                if !callback_result {
                    return Ok(());
                }

                kind = try!(r.read_byte());
            }

            Node::END => (),
            Node::ESCAPE => data.push(try!(r.read_byte())),
            _ => data.push(b),
        }
    }
}
