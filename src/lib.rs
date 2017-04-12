//! `bcmp` is a simple crate which offers data comparison mechanisms which go beyond the simple 
//! equality. It only operates on byte slices, hence its name, and relies on efficiently finding 
//! common substrings between two blob of data. This is implemented using `HashMap` which should 
//! offer linear time operation provided the [`MatchKey`](trait.MatchKey.html) is large enough.

extern crate bytepack;

pub mod hashmatch;
pub mod ukkonen;
#[cfg(test)]
mod tests;

use hashmatch::{HashMatchKey, HashMatchIterator};

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

pub trait MatchIterator<'a> {
    fn new(first: &'a[u8], second: &'a[u8]) -> Self;
    fn reset(&mut self);
}

/// Return the longest common substring between two byte slices.
pub fn longest_common_substring<T: HashMatchKey>(first: &[u8], second: &[u8]) -> Match {
    let mut longest = Match::new(0,0,0);
    let match_iter = HashMatchIterator::<T>::new(first, second);
    for m in match_iter {
        if m.length > longest.length {
            longest = m;
        }
    }
    return longest;
}

/// Return the `N` longest common substrings between two byte slices. The vector is sorted in 
/// decreasing order of  [`Match`](struct.Match.html) length.
pub fn longest_common_substrings<T: HashMatchKey>(first: &[u8], second: &[u8], number: usize) -> Vec<Match> {
    let match_iter = HashMatchIterator::<T>::new(first, second);
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
/// The result is highly dependent on the [`HashMatchKey`](trait.HashMatchKey.html) chosen. For example a 
/// `u32` [`HashMatchKey`](trait.HashMatchKey.html) might cause holes of four bytes or less.
pub fn patch_set<T: HashMatchKey>(first: &[u8], second: &[u8]) -> Vec<Match> {
    let mut match_iter = HashMatchIterator::<T>::new(first, second);
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
