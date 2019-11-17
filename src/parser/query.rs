/*
 * Copyright 2019 Balaji Jinnah and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use nom::character::complete::digit1;
use nom::character::complete::space1;
use nom::error::ErrorKind;
use nom::error::ErrorKind::{Eof, Tag};
use nom::Err::{Error, Failure, Incomplete};
use nom::{
    alt, delimited, do_parse, eat_separator, eof, named, opt, parse_to, tag, take, take_str,
    take_till,
};

#[derive(Debug)]
pub struct Query {
    partition: String,
    query_str: String,
}

named!(parse_selection<&str, &str>, do_parse!(
    tag!("SELECT") >>
    eat_separator!(&b" \t"[..]) >>
    query_str: take_till!(|ch| ch == ' ') >>
    eat_separator!(&b" \t"[..]) >>
    (query_str)
));

named!(parse_from<&str, &str>, do_parse!(
    tag!("FROM") >>
    eat_separator!(&b" \t"[..]) >>
    partition_name: take_till!(|ch| ch == ';') >>
    take!(1) >> // consume semicolon
    eof!() >>
    (partition_name)
));

named!(
    parse_top<&str,Option<&str>>,
    opt!(do_parse!(tag!("TOP") >> 
    eat_separator!(&b" \t"[..]) >>
    count: take_till!(|ch| ch == ' ') >>
    eat_separator!(&b" \t"[..]) >>
    (count)))
);

/// parse_query will parse the query. please don't even add a single line in this
/// file without test case. I manually tested all the possible case. I just wrote
/// without test case beacause, I want to bootstarp it asap. But, I'll promise that
/// I'll write test case.
pub fn parse_query(query: &str) -> Result<Query, String> {
    let res = parse_selection(query);
    if let Err(e) = res {
        match e {
            Error(e) | Failure(e) => match e.1 {
                Tag => return Err(String::from("expected SELECT statement")),
                _ => return Err(String::from("unexpected query")),
            },
            Incomplete(_) => {
                return Err(String::from(
                    "incomplete SELECT statement. eg: SELECT account FROM app_name;",
                ))
            }
        }
    }
    let res = res.unwrap();
    let selection = String::from(res.1);
    let res = parse_from(res.0);
    if let Err(e) = res {
        match e {
            Error(e) | Failure(e) => match e.1 {
                Tag => return Err(format!("expected FROM after SELECT {}", selection)),
                Eof => return Err(format!("expected end of line after;")),
                _ => return Err(format!("unexpected query")),
            },
            Incomplete(_) => {
                return Err(String::from(
                    "incomplete SELECT statement. eg: SELECT account FROM app_name;",
                ))
            }
        }
    }
    let res = res.unwrap();
    let partition = String::from(res.1);
    Ok(Query {
        query_str: selection,
        partition: partition,
    })
}
