use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "drd.pest"]
pub struct DrdParser;

fn main() {
    let successful_parse = DrdParser::parse(
        Rule::program,
        "Staff = [{name 'Alice'} {name 'Bob'}]; bob = name <- Staff ? name == 'Bob';",
    );
    println!("{:#?}", successful_parse);
}
