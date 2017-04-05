//! `bcmp` is a simple crate which offers data comparison mechanisms which go beyond the simple 
//! equality. It only operates on byte slices, hence its name, and relies on efficiently finding 
//! common substrings between two blob of data. This is implemented using `HashMap` which should 
//! offer linear time operation provided the [`MatchKey`](trait.MatchKey.html) is large enough.

extern crate bytepack;

use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Cursor;
use std::iter::Iterator;
use std::mem::size_of;

use bytepack::{Packed, Unpacker};

/// Trait marking types which can be used as a matching key in the `HashMap`
///
/// The larger the `MatchKey` is, the faster the implementation will be but 
/// the memory consumption and minimum common substring length will proportionally increase.
/// For example a `u32` `MatchKey` allows to find common substring equal or longer than 4 bytes 
/// while a `u64` `MatchKey` will be significantly faster but consumes twice the memory and only 
/// allows to find common substring equal or longer than 8 bytes.
pub trait MatchKey: Packed + Hash + Eq + Copy {}

impl MatchKey for u8      {}
impl MatchKey for [u8;2]  {}
impl MatchKey for [u8;3]  {}
impl MatchKey for [u8;4]  {}
impl MatchKey for [u8;5]  {}
impl MatchKey for [u8;6]  {}
impl MatchKey for [u8;7]  {}
impl MatchKey for [u8;8]  {}
impl MatchKey for u16     {}
impl MatchKey for [u16;2] {}
impl MatchKey for [u16;3] {}
impl MatchKey for [u16;4] {}
impl MatchKey for [u16;5] {}
impl MatchKey for [u16;6] {}
impl MatchKey for [u16;7] {}
impl MatchKey for [u16;8] {}
impl MatchKey for u32     {}
impl MatchKey for [u32;2] {}
impl MatchKey for [u32;3] {}
impl MatchKey for [u32;4] {}
impl MatchKey for [u32;5] {}
impl MatchKey for [u32;6] {}
impl MatchKey for [u32;7] {}
impl MatchKey for [u32;8] {}
impl MatchKey for u64     {}
impl MatchKey for [u64;2] {}
impl MatchKey for [u64;3] {}
impl MatchKey for [u64;4] {}
impl MatchKey for [u64;5] {}
impl MatchKey for [u64;6] {}
impl MatchKey for [u64;7] {}
impl MatchKey for [u64;8] {}

fn build_map<T: MatchKey>(c: &mut Cursor<&[u8]>) -> HashMap<T,Vec<usize>> {
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

/// A structure representing a matching substring between two pieces of data.
#[derive(Clone,Copy,Debug)]
pub struct Match {
    /// Start of the string in the first piece of data.
    pub first_pos: usize,
    /// Start of the string in the second piece of data.
    pub second_pos: usize,
    /// Length of the string.
    pub length: usize,
}

impl Match {
    /// Allocate a new `Match`.
    pub fn new(first_pos: usize, second_pos: usize, length: usize) -> Match {
        Match {
            first_pos: first_pos,
            second_pos: second_pos,
            length: length,
        }
    }
    /// `first_pos + length`
    pub fn first_end(&self) -> usize {
        self.first_pos + self.length
    }
    /// `second_pos + length`
    pub fn second_end(&self) -> usize {
        self.second_pos + self.length
    }
}

/// An iterator over all the [`Match`](struct.Match.html) bewteen two pieces of data.
///
/// The [`Match`](struct.Match.html) are returned in the order of the second file data. This means  
/// the [`second_pos`](struct.Match.html#second_pos.v) of the next [`Match`](struct.Match.html) is 
/// always equal or greater than the previous [`Match`](struct.Match.html).
///
/// For efficiency reasons, submatches are never returned. This means if we iterate over the 
/// [`Match`](struct.Match.html) of `"abcd"` and `"012abcd34"`, only `"abcd"` is returned. The 
/// submatches `"abc"`, `"bcd"`, `"ab"`, ... are not returned but can easily be computed.
///
/// # Examples
/// 
/// ```
/// use bcmp::MatchIterator;
///
/// let a = "abcdefg";
/// let b = "012abc34cdef56efg78abcdefg";
/// let match_iter = MatchIterator::<u16>::new(a.as_bytes(), b.as_bytes());
/// for m in match_iter {
///     println!("Match: {:}", &a[m.first_pos..m.first_end()]);
/// }
/// ```
pub struct MatchIterator<'a, T: MatchKey> {
    first: Cursor<&'a [u8]>,
    second: Cursor<&'a [u8]>,
    second_len: usize,
    i: usize,
    j: usize,
    map: HashMap<T,Vec<usize>>,
    matched: HashMap<isize, usize>
}

impl<'a, T: MatchKey> MatchIterator<'a, T> {
    /// Allocate a new iterator over the matches between two byte slices
    pub fn new(first: &'a [u8], second: &'a [u8]) -> MatchIterator<'a, T> {
        let second_len = second.len() - size_of::<T>() + 1;
        let mut first_cursor = Cursor::new(first);
        let second_cursor = Cursor::new(second);
        let map = build_map(&mut first_cursor);
        MatchIterator {
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

impl<'a, T: MatchKey> Iterator for MatchIterator<'a, T> {
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
                    if !(self.matched.contains_key(&delta) && self.matched.get(&delta).unwrap() > &self.j) {
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

/// Return the longest common substring between two byte slices.
pub fn longest_common_substring<T: MatchKey>(first: &[u8], second: &[u8]) -> Match {
    let mut longest = Match::new(0,0,0);
    let match_iter = MatchIterator::<T>::new(first, second);
    for m in match_iter {
        if m.length > longest.length {
            longest = m;
        }
    }
    return longest;
}

/// Return the `N` longest common substrings between two byte slices. The vector is sorted in 
/// decreasing order of  [`Match`](struct.Match.html) length.
pub fn longest_common_substrings<T: MatchKey>(first: &[u8], second: &[u8], number: usize) -> Vec<Match> {
    let match_iter = MatchIterator::<T>::new(first, second);
    // Number +1 to avoid realocation when inserting
    let mut top = Vec::<Match>::with_capacity(number + 1);
    let mut threshold = 0;

    for m in match_iter {
        if m.length > threshold {
            // Find an insertion position
            let mut insert_pos = 0;
            while insert_pos < top.len() && top[insert_pos].length > m.length {
                insert_pos += 1;
            }
            top.insert(insert_pos, m);
            if top.len() > number {
                top.truncate(number);
                threshold = top.last().unwrap().length;
            }
        }
    }

    return top;
}

/// Identify the smallest set of patches needed the build the second byte slice from the first.
/// The returned set might be incomplete if some part of the second byte slice could not be found 
/// in the first.
///
/// The result is highly dependent on the [`MatchKey`](trait.MatchKey.html) chosen. For example a 
/// `u32` [`MatchKey`](trait.MatchKey.html) might cause holes of four bytes or less.
pub fn patch_set<T: MatchKey>(first: &[u8], second: &[u8]) -> Vec<Match> {
    let mut match_iter = MatchIterator::<T>::new(first, second);
    let mut patches = Vec::<Match>::new();
    // Always push first patch
    if let Some(m) = match_iter.next() {
        patches.push(m);
    }
    for mut m in match_iter {
        // Determine how the new match fit in the patch set.
        let last = patches.len() - 1;
        // If it covers more of the second file it is interesting.
        if m.second_end() > patches[last].second_end() {
            // If it's just better than the last patch then replace it
            if m.second_pos == patches[last].second_pos {
                patches[last] = m;
            }
            // If it encompasses the last patch, truncate it and replace it
            else if m.second_pos < patches[last].second_pos {
                let overlap = patches[last].second_pos - m.second_pos;
                m.first_pos += overlap;
                m.second_pos += overlap;
                m.length -= overlap;
                patches[last] = m;
            }
            // If it's overlaping, append it but shorten it (because of the enumeration algorithm,
            // this makes it possible to replace it by another overlaping patch
            else if m.second_pos > patches[last].second_pos && m.second_pos < patches[last].second_end() {
                let overlap = patches[last].second_end() - m.second_pos;
                m.first_pos += overlap;
                m.second_pos += overlap;
                m.length -= overlap;
                patches.push(m);
            }
            // Else just append it.
            else {
                patches.push(m);
            }
        }
    }
    return patches;
}

/// Find the list of unique strings from the second byte slice which can't be found in the first.
/// 
/// The [`MatchKey`](trait.MatchKey.html) highly influence the result because it determines the 
/// minimal length of a common string. The longer is the [`MatchKey`](trait.MatchKey.html), the more 
/// unique strings will be found.
pub fn unique_strings<T: MatchKey>(first: &[u8], second: &[u8]) -> Vec<(usize,usize)> {
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

#[cfg(test)]
mod tests;
