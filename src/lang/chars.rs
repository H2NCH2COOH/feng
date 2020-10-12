use super::error::Error;

pub struct Chars<'a, B>
where
    B: Iterator<Item = std::io::Result<u8>>,
{
    bytes: &'a mut B,
    ended: bool,
}

impl<'a, B> Chars<'a, B>
where
    B: Iterator<Item = std::io::Result<u8>>,
{
    pub fn new(bytes: &'a mut B) -> Self {
        Chars {
            bytes,
            ended: false,
        }
    }
}

impl<'a, B> Iterator for Chars<'a, B>
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
                if (new_byte & 0b1000_0000) == 0b0000_0000 {
                    // ASCII
                    return Some(Ok(std::char::from_u32(new_byte as u32).unwrap()));
                } else if (new_byte & 0b1110_0000) == 0b1100_0000 {
                    // Two bytes
                    bc = 2;
                    point = ((new_byte & 0b0001_1111) as u32) << 6;
                } else if (new_byte & 0b1111_0000) == 0b1110_0000 {
                    // Three bytes
                    bc = 3;
                    point = ((new_byte & 0b0000_1111) as u32) << 6;
                } else if (new_byte & 0b1111_1000) == 0b1111_0000 {
                    // Four bytes
                    bc = 4;
                    point = ((new_byte & 0b0000_0111) as u32) << 6;
                } else {
                    break;
                }

                bc -= 1;
            } else if bc > 0 && (new_byte & 0b1100_0000) == 0b1000_0000 {
                point |= (new_byte & 0b0011_1111) as u32;
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
        Some(Err(Error::Utf8Err(staging)))
    }
}
