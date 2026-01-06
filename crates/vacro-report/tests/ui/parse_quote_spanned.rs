#[vacro_report::scope]
fn main() {
    macros::parse_stmt_spanned!(let a = 10);
}
