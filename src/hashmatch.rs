//! HashMatch is a binary matching algorithm based on a `HashMap` to retrieve the begining of 
//! matching strings.
//!
//! It relies on using a [`HashMatchKey`](trait.HashMatchKey.html) long enough to
//! weed out "random" matches to obtain linear time performances. This 
//! [`HashMatchKey`](trait.HashMatchKey.html) offers a tradeoff between the speed and the minimal 
//! matching length.

use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Cursor;
use std::iter::Iterator;
use std::mem::size_of;

use bytepack::{Packed, Unpacker};

use Match;

/// Trait marking types which can be used as a matching key in the `HashMap`.
///
/// The larger the `HashMatchKey` is, the faster the implementation will be but the minimal matching 
/// length will proportionally increase. For example a `u32` `HashMatchKey` allows to find common 
/// substring equal or longer than 4 bytes while a `u64` `HashMatchKey` will be significantly faster 
/// but only allows to find common substring equal or longer than 8 bytes.
pub trait HashMatchKey: Packed + Hash + Eq + Copy {}

impl HashMatchKey for u8      {}
impl HashMatchKey for [u8;2]  {}
impl HashMatchKey for [u8;3]  {}
impl HashMatchKey for [u8;4]  {}
impl HashMatchKey for [u8;5]  {}
impl HashMatchKey for [u8;6]  {}
impl HashMatchKey for [u8;7]  {}
impl HashMatchKey for [u8;8]  {}
impl HashMatchKey for u16     {}
impl HashMatchKey for [u16;2] {}
impl HashMatchKey for [u16;3] {}
impl HashMatchKey for [u16;4] {}
impl HashMatchKey for [u16;5] {}
impl HashMatchKey for [u16;6] {}
impl HashMatchKey for [u16;7] {}
impl HashMatchKey for [u16;8] {}
impl HashMatchKey for u32     {}
impl HashMatchKey for [u32;2] {}
impl HashMatchKey for [u32;3] {}
impl HashMatchKey for [u32;4] {}
impl HashMatchKey for [u32;5] {}
impl HashMatchKey for [u32;6] {}
impl HashMatchKey for [u32;7] {}
impl HashMatchKey for [u32;8] {}
impl HashMatchKey for u64     {}
impl HashMatchKey for [u64;2] {}
impl HashMatchKey for [u64;3] {}
impl HashMatchKey for [u64;4] {}
impl HashMatchKey for [u64;5] {}
impl HashMatchKey for [u64;6] {}
impl HashMatchKey for [u64;7] {}
impl HashMatchKey for [u64;8] {}

fn build_map<T: HashMatchKey>(c: &mut Cursor<&[u8]>) -> HashMap<T,Vec<usize>> {
    let size = c.get_ref().len() - size_of::<T>() + 1;
    let mut map = HashMap::<T, Vec<usize>>::with_capacity(size);
    for i in 0..size {
        c.set_position(i as u64);
        let v = c.unpack::<T>().unwrap();
        if !map.contains_key(&v) {
            map.insert(v, Vec::<usize>::new());
        }
        map.get_mut(&v).unwrap().push(i);
    }
    return map;
}

/// An iterator over all the [`Match`](../struct.Match.html) bewteen two pieces of data.
///
/// # Examples
/// 
/// ```
/// use bcmp::hashmatch::HashMatchIterator;
///
/// let a = "abcdefg";
/// let b = "012abc34cdef56efg78abcdefg";
/// let match_iter = HashMatchIterator::<u16>::new(a.as_bytes(), b.as_bytes());
/// for m in match_iter {
///     println!("Match: {:}", &a[m.first_pos..m.first_end()]);
/// }
/// ```
pub struct HashMatchIterator<'a, T: HashMatchKey> {
    first: Cursor<&'a [u8]>,
    second: Cursor<&'a [u8]>,
    second_len: usize,
    i: usize,
    j: usize,
    map: HashMap<T,Vec<usize>>,
    matched: HashMap<isize, usize>
}

impl<'a, T: HashMatchKey> HashMatchIterator<'a, T> {
    /// Allocate a new iterator over the matches between two byte slices
    pub fn new(first: &'a [u8], second: &'a [u8]) -> HashMatchIterator<'a, T> {
        let second_len = second.len() - size_of::<T>() + 1;
        let mut first_cursor = Cursor::new(first);
        let second_cursor = Cursor::new(second);
        let map = build_map(&mut first_cursor);
        HashMatchIterator {
            first: first_cursor,
            second: second_cursor,
            second_len: second_len,
            i: 0,
            j: 0,
            map: map,
            matched: HashMap::new()
        }
    }
    /// Reset the iterator to its start. This allows to iterate multiple times over the matches 
    /// without wasting time regenerating the `HashMap`.
    pub fn reset(&mut self) {
        self.i = 0;
        self.j = 0;
        self.matched.clear();
    }
}

impl<'a, T: HashMatchKey> Iterator for HashMatchIterator<'a, T> {
    type Item = Match;
    fn next(&mut self) -> Option<Match> {
        while self.j < self.second_len {
            self.second.set_position(self.j as u64);
            let v = self.second.unpack::<T>().unwrap();
            if let Some(positions) = self.map.get(&v) {
                while self.i < positions.len() {
                    let first_pos = positions[self.i];
                    self.i += 1;
                    // Check if this is a not part of a match already returned
                    let delta = first_pos as isize - self.j as isize;
                    if !(self.matched.contains_key(&delta) && self.matched.get(&delta).unwrap() >= &self.j) {
                        let first_data = self.first.get_ref();
                        let second_data = self.second.get_ref();
                        // Compute match length
                        let mut idx = 0;
                        while (first_pos + idx) < first_data.len() && 
                              (self.j + idx) < second_data.len() &&
                              first_data[first_pos + idx] == second_data[self.j + idx] {
                            idx += 1;
                        }
                        // Update matched
                        self.matched.insert(delta, self.j + idx);
                        return Some(Match::new(first_pos, self.j, idx));
                    }
                }
            }
            self.j += 1;
            self.i = 0;
        }
        return None;
    }
}

/// Find the list of unique strings from the second byte slice which can't be found in the first.
/// 
/// The [`HashMatchKey`](trait.HashMatchKey.html) highly influence the result because it determines the 
/// minimal length of a common string. The longer is the [`HashMatchKey`](trait.HashMatchKey.html), the more 
/// unique strings will be found.
pub fn unique_strings<T: HashMatchKey>(first: &[u8], second: &[u8]) -> Vec<(usize,usize)> {
    let mut first_cursor = Cursor::new(first);
    let mut second_cursor = Cursor::new(second);
    let map = build_map::<T>(&mut first_cursor);
    let second_len = second.len() - size_of::<T>() + 1;

    let mut uniques = Vec::<(usize,usize)>::new();
    let mut current : Option<(usize,usize)> = None;

    for i in 0..second_len {
        second_cursor.set_position(i as u64);
        let v = second_cursor.unpack::<T>().unwrap();
        // v exists in first: terminate an existing unique string
        if map.contains_key(&v) {
            if let Some(mut unique) = current.take() {
                // Eliminate the aliasing from left
                if unique.0 > 0 {
                    unique.0 += size_of::<T>() - 1;
                }
                // Append if non-empty
                if unique.0 < unique.1 {
                    uniques.push(unique);
                }
            }
        }
        // v doesn't exist in first: start or extend a unique string
        else {
            current = match current {
                Some((start, _)) => Some((start, i + 1)),
                None => Some((i, i + 1))
            };
        }
    }
    // Terminate the remaining unique string
    if let Some(mut unique) = current.take() {
        // Eliminate the aliasing from left
        if unique.0 > 0 {
            unique.0 += size_of::<T>() - 1;
        }
        // Also has aliasing from the right because it is the last
        unique.1 += size_of::<T>() - 1;
        // Append if non-empty
        if unique.0 < unique.1 {
            uniques.push(unique);
        }
    }

    return uniques;
}
