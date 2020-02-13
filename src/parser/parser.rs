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
use failure::format_err;
use failure::Error;
use pest;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "parser/query.pest"]
struct QueryParser;

/// default log line limit.
const DEFAULT_LIMIT: u64 = 10000;

/// default distance to fuzzy search.
const DEFAULT_DISTANCE: u32 = 2;

/// Selection hold the selection statement.
#[derive(Default, Debug, Clone)]
pub struct Selection {
    pub structured: bool,
    pub attr: Option<String>,
    pub value: String,
}

/// Count will tell which attribute that should be counted
/// on
#[derive(Default, Debug)]
pub struct Count {
    pub attr: String,
    pub alias: String,
    pub by: Option<String>,
}

/// Distinct will allow you to find the distinct attributes
/// on the selection field.
#[derive(Default, Debug)]
pub struct Distinct {
    pub attr: String,
    pub count: bool,
}

/// Average will allow you to do average on the filtered values.
#[derive(Default, Debug)]
pub struct Average {
    pub attr: String,
    pub alias: String,
    pub by: Option<String>,
}

#[derive(Default, Debug)]
pub struct Query {
    pub selection: Option<Selection>,
    pub count: Option<Count>,
    pub distinct: Option<Distinct>,
    pub aggregation_exist: bool, // Pathivu only accepts only one aggregation. Atleast for now.
    pub average: Option<Average>,
    pub soruces: Vec<String>,
    pub limit: u64,
    pub distance: u32,
}

impl Query {
    /// is_aggregation_exist tells whether aggregation
    /// exist or not.
    fn is_aggregation_exist(&self) -> bool {
        self.aggregation_exist
    }
}

/// parse will parse the given query string into internal
/// Pathivu query structure.
pub fn parse(query: String) -> Result<Query, Error> {
    let mut query_inner = Query::default();
    // Don't parse empty string
    if query == "" {
        return Ok(query_inner);
    }
    let mut result = QueryParser::parse(Rule::query, &query)?;
    let tokens = result.next().unwrap();
    parse_query(tokens, &mut query_inner)?;
    if query_inner.is_aggregation_exist() && query_inner.limit != 0 {
        return Err(format_err!(
            "Limit is not supported with aggregation. At least for now"
        ));
    }

    // If there is no limit, update the default limit.
    if query_inner.limit == 0 {
        query_inner.limit = DEFAULT_LIMIT;
    }

    if query_inner.distance == 0 {
        // set the default distance if there is no distance specified.
        query_inner.distance = DEFAULT_DISTANCE;
    }
    Ok(query_inner)
}

/// parse_average will parse average group by.
fn parse_average(pair: Pair<'_, Rule>, mut query: &mut Query) {
    let mut inner = pair.into_inner();
    // parse the mandatory fields.
    let count_attr = inner.next().unwrap().as_str();
    let alias = inner.next().unwrap().as_str();
    let mut average = Average {
        attr: String::from(count_attr),
        alias: String::from(alias),
        by: None,
    };
    // parse if we have any group by.
    match inner.next() {
        Some(pair) => average.by = Some(String::from(pair.as_str())),
        None => {}
    }
    // update the average.
    query.average = Some(average);
    query.aggregation_exist = true;
}

/// parse_query is used to parse the Pathivu query into internal
/// query structure.
pub fn parse_query(pair: Pair<'_, Rule>, mut query: &mut Query) -> Result<(), Error> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::count => {
                if query.is_aggregation_exist() {
                    return Err(format_err!(
                        "{}",
                        "Only one aggregation is supported. At least for now"
                    ));
                }
                parse_count(inner, &mut query);
            }
            Rule::structured => {
                if query.selection.is_some() {
                    return Err(format_err!("{}", "Only one selection statement exist"));
                }
                parse_structured(inner, &mut query)
            }
            Rule::query => parse_query(inner, &mut query)?,
            Rule::query_block => parse_query(inner, &mut query)?,
            Rule::distinct | Rule::distinct_count => {
                if query.is_aggregation_exist() {
                    return Err(format_err!(
                        "{}",
                        "Only one aggregation is supported. At least for now"
                    ));
                }
                parse_distinct(inner, &mut query);
            }
            Rule::average => {
                if query.is_aggregation_exist() {
                    return Err(format_err!(
                        "{}",
                        "Only one aggregation is supported. At least for now"
                    ));
                }
                parse_average(inner, &mut query);
            }
            Rule::unstructured => {
                if query.selection.is_some() {
                    return Err(format_err!("{}", "Only one selection statement exist"));
                }
                parse_unstructured(inner, &mut query);
            }
            Rule::limit => {
                if query.limit != 0 {
                    return Err(format_err!("You can only use one limit operator"));
                }
                parse_limit(inner, &mut query);
            }
            Rule::source => {
                if query.soruces.len() != 0 {
                    return Err(format_err!("You can mention source only once"));
                }
                parse_source(inner, &mut query);
            }
            Rule::distance => {
                if query.distance != 0 {
                    return Err(format_err!("Only one distance limit is allowed"));
                }
                parse_distance(inner, &mut query);
            }
            _ => {}
        }
    }
    Ok(())
}

/// parse_fuzzy will parse the fuzzy distance limit
fn parse_distance(pair: Pair<'_, Rule>, query: &mut Query) {
    let limit = pair.into_inner().next().unwrap().as_str();
    query.distance = limit.parse().unwrap();
}
/// parse_unstructured will parse the unstructed selection statement.
fn parse_unstructured(pair: Pair<'_, Rule>, query: &mut Query) {
    let mut inner = pair.into_inner();
    // Get the message.
    let msg = inner.next().unwrap().as_str();
    let msg = &msg[1..msg.len() - 1];
    let selection = Selection {
        value: msg.to_string(),
        attr: None,
        structured: false,
    };
    query.selection = Some(selection);
}

/// parse_dictinct is used to parse distinct.
fn parse_distinct(pair: Pair<'_, Rule>, query: &mut Query) {
    let rule = pair.as_rule();
    // parse attribute and alias.
    let mut inner = pair.into_inner();
    let attr = inner.next().unwrap().as_str();
    let mut distinct = Distinct {
        attr: String::from(attr),
        count: false,
    };
    // Check whether it is distinct count or not.
    if rule == Rule::distinct_count {
        distinct.count = true;
    }
    query.distinct = Some(distinct);
    query.aggregation_exist = true;
}

/// parse_count is used to parse the count expression.
fn parse_count(pair: Pair<'_, Rule>, query: &mut Query) {
    let mut inner = pair.into_inner();
    // parse the mandatory fields.
    let count_attr = inner.next().unwrap().as_str();
    let alias = inner.next().unwrap().as_str();
    let mut count = Count {
        attr: String::from(count_attr),
        alias: String::from(alias),
        by: None,
    };
    // parse if we have any group by.
    match inner.next() {
        Some(pair) => count.by = Some(String::from(pair.as_str())),
        None => {}
    }
    // update the count.
    query.count = Some(count);
    query.aggregation_exist = true;
}

/// parse_structured is used to parse the structured queries.
fn parse_structured(pair: Pair<'_, Rule>, query: &mut Query) {
    let mut inner = pair.into_inner();
    // parse the flattened key and value.
    let attr = inner.next().unwrap().as_str();
    let value = inner.next().unwrap().as_str();
    let value = &value[1..value.len() - 1];
    let selection = Selection {
        structured: true,
        attr: Some(String::from(attr)),
        value: value.to_string(),
    };
    query.selection = Some(selection);
}

/// parse_source will parse the source that pathivu needs to be quried
/// on.
fn parse_source(pair: Pair<'_, Rule>, query: &mut Query) {
    let sources = pair.into_inner();
    // populate all the sources.
    for source in sources {
        query.soruces.push(source.as_str().to_string());
    }
}

/// parse_limit will parse the limit which is to limit the number of
/// log lines that needs to be responded.
fn parse_limit(pair: Pair<'_, Rule>, query: &mut Query) {
    let limit = pair.into_inner().next().unwrap().as_str();
    query.limit = limit.parse().unwrap();
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_average() {
        // basic assertion
        let query = parse(String::from("avg(weight) as avg_weight")).unwrap();
        let avg = query.average.unwrap();
        assert_eq!(avg.attr, "weight".to_string());
        assert_eq!(avg.alias, "avg_weight".to_string());

        // by assertion
        let query = parse(String::from("avg(weight) as avg_weight by hello")).unwrap();
        let avg = query.average.unwrap();
        assert_eq!(avg.attr, "weight".to_string());
        assert_eq!(avg.alias, "avg_weight".to_string());
        assert_eq!(avg.by.unwrap(), "hello".to_string());
    }

    #[test]
    fn test_average_err() {
        let err = parse(String::from("avg(weight) as avg_weight | limit 10")).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Limit is not supported with aggregation. At least for now"
        );
    }
    #[test]
    fn test_selection() {
        // unstructured selection assertion
        let query = parse(String::from("message = \"pathivu\"")).unwrap();
        let selection = query.selection.unwrap();
        assert_eq!(selection.value, "pathivu");
        assert_eq!(selection.structured, false);

        // structured selection assertion
        let query = parse(String::from("name.location = \"kumari kandam\"")).unwrap();
        let selection = query.selection.unwrap();
        assert_eq!(selection.value, "kumari kandam");
        assert_eq!(selection.structured, true);
        assert_eq!(selection.attr.unwrap(), "name.location");
        let query = parse(String::from("name.location= \"kumari kandam\"")).unwrap();
        let selection = query.selection.unwrap();
        assert_eq!(selection.value, "kumari kandam");
        assert_eq!(selection.structured, true);
        assert_eq!(selection.attr.unwrap(), "name.location");
        let query = parse(String::from("name.location =\"kumari kandam\"")).unwrap();
        let selection = query.selection.unwrap();
        assert_eq!(selection.value, "kumari kandam");
        assert_eq!(selection.structured, true);
        assert_eq!(selection.attr.unwrap(), "name.location");
        let query = parse(String::from("name.location=\"kumari kandam\"")).unwrap();
        let selection = query.selection.unwrap();
        assert_eq!(selection.value, "kumari kandam");
        assert_eq!(selection.structured, true);
        assert_eq!(selection.attr.unwrap(), "name.location");
    }
    #[test]
    fn test_count() {
        // basic assertion.
        let query = parse(String::from("count(location) as num_of_location")).unwrap();
        let count = query.count.unwrap();
        assert_eq!(count.attr, "location");
        assert_eq!(count.alias, "num_of_location");
        assert_eq!(count.by, None);

        // count on by assertion
        let query = parse(String::from(
            "count(location) as num_of_location by country",
        ))
        .unwrap();
        let count = query.count.unwrap();
        assert_eq!(count.attr, "location");
        assert_eq!(count.alias, "num_of_location");
        assert_eq!(count.by.unwrap(), "country");
    }
    #[test]
    fn test_distinct() {
        // basic assertion.
        let query = parse(String::from("distinct(country)")).unwrap();
        let distinct = query.distinct.unwrap();
        assert_eq!(distinct.attr, "country");
        assert_eq!(distinct.count, false);
        // distinct_count assertion.
        let query = parse(String::from("distinct_count(country.hello/world)")).unwrap();
        let distinct = query.distinct.unwrap();
        assert_eq!(distinct.attr, "country.hello/world");
        assert_eq!(distinct.count, true);
    }

    #[test]
    fn test_limit() {
        let query = parse(String::from("limit 100")).unwrap();
        assert_eq!(query.limit, 100);
    }

    #[test]
    fn test_combined() {
        let query = parse(String::from(
            "message=\"succeed\" | count(country) as num_of_country",
        ))
        .unwrap();

        let count = query.count.unwrap();
        assert_eq!(count.attr, "country");
        assert_eq!(count.alias, "num_of_country");
        let selection = query.selection.unwrap();
        assert_eq!(selection.attr, None);
        assert_eq!(selection.value, "succeed");
        assert_eq!(selection.structured, false);
    }
    #[test]
    fn test_distance() {
        let query = parse(String::from("distance 100")).unwrap();
        assert_eq!(query.distance, 100);
    }
}
