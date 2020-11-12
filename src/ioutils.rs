use super::{DisplayError, Item, Problem, Solution};
use std::fs;
use std::str::FromStr;

#[derive(Debug)]
pub struct SolutionsFromFile(pub Vec<Solution>);

impl FromStr for SolutionsFromFile {
    type Err = DisplayError;
    fn from_str(file_name: &str) -> Result<SolutionsFromFile, DisplayError> {
        Ok(SolutionsFromFile(
            fs::read_to_string(file_name)
                .map_err(|e| {
                    DisplayError(format!(
                        "Could not solution load file: {}, because: {}",
                        file_name, e
                    ))
                })?
                .lines()
                .map(|line| {
                    parse_solution_line(line).map_err(|e| format!("{}\nSolution Line: {}", e, line))
                })
                .collect::<Result<_, _>>()
                .map_err(DisplayError)?,
        ))
    }
}

#[derive(Debug)]
pub struct ProblemFromfile(pub Vec<Problem>);

impl FromStr for ProblemFromfile {
    type Err = DisplayError;
    fn from_str(file_name: &str) -> Result<ProblemFromfile, DisplayError> {
        Ok(ProblemFromfile(
            fs::read_to_string(file_name)
                .map_err(|e| {
                    DisplayError(format!(
                        "Could not problem load file: {}, because: {}",
                        file_name, e
                    ))
                })?
                .lines()
                .map(|line| {
                    parse_problem_line(line).map_err(|e| format!("{}\nProblem Line: {}", e, line))
                })
                .collect::<Result<_, _>>()
                .map_err(DisplayError)?,
        ))
    }
}

pub fn next_parse_with_err<'a, T, K>(iter: &mut T) -> Result<K, String>
where
    T: Iterator<Item = &'a str>,
    K: FromStr,
    <K as std::str::FromStr>::Err: std::fmt::Debug,
{
    Ok(iter
        .next()
        .ok_or_else(|| "Line exhasted, but next item was expecting".to_string())?
        .parse()
        .map_err(|e| format!("Could not parse number {:?}", e))?)
}

pub fn parse_problem_line(line: &str) -> Result<Problem, String> {
    let mut iter = line.split(' ').filter(|x| !x.is_empty());
    let id: i32 = next_parse_with_err(&mut iter)?;
    let size = next_parse_with_err(&mut iter)?;
    let max_weight = next_parse_with_err(&mut iter)?;
    let min_cost = match () {
        () if id < 0 => Ok(Some(next_parse_with_err(&mut iter)?)),
        () if id > 0 => Ok(None),
        _ => Err("zero id not permitted".to_string()),
    }?;
    let items = (0..size)
        .map(|_| {
            let weight = next_parse_with_err(&mut iter)?;
            let cost = next_parse_with_err(&mut iter)?;
            Ok(Item { weight, cost })
        })
        .collect::<Result<Vec<_>, String>>()?;
    assert_eq!(
        iter.next(),
        None,
        "Line was not exhausted, wrong problem line!"
    );
    Ok(Problem {
        id: id.abs() as u32,
        max_weight,
        size,
        min_cost,
        items,
    })
}

pub fn parse_solution_line(line: &str) -> Result<Solution, String> {
    let mut iter = line.split(' ').filter(|x| !x.is_empty());
    let id = next_parse_with_err(&mut iter)?;
    let size = next_parse_with_err(&mut iter)?;
    let cost = next_parse_with_err(&mut iter)?;
    let items = Some(
        (0..size)
            .map(|_| {
                match iter
                    .next()
                    .ok_or_else(|| "Not enough bits in line!".to_string())?
                {
                    "1" => Ok(true),
                    "0" => Ok(false),
                    _ => Err("Reference solution is not in (0, 1)!".to_string()),
                }
            })
            .collect::<Result<Vec<_>, String>>()?,
    );
    if iter.next() != None {
        return Err("Line was not exhausted, wrong solution line!".to_string());
    }
    Ok(Solution {
        id,
        size,
        cost,
        items,
    })
}
