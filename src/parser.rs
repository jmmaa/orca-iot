use nom::{
    branch::alt,
    bytes::streaming::tag,
    character::complete::char,
    combinator::{all_consuming, complete, eof},
    multi::many0,
    number::complete::float,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

use std::collections::HashMap;

pub fn whitespace(input: &str) -> IResult<&str, char, nom::error::Error<&str>> {
    let mut parse = alt((char(' '), char('\n'), char('\t'), char('\r')));
    parse(input)
}

pub fn whitespaces(input: &str) -> IResult<&str, Vec<char>, nom::error::Error<&str>> {
    let mut parse = many0(whitespace);

    parse(input)
}

pub fn value(input: &str) -> IResult<&str, f32, nom::error::Error<&str>> {
    let mut parse = alt((
        terminated(float, whitespaces),
        delimited(whitespaces, float, whitespaces),
        preceded(whitespaces, float),
        float,
    ));

    parse(input)
}

pub fn keyword<'a>(
    name: &'a str,
) -> impl FnMut(&'a str) -> IResult<&str, &str, nom::error::Error<&str>> {
    alt((
        terminated(tag(name), whitespaces),
        delimited(whitespaces, tag(name), whitespaces),
        preceded(whitespaces, tag(name)),
        tag(name),
    ))
}

pub fn reading<'a>(
    name: &'a str,
) -> impl FnMut(&'a str) -> IResult<&str, (&str, f32), nom::error::Error<&str>> {
    return separated_pair(keyword(name), char(':'), value);
}

pub type Readings<'a> = (
    (&'a str, f32),
    (&'a str, f32),
    (&'a str, f32),
    (&'a str, f32),
    (&'a str, f32),
);

pub fn readings(input: &str) -> IResult<&str, Readings, nom::error::Error<&str>> {
    let mut parse = complete(tuple((
        reading("temperature"),
        reading("pressure"),
        reading("windspeed"),
        reading("waterlevel"),
        terminated(reading("humidity"), eof),
    )));

    parse(input)
}

pub fn consume(input: &str) -> IResult<&str, Readings, nom::error::Error<&str>> {
    let mut parse = all_consuming(readings);
    parse(input)
}

pub fn parse(input: &str) -> Result<ParsedData, nom::Err<nom::error::Error<&str>>> {
    return match consume(input) {
        Ok(res) => Ok(ParsedData { readings: res.1 }),
        Err(e) => Err(e),
    };
}

pub struct ParsedData<'a> {
    readings: Readings<'a>,
}

impl<'a> ParsedData<'a> {
    pub fn to_tuple(&self) -> Readings {
        self.readings
    }
    pub fn to_hashmap(&self) -> HashMap<&str, f32> {
        let mut hm = HashMap::new();

        let windspeed = self.readings.0;
        let pressure = self.readings.1;
        let temperature = self.readings.2;
        let humidity = self.readings.3;
        let waterlevel = self.readings.4;

        hm.insert(windspeed.0, windspeed.1);
        hm.insert(pressure.0, pressure.1);
        hm.insert(temperature.0, temperature.1);
        hm.insert(humidity.0, humidity.1);
        hm.insert(waterlevel.0, waterlevel.1);

        hm
    }
}
