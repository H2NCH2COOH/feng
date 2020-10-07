use super::error::Error;

pub struct Chars<B>
where
    B: Iterator<Item = std::io::Result<u8>>,
{
    bytes: B,
    ended: bool,
}

impl<B> Chars<B>
where
    B: Iterator<Item = std::io::Result<u8>>,
{
    pub fn new(bytes: B) -> Self {
        Chars {
            bytes: bytes,
            ended: false,
        }
    }
}

impl<B> Iterator for Chars<B>
where
    B: Iterator<Item = std::io::Result<u8>>,
{
    type Item = Result<char, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        let mut count = 0;
        let mut staging = [0u8; 4];
        let mut point = 0u32;
        let mut bc = 0;
        loop {
            let new_byte = match self.bytes.next() {
                Some(Ok(b)) => b,
                Some(Err(e)) => {
                    self.ended = true;
                    return Some(Err(Error::IoErr(e)));
                }
                None => {
                    self.ended = true;
                    return None;
                }
            };

            staging[count] = new_byte;
            count += 1;

            if count == 1 {
                if (new_byte & 0x80) == 0x00 {
                    // ASCII
                    return Some(Ok(std::char::from_u32(new_byte as u32).unwrap()));
                } else if (new_byte & 0xE0) == 0xC0 {
                    // Two bytes
                    bc = 2;
                    point = ((new_byte & 0x1F) as u32) << 6;
                } else if (new_byte & 0xF0) == 0xE0 {
                    // Three bytes
                    bc = 3;
                    point = ((new_byte & 0x0F) as u32) << 6;
                } else if (new_byte & 0xF8) == 0xF0 {
                    // Four bytes
                    bc = 4;
                    point = ((new_byte & 0x07) as u32) << 6;
                } else {
                    break;
                }

                bc -= 1;
            } else if bc > 0 && (new_byte & 0xC0) == 0x80 {
                point |= (new_byte & 0x3F) as u32;
                bc -= 1;

                if bc == 0 {
                    return Some(Ok(std::char::from_u32(point).unwrap()));
                }

                point <<= 6;
            } else {
                break;
            }
        }

        self.ended = true;
        return Some(Err(Error::Utf8Err(staging)));
    }
}
