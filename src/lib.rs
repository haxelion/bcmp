//! `bcmp` is a simple crate which offers data comparison mechanisms which go beyond the simple 
//! equality. It only operates on byte slices, hence its name, and relies on efficiently finding 
//! common substrings between two blob of data. The implementation relies on two different linear 
//! time algorithms: a `HashMap` based algorithm called [`HashMatch`](hashmatch/index.html) and 
//! a suffix tree built using Ukkonen algorithm called [`TreeMatch`](treematch/index.html).
//!
//! # Examples
//! 
//! Iterate over the matches between two strings using [`HashMatch`](hashmatch/index.html) with a 
//! minimum match length of 2 bytes:
//!
//! ```
//! use bcmp::{AlgoSpec, MatchIterator};
//!
//! let a = "abcdefg";
//! let b = "012abc34cdef56efg78abcdefg";
//! let match_iter = MatchIterator::new(a.as_bytes(), b.as_bytes(), AlgoSpec::HashMatch(2));
//! for m in match_iter {
//!     println!("Match: {:}", &a[m.first_pos..m.first_end()]);
//! }
//! ```
//!
//! Construct a patch set to build the file `b` from the file `a` using [`TreeMatch`](treematch/index.html) 
//! with a minimum match length of 4 bytes:
//! 
//! ```no_run
//! use std::fs::File;
//! use std::io::Read;
//! 
//! use bcmp::{AlgoSpec, patch_set};
//! 
//! let mut a = Vec::<u8>::new();
//! let mut b = Vec::<u8>::new();
//! File::open("a").unwrap().read_to_end(&mut a);
//! File::open("b").unwrap().read_to_end(&mut b);
//!
//! let ps = patch_set(&a, &b, AlgoSpec::TreeMatch(4));
//! for patch in ps {
//!     println!("b[0x{:x}..0x{:x}] == a[0x{:x}..0x{:x}]", patch.second_pos, patch.second_end(), patch.first_pos, patch.first_end());
//! }
//! ```

extern crate bytepack;

pub mod hashmatch;
pub mod treematch;
#[cfg(test)]
mod tests;

use std::iter::Iterator;

use hashmatch::HashMatchIterator;
use treematch::TreeMatchIterator;

/// A structure representing a matching substring between two pieces of data.
#[derive(Clone,Copy,Debug,PartialEq, Eq)]
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

/// An enumeration describing the algorithm specification: either [`HashMatch`](hashmatch/index.html) 
/// or [`TreeMatch`](treematch/index.html) with the minimal matching length parameter.
#[derive(Clone,Copy,Debug)]
pub enum AlgoSpec {
    /// The parameter is the minimal matching length which will determine the 
    /// [`HashMatchKey`](hashmatch/trait.HashMatchKey.html) used.
    HashMatch(usize),
    /// The parameter is the minimal matching length.
    TreeMatch(usize)
}

/// A generic wrapper for [`HashMatchIterator`](hashmatch/struct.HashMatchIterator.html) and 
/// [`TreeMatchIterator`](treematch/struct.TreeMatchIterator.html).
///
/// Both algorithms will return the same matches but the exact order may vary. 
/// The only ordering guarantee is that the [`Match`](struct.Match.html) will be returned in 
/// ascending order of the [`second_pos`](struct.Match.html#second_pos.v) field.
///
/// For efficiency reasons, submatches are never returned. This means if we iterate over the 
/// [`Match`](struct.Match.html) of `"abcd"` and `"012abcd34"`, only `"abcd"` is returned. The 
/// submatches `"abc"`, `"bcd"`, `"ab"`, ... are never returned but can easily be computed from the 
/// encompassing [`Match`](struct.Match.html).
pub struct MatchIterator<'a> {
    iter: Box<Iterator<Item=Match> + 'a>
}

impl<'a> MatchIterator<'a> {
    /// Build a new `MatchIterator` from two pieces of data to compare and an [`AlgoSpec`](enum.AlgoSpec.html).
    ///
    /// # Panics
    ///
    /// It will panic if the [`AlgoSpec`](enum.AlgoSpec.html) is not supported. 
    /// [`TreeMatch`](treematch/index.html) supports any minimum matching length but 
    /// [`HashMatch`](hashmatch/index.html) only supports length of 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 
    /// 14, 16, 20, 24, 28, 32, 40, 48, 56 and 64 bytes.
    pub fn new(first: &'a [u8], second: &'a [u8], algo_spec: AlgoSpec) -> MatchIterator<'a> {
        MatchIterator {
            iter: match algo_spec {
                AlgoSpec::TreeMatch(mml) => Box::new(TreeMatchIterator::new(first, second, mml)),
                AlgoSpec::HashMatch(1) => Box::new(HashMatchIterator::<u8>::new(first, second)),
                AlgoSpec::HashMatch(2) => Box::new(HashMatchIterator::<u16>::new(first, second)),
                AlgoSpec::HashMatch(3) => Box::new(HashMatchIterator::<[u8;3]>::new(first, second)),
                AlgoSpec::HashMatch(4) => Box::new(HashMatchIterator::<u32>::new(first, second)),
                AlgoSpec::HashMatch(5) => Box::new(HashMatchIterator::<[u8;5]>::new(first, second)),
                AlgoSpec::HashMatch(6) => Box::new(HashMatchIterator::<[u16;3]>::new(first, second)),
                AlgoSpec::HashMatch(7) => Box::new(HashMatchIterator::<[u8;7]>::new(first, second)),
                AlgoSpec::HashMatch(8) => Box::new(HashMatchIterator::<u64>::new(first, second)),
                AlgoSpec::HashMatch(10) => Box::new(HashMatchIterator::<[u16;5]>::new(first, second)),
                AlgoSpec::HashMatch(12) => Box::new(HashMatchIterator::<[u32;3]>::new(first, second)),
                AlgoSpec::HashMatch(14) => Box::new(HashMatchIterator::<[u16;7]>::new(first, second)),
                AlgoSpec::HashMatch(16) => Box::new(HashMatchIterator::<[u64;2]>::new(first, second)),
                AlgoSpec::HashMatch(20) => Box::new(HashMatchIterator::<[u32;5]>::new(first, second)),
                AlgoSpec::HashMatch(24) => Box::new(HashMatchIterator::<[u64;3]>::new(first, second)),
                AlgoSpec::HashMatch(28) => Box::new(HashMatchIterator::<[u32;7]>::new(first, second)),
                AlgoSpec::HashMatch(32) => Box::new(HashMatchIterator::<[u64;4]>::new(first, second)),
                AlgoSpec::HashMatch(40) => Box::new(HashMatchIterator::<[u64;5]>::new(first, second)),
                AlgoSpec::HashMatch(48) => Box::new(HashMatchIterator::<[u64;6]>::new(first, second)),
                AlgoSpec::HashMatch(56) => Box::new(HashMatchIterator::<[u64;7]>::new(first, second)),
                AlgoSpec::HashMatch(64) => Box::new(HashMatchIterator::<[u64;8]>::new(first, second)),
                _ => panic!("Unsupported AlgoSpec")
            }
        }
    }
}

impl<'a> Iterator for MatchIterator<'a> {
    type Item = Match;
    #[inline]
    fn next(&mut self) -> Option<Match> {
        self.iter.next()
    }
}

/// Return the longest common substring between two byte slices.
pub fn longest_common_substring(first: &[u8], second: &[u8], algo_spec: AlgoSpec) -> Match {
    let mut longest = Match::new(0,0,0);
    let match_iter = MatchIterator::new(first, second, algo_spec);
    for m in match_iter {
        if m.length > longest.length {
            longest = m;
        }
    }
    return longest;
}

/// Return the `N` longest common substrings between two byte slices. The vector is sorted in 
/// decreasing order of  [`Match`](struct.Match.html) length.
pub fn longest_common_substrings(first: &[u8], second: &[u8], algo_spec: AlgoSpec, number: usize) -> Vec<Match> {
    let match_iter = MatchIterator::new(first, second, algo_spec);
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
/// 
/// The returned set might be incomplete if some part of the second byte slice could not be found 
/// in the first. The result is highly dependent on the minimal matching length chosen.
pub fn patch_set(first: &[u8], second: &[u8], algo_spec: AlgoSpec) -> Vec<Match> {
    let mut match_iter = MatchIterator::new(first, second, algo_spec);
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
/// The [`AlgoSpec`](enum.AlgoSpec.html) highly influence the result because it determines the 
/// minimal length of a match. The longer is the minimal length of a match, the more 
/// unique strings will be found.
pub fn unique_strings(first: &[u8], second: &[u8], algo_spec: AlgoSpec) -> Vec<(usize,usize)> {
    let match_iter = MatchIterator::new(first, second, algo_spec);
    let mut uniques = Vec::<(usize,usize)>::new();
    let mut covered = 0;

    for m in match_iter {
        // There is a lapse in the second file coverage, add a unique string
        if m.second_pos > covered {
            uniques.push((covered, m.second_pos));
        }
        // If more of the file is covered, extend the coverage
        if m.second_end() > covered {
            covered = m.second_end();
        }
    }
    if covered < second.len() {
        uniques.push((covered, second.len()));
    }

    return uniques;
}
