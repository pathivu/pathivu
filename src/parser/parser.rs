use pest;
use pest::Parser;
#[macro_use]
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "/home/schoolboy/pathivu/src/parser/query.pest"]
pub struct QueryParser;

fn parse(query: String) {
    let result = QueryParser::parse(Rule::query, &query);
    println!("{:?}", result);
}
