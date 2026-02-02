fn main() {
    macros::parse_roles!({
        a: hello,
        b: world!,
    });
}
