use drd::{parse::parse, serialise::serialise};

#[test]
fn test_serialise() {
    assert_eq!(serialise(parse("1").unwrap()), "1");

    assert_eq!(serialise(parse("(((1)))").unwrap()), "1");

    assert_eq!(serialise(parse("not true").unwrap()), "not true");

    assert_eq!(serialise(parse("1 * 2 + 3 * 4").unwrap()), "1*2+3*4");
    assert_eq!(serialise(parse("1 * (2 + 3) * 4").unwrap()), "1*(2+3)*4");

    assert_eq!(
        serialise(parse("a, b, c : d, e, f").unwrap()),
        "a,b,c:d,e,f"
    );

    assert_eq!(
        serialise(parse("a = 1; b = 2; c = 3; a + b - c").unwrap()),
        "a=1;b=2;c=3;a+b-c"
    );
    assert_eq!(
        serialise(parse("a = (b = c; d); e").unwrap()),
        "a=(b=c;d);e"
    );

    assert_eq!(serialise(parse("a * b * c").unwrap()), "a*b*c");
    assert_eq!(serialise(parse("(a * b) * c").unwrap()), "a*b*c");
    assert_eq!(serialise(parse("a * (b * c)").unwrap()), "a*(b*c)");
}
