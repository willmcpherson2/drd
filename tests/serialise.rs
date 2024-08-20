use drd::{parse::parse, serialise::serialise};

macro_rules! run {
    ($input:expr, $output:expr) => {{
        let parsed = parse($input).unwrap();
        let serialised = serialise(parsed.clone());
        assert_eq!(serialised, $output);
        let re_parsed = parse(&serialised).unwrap();
        assert_eq!(parsed, re_parsed);
    }};
}

#[test]
fn test_serialise() {
    run!("1", "1");

    run!("(((1)))", "1");

    run!("not true", "not true");

    run!("1 * 2 + 3 * 4", "1*2+3*4");
    run!("1 * (2 + 3) * 4", "1*(2+3)*4");

    run!("a, b, c : d, e, f", "a,b,c:d,e,f");

    run!("a = 1; b = 2; c = 3; a + b - c", "a=1;b=2;c=3;a+b-c");
    run!("a = (b = c; d); e", "a=(b=c;d);e");

    run!("a * b * c", "a*b*c");
    run!("(a * b) * c", "a*b*c");
    run!("a * (b * c)", "a*(b*c)");
}
