extern crate bytepack;

use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Cursor;
use std::iter::Iterator;
use std::mem::size_of;

use bytepack::{Packed, Unpacker};

pub trait MatchKey: Packed + Hash + Eq + Copy {}

impl MatchKey for u8      {}
impl MatchKey for [u8;2]  {}
impl MatchKey for [u8;3]  {}
impl MatchKey for [u8;4]  {}
impl MatchKey for [u8;5]  {}
impl MatchKey for [u8;6]  {}
impl MatchKey for [u8;7]  {}
impl MatchKey for u16     {}
impl MatchKey for [u16;2] {}
impl MatchKey for [u16;3] {}
impl MatchKey for [u16;4] {}
impl MatchKey for [u16;5] {}
impl MatchKey for [u16;6] {}
impl MatchKey for [u16;7] {}
impl MatchKey for u32     {}
impl MatchKey for [u32;2] {}
impl MatchKey for [u32;3] {}
impl MatchKey for [u32;4] {}
impl MatchKey for [u32;5] {}
impl MatchKey for [u32;6] {}
impl MatchKey for [u32;7] {}
impl MatchKey for u64     {}
impl MatchKey for [u64;2] {}
impl MatchKey for [u64;3] {}
impl MatchKey for [u64;4] {}
impl MatchKey for [u64;5] {}
impl MatchKey for [u64;6] {}
impl MatchKey for [u64;7] {}
impl MatchKey for [u64;8] {}

fn build_map<T: MatchKey>(c: &mut Cursor<&[u8]>) -> HashMap<T,Vec<usize>> {
    let mut map = HashMap::<T, Vec<usize>>::new();
    let size = c.get_ref().len() - size_of::<T>() + 1;
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

#[derive(Clone,Copy,Debug)]
pub struct Match {
    pub first_pos: usize,
    pub second_pos: usize,
    pub length: usize,
}

impl Match {
    pub fn new(first_pos: usize, second_pos: usize, length: usize) -> Match {
        Match {
            first_pos: first_pos,
            second_pos: second_pos,
            length: length,
        }
    }

    pub fn first_end(&self) -> usize {
        self.first_pos + self.length
    }

    pub fn second_end(&self) -> usize {
        self.second_pos + self.length
    }
}

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
    // Terminate the last unique string
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
