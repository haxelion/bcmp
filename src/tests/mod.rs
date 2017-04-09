use longest_common_substring;
use longest_common_substrings;
use patch_set;
use unique_strings;
use ukkonen::SuffixTree;

#[test]
fn lcs() {
    let a = "abcdefghijklmnopqrstuvwxyz";
    let b = "rstufghijklmnopqvwxyzabcde";
    let m = longest_common_substring::<u32>(a.as_bytes(), b.as_bytes());
    assert!(m.first_pos  == 5);
    assert!(m.second_pos == 4);
    assert!(m.length     == 12);
}

#[test]
fn lcss() {
    let a = "abcdefghijklmnopqrstuvwxyz";
    let b = "rstufghijklmnopqvwxyzabcde";
    let ms = longest_common_substrings::<u16>(a.as_bytes(), b.as_bytes(), 10);
    assert!(ms.len() == 4);
    assert!(ms[0].first_pos  == 5);
    assert!(ms[0].second_pos == 4);
    assert!(ms[0].length     == 12);
    assert!(ms[1].first_pos  == 0);
    assert!(ms[1].second_pos == 21);
    assert!(ms[1].length     == 5);
    assert!(ms[2].first_pos  == 21);
    assert!(ms[2].second_pos == 16);
    assert!(ms[2].length     == 5);
    assert!(ms[3].first_pos  == 17);
    assert!(ms[3].second_pos == 0);
    assert!(ms[3].length     == 4);
}

#[test]
fn ps1() {
    let a = "abcdefghijqrstuvwxyzfghijklmnopqr";
    let b = "abcdefghijklmnopqrstuvwxyz";
    let ps = patch_set::<u64>(a.as_bytes(), b.as_bytes());
    assert!(ps.len() == 3);
    assert!(ps[0].first_pos  == 0);
    assert!(ps[0].second_pos == 0);
    assert!(ps[0].length     == 10);
    assert!(ps[1].first_pos  == 25);
    assert!(ps[1].second_pos == 10);
    assert!(ps[1].length     == 8);
    assert!(ps[2].first_pos  == 12);
    assert!(ps[2].second_pos == 18);
    assert!(ps[2].length     == 8);
}

#[test]
fn ps2() {
    let a = "abcdefghijklmnhijklmnopqrstuopqrstuvwxyz";
    let b = "abcdefghijklmnopqrstuvwxyz";
    let ps = patch_set::<u8>(a.as_bytes(), b.as_bytes());
    assert!(ps.len() == 2);
    assert!(ps[0].first_pos  == 0);
    assert!(ps[0].second_pos == 0);
    assert!(ps[0].length     == 14);
    assert!(ps[1].first_pos  == 28);
    assert!(ps[1].second_pos == 14);
    assert!(ps[1].length     == 12);
}

#[test]
fn ps3() {
    let a = "abcdefghijklhijklmnhijklmnopqrstuqrstuvwxyz";
    let b = "abcdefghijklmnopqrstuvwxyz";
    let ps = patch_set::<u32>(a.as_bytes(), b.as_bytes());
    assert!(ps.len() == 3);
    assert!(ps[0].first_pos  == 0);
    assert!(ps[0].second_pos == 0);
    assert!(ps[0].length     == 12);
    assert!(ps[1].first_pos  == 24);
    assert!(ps[1].second_pos == 12);
    assert!(ps[1].length     == 9);
    assert!(ps[2].first_pos  == 38);
    assert!(ps[2].second_pos == 21);
    assert!(ps[2].length     == 5);
}

#[test]
fn us1() {
    let a = "abcdefghijklmnopqrstuvwxyz";
    let b = "abcdef01ghijklmnop3456qrstuvwxyz";
    let us = unique_strings::<u32>(a.as_bytes(), b.as_bytes());
    assert!(us.len() == 2);
    assert!(us[0].0 == 6);
    assert!(us[0].1 == 8);
    assert!(us[1].0 == 18);
    assert!(us[1].1 == 22);
}

#[test]
fn us2() {
    let a = "abcdefghijklmnopqrstuvwxyz";
    let b = "01234";
    let us = unique_strings::<u16>(a.as_bytes(), b.as_bytes());
    assert!(us.len() == 1);
    assert!(us[0].0 == 0);
    assert!(us[0].1 == 5);
}

#[test]
fn us3() {
    let a = "abcdefghijklmnopqrstuvwxyz";
    let b = "1234abcd5678";
    let us = unique_strings::<u8>(a.as_bytes(), b.as_bytes());
    assert!(us.len() == 2);
    assert!(us[0].0 == 0);
    assert!(us[0].1 == 4);
    assert!(us[1].0 == 8);
    assert!(us[1].1 == 12);
}

#[test]
fn us4() {
    let a = "abcdefghijklmnopqrstuvwxyz";
    let b = "abcdefghaxelionijklmnopqrstuvwxyz";
    let us = unique_strings::<u32>(a.as_bytes(), b.as_bytes());
    assert!(us.len() == 1);
    assert!(us[0].0 == 8);
    assert!(us[0].1 == 15);
    let us = unique_strings::<u8>(a.as_bytes(), b.as_bytes());
    assert!(us.len() == 0);
}

#[test]
fn stree1() {
    let s = "banana";
    let stree = SuffixTree::new(s.as_bytes());
    println!("{}", &stree.to_graphviz(s.as_bytes()));
}
