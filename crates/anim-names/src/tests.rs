macro_rules! test {
    ($name:ident, $all:expr, $fwd:ident, $rev:ident) => {
        #[test]
        fn $name() {
            for (hash, bytes, string) in $all.iter().copied() {
                let expected = Some((hash, string));
                let actual = $fwd(bytes);
                assert_eq!(actual, expected);

                let expected = Some(bytes);
                let actual = $rev(hash, string);
                assert_eq!(actual, expected);
            }
        }
    };
}
pub(crate) use test;
