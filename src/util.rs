// Copyright (c) 2014, 2015 Robert Clipsham <robert@octarineparrot.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Miscellaneous utilities for low level networking

extern crate libc;

use packet::PrimitiveValues;
use packet::ip::IpNextHeaderProtocol;
use pnet_macros_support::types::u16be;

use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::slice;
use std::str::FromStr;
use std::u8;

/// A MAC address
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct MacAddr(pub u8, pub u8, pub u8, pub u8, pub u8, pub u8);

impl MacAddr {
    /// Construct a new MacAddr
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> MacAddr {
        MacAddr(a, b, c, d, e, f)
    }
}

impl PrimitiveValues for MacAddr {
    type T = (u8, u8, u8, u8, u8, u8);
    fn to_primitive_values(&self) -> (u8, u8, u8, u8, u8, u8) {
        (self.0, self.1, self.2, self.3, self.4, self.5)
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt,
               "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
               self.0,
               self.1,
               self.2,
               self.3,
               self.4,
               self.5)
    }
}

impl fmt::Debug for MacAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

// FIXME Is this the right way to do this? Which occurs is an implementation
//       issue rather than actually defined - is it useful to provide these
//       errors, or would it be better to just give ()?
/// Represents an error which occurred whilst parsing a MAC address
#[derive(Copy, Debug, PartialEq, Eq, Clone)]
pub enum ParseMacAddrErr {
    /// The MAC address has too many components, eg. 00:11:22:33:44:55:66
    TooManyComponents,
    /// The MAC address has too few components, eg. 00:11
    TooFewComponents,
    /// One of the components contains an invalid value, eg. 00:GG:22:33:44:55
    InvalidComponent,
}

impl FromStr for MacAddr {
    type Err = ParseMacAddrErr;
    fn from_str(s: &str) -> Result<MacAddr, ParseMacAddrErr> {
        let mut parts = [0u8; 6];
        let splits = s.split(':');
        let mut i = 0;
        for split in splits {
            if i == 6 {
                return Err(ParseMacAddrErr::TooManyComponents);
            }
            match u8::from_str_radix(split, 16) {
                Ok(b) if split.len() != 0 => parts[i] = b,
                _ => return Err(ParseMacAddrErr::InvalidComponent),
            }
            i += 1;
        }

        if i == 6 {
            Ok(MacAddr(parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]))
        } else {
            Err(ParseMacAddrErr::TooFewComponents)
        }
    }
}

#[test]
fn mac_addr_from_str() {
    assert_eq!("00:00:00:00:00:00".parse(), Ok(MacAddr(0, 0, 0, 0, 0, 0)));
    assert_eq!("ff:ff:ff:ff:ff:ff".parse(),
               Ok(MacAddr(0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF)));
    assert_eq!("12:34:56:78:90:ab".parse(),
               Ok(MacAddr(0x12, 0x34, 0x56, 0x78, 0x90, 0xAB)));
    assert_eq!("::::::".parse::<MacAddr>(),
               Err(ParseMacAddrErr::InvalidComponent));
    assert_eq!("0::::::".parse::<MacAddr>(),
               Err(ParseMacAddrErr::InvalidComponent));
    assert_eq!("::::0::".parse::<MacAddr>(),
               Err(ParseMacAddrErr::InvalidComponent));
    assert_eq!("12:34:56:78".parse::<MacAddr>(),
               Err(ParseMacAddrErr::TooFewComponents));
    assert_eq!("12:34:56:78:".parse::<MacAddr>(),
               Err(ParseMacAddrErr::InvalidComponent));
    assert_eq!("12:34:56:78:90".parse::<MacAddr>(),
               Err(ParseMacAddrErr::TooFewComponents));
    assert_eq!("12:34:56:78:90:".parse::<MacAddr>(),
               Err(ParseMacAddrErr::InvalidComponent));
    assert_eq!("12:34:56:78:90:00:00".parse::<MacAddr>(),
               Err(ParseMacAddrErr::TooManyComponents));
    assert_eq!("xx:xx:xx:xx:xx:xx".parse::<MacAddr>(),
               Err(ParseMacAddrErr::InvalidComponent));
}

#[test]
fn str_from_mac_addr() {
    assert_eq!(format!("{}", MacAddr(0, 0, 0, 0, 0, 0)),
               "00:00:00:00:00:00");
    assert_eq!(format!("{}", MacAddr(0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF)),
               "ff:ff:ff:ff:ff:ff");
    assert_eq!(format!("{}", MacAddr(0x12, 0x34, 0x56, 0x78, 0x09, 0xAB)),
               "12:34:56:78:09:ab");
}

/// Convert value to byte array
pub trait Octets {
    /// Output type - bytes array
    type Output;

    /// Return value as bytes (big-endian order)
    fn octets(&self) -> Self::Output;
}

impl Octets for u64 {
    type Output = [u8; 8];

    fn octets(&self) -> Self::Output {
        [(*self >> 56) as u8,
         (*self >> 48) as u8,
         (*self >> 40) as u8,
         (*self >> 32) as u8,
         (*self >> 24) as u8,
         (*self >> 16) as u8,
         (*self >> 8) as u8,
         *self as u8]
    }
}

impl Octets for u32 {
    type Output = [u8; 4];

    fn octets(&self) -> Self::Output {
        [(*self >> 24) as u8, (*self >> 16) as u8, (*self >> 8) as u8, *self as u8]
    }
}

impl Octets for u16 {
    type Output = [u8; 2];

    fn octets(&self) -> Self::Output {
        [(*self >> 8) as u8, *self as u8]
    }
}

impl Octets for u8 {
    type Output = [u8; 1];

    fn octets(&self) -> Self::Output {
        [*self]
    }
}

/// Calculates a checksum. Used by ipv4 and icmp. The two bytes starting at `skipword * 2` will be
/// ignored. Supposed to be the checksum field, which is regarded as zero during calculation.
pub fn checksum(data: &[u8], skipword: usize) -> u16be {
    let sum = sum_be_words(data, skipword);
    finalize_checksum(sum)
}

fn finalize_checksum(mut sum: u32) -> u16be {
    while sum >> 16 != 0 {
        sum = (sum >> 16) + (sum & 0xFFFF);
    }
    !sum as u16
}

/// Calculate the checksum for a packet built on IPv4. Used by udp and tcp.
pub fn ipv4_checksum(data: &[u8],
                     skipword: usize,
                     extra_data: &[u8],
                     source: Ipv4Addr,
                     destination: Ipv4Addr,
                     next_level_protocol: IpNextHeaderProtocol)
    -> u16be {
    let mut sum = 0u32;

    // Checksum pseudo-header
    sum += ipv4_word_sum(source);
    sum += ipv4_word_sum(destination);

    let IpNextHeaderProtocol(next_level_protocol) = next_level_protocol;
    sum += next_level_protocol as u32;

    let len = data.len() + extra_data.len();
    sum += len as u32;

    // Checksum packet header and data
    sum += sum_be_words(data, skipword);
    sum += sum_be_words(extra_data, extra_data.len() / 2);

    finalize_checksum(sum)
}

fn ipv4_word_sum(ip: Ipv4Addr) -> u32 {
    let octets = ip.octets();
    ((octets[0] as u32) << 8 | octets[1] as u32) + ((octets[2] as u32) << 8 | octets[3] as u32)
}

/// Calculate the checksum for a packet built on IPv6
pub fn ipv6_checksum(data: &[u8],
                     skipword: usize,
                     extra_data: &[u8],
                     source: Ipv6Addr,
                     destination: Ipv6Addr,
                     next_level_protocol: IpNextHeaderProtocol)
    -> u16be {
    let mut sum = 0u32;

    // Checksum pseudo-header
    sum += ipv6_word_sum(source);
    sum += ipv6_word_sum(destination);

    let IpNextHeaderProtocol(next_level_protocol) = next_level_protocol;
    sum += next_level_protocol as u32;

    let len = data.len() + extra_data.len();
    sum += len as u32;

    // Checksum packet header and data
    sum += sum_be_words(data, skipword);
    sum += sum_be_words(extra_data, extra_data.len() / 2);

    finalize_checksum(sum)
}

fn ipv6_word_sum(ip: Ipv6Addr) -> u32 {
    ip.segments().iter().map(|x| *x as u32).sum()
}

/// Sum all words (16 bit chunks) in the given data. The word at word offset
/// `skipword` will be skipped. Each word is treated as big endian.
fn sum_be_words(data: &[u8], skipword: usize) -> u32 {
    let len = data.len();
    let wdata: &[u16] = unsafe { slice::from_raw_parts(data.as_ptr() as *const u16, len / 2) };
    assert!(skipword <= wdata.len());

    let mut sum = 0u32;
    let mut i = 0;
    while i < skipword {
        sum += u16::from_be(unsafe { *wdata.get_unchecked(i) }) as u32;
        i += 1;
    }
    i += 1;
    while i < wdata.len() {
        sum += u16::from_be(unsafe { *wdata.get_unchecked(i) }) as u32;
        i += 1;
    }
    // If the length is odd, make sure to checksum the final byte
    if len & 1 != 0 {
        sum += (unsafe { *data.get_unchecked(len - 1) } as u32) << 8;
    }

    sum
}

#[cfg(all(test, feature = "benchmark"))]
mod checksum_benchmarks {
    use super::checksum;
    use test::{Bencher, black_box};

    #[bench]
    fn bench_checksum_small(b: &mut Bencher) {
        let data = vec![99u8; 20];
        b.iter(|| checksum(black_box(&data), 5));
    }

    #[bench]
    fn bench_checksum_large(b: &mut Bencher) {
        let data = vec![123u8; 1024];
        b.iter(|| checksum(black_box(&data), 5));
    }
}
