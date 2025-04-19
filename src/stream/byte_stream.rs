use bytes::{BytesMut, BufMut, Buf};
use std::convert::TryInto;

pub struct ByteStream {
    buffer: BytesMut,
    offset: usize,
    bit_offset: u8,
}

impl ByteStream {
    pub fn new() -> Self {
        ByteStream {
            buffer: BytesMut::new(),
            offset: 0,
            bit_offset: 0,
        }
    }

    pub fn read_int(&mut self) -> i32 {
        self.bit_offset = 0;
        if self.offset + 4 <= self.buffer.len() {
            let value = i32::from_be_bytes(
                self.buffer[self.offset..self.offset + 4].try_into().unwrap()
            );
            self.offset += 4;
            value
        } else {
            0
        }
    }

    pub fn skip(&mut self, len: usize) {
        self.bit_offset = (self.bit_offset + len as u8) % 8;
        self.offset += len / 8;
    }

    pub fn read_short(&mut self) -> i16 {
        self.bit_offset = 0;
        if self.offset + 2 <= self.buffer.len() {
            let value = i16::from_be_bytes(
                self.buffer[self.offset..self.offset + 2].try_into().unwrap()
            );
            self.offset += 2;
            value
        } else {
            0
        }
    }

    pub fn write_short(&mut self, value: i16) {
        self.bit_offset = 0;
        self.ensure_capacity(2);
        self.buffer[self.offset..self.offset + 2].copy_from_slice(&value.to_be_bytes());
        self.offset += 2;
    }

    pub fn write_int(&mut self, value: i32) {
        self.bit_offset = 0;
        self.ensure_capacity(4);
        self.buffer[self.offset..self.offset + 4].copy_from_slice(&value.to_be_bytes());
        self.offset += 4;
    }

    pub fn write_string(&mut self, value: Option<&str>) {
        match value {
            Some(s) if s.len() <= 90000 => {
                let bytes = s.as_bytes();
                self.write_int(bytes.len() as i32);
                self.ensure_capacity(bytes.len());
                self.buffer[self.offset..self.offset + bytes.len()].copy_from_slice(bytes);
                self.offset += bytes.len();
            }
            _ => {
                self.write_int(-1);
            }
        }
    }

    pub fn read_string(&mut self) -> String {
        let length = self.read_int();
        if length > 0 && length < 90000 && (self.offset + length as usize) <= self.buffer.len() {
            let string = String::from_utf8(
                self.buffer[self.offset..self.offset + length as usize].to_vec()
            ).unwrap_or_default();
            self.offset += length as usize;
            string
        } else {
            String::new()
        }
    }

    pub fn read_data_reference(&mut self) -> [i32; 2] {
        let a1 = self.read_vint();
        [a1, if a1 == 0 { 0 } else { self.read_vint() }]
    }

    pub fn write_data_reference(&mut self, value1: i32, value2: i32) {
        if value1 < 1 {
            self.write_vint(0);
        } else {
            self.write_vint(value1);
            self.write_vint(value2);
        }
    }

    pub fn write_vint(&mut self, value: i32) {
        self.bit_offset = 0;
        let mut temp = ((value >> 25) & 0x40) as u8;
        let mut flipped = (value ^ (value >> 31)) as u32;
        temp |= (value & 0x3F) as u8;
        let mut value = (value >> 6) as u32;
        flipped >>= 6;

        if flipped == 0 {
            self.write_byte(temp);
            return;
        }

        self.write_byte(temp | 0x80);

        flipped >>= 7;
        let mut r = if flipped != 0 { 0x80 } else { 0 };
        self.write_byte((value & 0x7F) as u8 | r);
        value >>= 7;

        while flipped != 0 {
            flipped >>= 7;
            r = if flipped != 0 { 0x80 } else { 0 };
            self.write_byte((value & 0x7F) as u8 | r);
            value >>= 7;
        }
    }

    pub fn read_vint(&mut self) -> i32 {
        let mut result: u32 = 0;
        let mut shift = 0;
        let mut s: u8;
        let mut a1: u8;
        let mut a2: u8;

        loop {
            if self.offset >= self.buffer.len() {
                return 0;
            }
            let byte = self.buffer[self.offset];
            self.offset += 1;

            if shift == 0 {
                a1 = (byte & 0x40) >> 6;
                a2 = (byte & 0x80) >> 7;
                s = (byte << 1) & !0x81;
                s |= (a2 << 7) | a1;
            } else {
                s = byte;
            }

            result |= ((s & 0x7F) as u32) << shift;
            shift += 7;

            if (s & 0x80) == 0 {
                break;
            }
        }

        ((result >> 1) ^ (-((result & 1) as i32) as u32)) as i32
    }

    pub fn write_boolean(&mut self, value: bool) {
        if self.bit_offset == 0 {
            self.ensure_capacity(1);
            self.buffer[self.offset] = 0;
            self.offset += 1;
        }

        if value {
            self.buffer[self.offset - 1] |= 1 << self.bit_offset;
        }

        self.bit_offset = (self.bit_offset + 1) % 8;
    }

    pub fn read_boolean(&mut self) -> bool {
        self.read_vint() >= 1
    }

    pub fn write_long_long(&mut self, value: i64) {
        self.write_int((value >> 32) as i32);
        self.write_int(value as i32);
    }

    pub fn write_logic_long(&mut self, value1: i32, value2: i32) {
        self.write_vint(value1);
        self.write_vint(value2);
    }

    pub fn read_logic_long(&mut self) -> [i32; 2] {
        [self.read_vint(), self.read_vint()]
    }

    pub fn write_long(&mut self, value1: i32, value2: i32) {
        self.write_int(value1);
        self.write_int(value2);
    }

    pub fn read_long(&mut self) -> [i32; 2] {
        [self.read_int(), self.read_int()]
    }

    pub fn write_byte(&mut self, value: u8) {
        self.bit_offset = 0;
        self.ensure_capacity(1);
        self.buffer[self.offset] = value;
        self.offset += 1;
    }

    pub fn write_bytes(&mut self, buffer: &[u8]) {
        if !buffer.is_empty() {
            self.write_int(buffer.len() as i32);
            self.ensure_capacity(buffer.len());
            self.buffer[self.offset..self.offset + buffer.len()].copy_from_slice(buffer);
            self.offset += buffer.len();
        } else {
            self.write_int(-1);
        }
    }

    pub fn ensure_capacity(&mut self, capacity: usize) {
        if self.offset + capacity > self.buffer.len() {
            self.buffer.resize(self.offset + capacity, 0);
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.offset = 0;
        self.bit_offset = 0;
    }

    pub fn get_length(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_buffer(&self) -> &BytesMut {
        &self.buffer
    }

    pub fn replace_buffer(&mut self, b: BytesMut) -> BytesMut {
        self.offset = 0;
        self.bit_offset = 0;
        std::mem::replace(&mut self.buffer, b)
    }
}