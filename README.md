bcmp
====

[![Crates.io](https://img.shields.io/crates/v/bcmp.svg)](https://crates.io/crates/bcmp)
[![Build Status](https://travis-ci.org/haxelion/bcmp.svg?branch=master)](https://travis-ci.org/haxelion/bcmp)
[![Docs.rs](https://docs.rs/bcmp/badge.svg)](https://docs.rs/bcmp)

`bcmp` is a simple crate which offers data comparison mechanisms which go beyond the simple 
equality. It only operates on byte slices, hence its name, and relies on efficiently finding 
 common substrings between two blob of data. The implementation relies on two different linear 
 time algorithms: a `HashMap` based algorithm called `HashMatch` and 
 a suffix tree built using Ukkonen algorithm called `TreeMatch`.


Example
-------

Iterating over the common substrings of two strings:

``` rust
extern crate bcmp;
use bcmp::{AlgoSpec, MatchIterator};

fn main() {
    let a = "abcdefg";
    let b = "012abc34cdef56efg78abcdefg";
    let match_iter = MatchIterator::new(a.as_bytes(), b.as_bytes(), AlgoSpec::HashMap(2));

    for m in match_iter {
        println!("Match: {:}", &a[m.first_pos..m.first_end()]);
    }
}
```
