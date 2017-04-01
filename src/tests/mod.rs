use longest_common_substring;
use longest_common_substrings;

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
    let ms = longest_common_substrings::<u32>(a.as_bytes(), b.as_bytes(), 10);
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
