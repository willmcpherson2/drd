use crate::parse::parse_program;

mod ast;
mod parse;

fn main() {
    println!(
        "{:#?}",
        parse_program(
            r#"
/* welcome to
my database */

Staff =
  name: 'Alice'; -- first row
  name: 'Bob'    -- second row

bob = name /* add more columns here */ <- Staff ? name == 'Bob'
"#
        )
    );
}
