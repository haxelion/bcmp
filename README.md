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


Examples
--------

Iterate over the matches between two strings using `HashMatch` with a 
minimum match length of 2 bytes:

``` rust
extern crate bcmp;

use bcmp::{AlgoSpec, MatchIterator};

fn main() {
    let a = "abcdefg";
    let b = "012abc34cdef56efg78abcdefg";
    let match_iter = MatchIterator::new(a.as_bytes(), b.as_bytes(), AlgoSpec::HashMatch(2));
    for m in match_iter {
        println!("Match: {:}", &a[m.first_pos..m.first_end()]);
    }
}
```

Construct a patch set to build the file `b` from the file `a` using `TreeMatch`
with a minimum match length of 4 bytes:

``` rust
extern crate bcmp;

use std::fs::File;
use std::io::Read;

use bcmp::{AlgoSpec, patch_set};

fn main() {
    let mut a = Vec::<u8>::new();
    let mut b = Vec::<u8>::new();
    File::open("a").unwrap().read_to_end(&mut a);
    File::open("b").unwrap().read_to_end(&mut b);

    let ps = patch_set(&a, &b, AlgoSpec::TreeMatch(4));
    for patch in ps {
        println!("b[0x{:x}..0x{:x}] == a[0x{:x}..0x{:x}]", patch.second_pos, patch.second_end(), patch.first_pos, patch.first_end());
    }
}
```
