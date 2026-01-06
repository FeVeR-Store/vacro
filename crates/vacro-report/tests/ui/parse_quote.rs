#[vacro_report::scope]
fn main() {
    macros::parse_stmt!(let a = 10);
}
