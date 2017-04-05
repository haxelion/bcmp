bcmp
====

[![Crates.io](https://img.shields.io/crates/v/bcmp.svg)](https://crates.io/crates/bcmp)
[![Build Status](https://travis-ci.org/haxelion/bcmp.svg?branch=master)](https://travis-ci.org/haxelion/bcmp)
[![Docs.rs](https://docs.rs/bcmp/badge.svg)](https://docs.rs/bcmp)

`bcmp` is a simple crate which offers data comparison mechanisms which go beyond the simple 
equality. It only operates on byte slices, hence its name, and relies on efficiently finding 
common substrings between two blob of data. This is implemented using `HashMap` which should 
offer linear time operation provided the `MatchKey` is large enough.


Example
-------

Iterating over the common substrings of two strings:

``` rust
extern crate bcmp;
use bcmp::MatchIterator;

fn main() {
    let a = "abcdefg";
    let b = "012abc34cdef56efg78abcdefg";
    let match_iter = MatchIterator::<u16>::new(a.as_bytes(), b.as_bytes());

    for m in match_iter {
        println!("Match: {:}", &a[m.first_pos..m.first_end()]);
    }
}
```
