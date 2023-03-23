//! refer to https://github.com/pypa/packaging/blob/main/src/packaging/version.py
//! 正则忽略大小写
//! this is version scheme, 与requirement_specifier中用的version string不同, 在specifier中还能用通配符*

use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_while_m_n},
    character::complete::{alphanumeric1, char as nomchar, digit1, satisfy},
    combinator::opt,
    multi::many0,
    sequence::{preceded, terminated, tuple},
    IResult, Parser,
};

use crate::requirements::{LocalVersionPart, Version};

pub fn epoch(input: &str) -> IResult<&str, u64> {
    terminated(digit1, nomchar('!'))
        .map(|s: &str| s.parse::<u64>().unwrap())
        .parse(input)
}

pub fn release(input: &str) -> IResult<&str, Vec<u64>> {
    digit1
        .and(many0(preceded(nomchar('.'), digit1)))
        .map(|(first, mut rest)| {
            rest.insert(0, first);
            rest.into_iter()
                .map(|s: &str| s.parse::<u64>().unwrap())
                .collect()
        })
        .parse(input)
}

// pre-release letter
pub fn pre_l(input: &str) -> IResult<&str, &str> {
    // 得把长的字符串往前放，不然匹配上a后，就不匹配后边的lpha了
    alt((
        tag_no_case("alpha"),
        tag_no_case("preview"),
        tag_no_case("beta"),
        tag_no_case("a"),
        tag_no_case("b"),
        tag_no_case("c"),
        tag_no_case("rc"),
        tag_no_case("pre"),
    ))(input)
}

//see _parse_letter_version from https://github.com/pypa/packaging/blob/main/src/packaging/version.py
pub fn pre(input: &str) -> IResult<&str, (String, u64)> {
    tuple((
        take_while_m_n(0, 1, |c| "-_.".contains(c)),
        pre_l,
        take_while_m_n(0, 1, |c| "-_.".contains(c)),
        opt(digit1),
    ))
    .map(|(_, l, _, n)| {
        let number = n.map_or(0u64, |s| s.parse().unwrap());
        let letter = l.to_lowercase();
        match letter.as_str() {
            "alpha" => ("a".to_string(), number),
            "beta" => ("b".to_string(), number),
            "c" | "pre" | "preview" => ("rc".to_string(), number),
            _ => (letter, number),
        }
    })
    .parse(input)
}

pub fn post_l(input: &str) -> IResult<&str, &str> {
    alt((tag_no_case("post"), tag_no_case("rev"), tag_no_case("r")))(input)
}

pub fn post(input: &str) -> IResult<&str, (String, u64)> {
    alt((
        preceded(nomchar('-'), digit1)
            .map(|s: &str| ("post".to_string(), s.parse::<u64>().unwrap())),
        tuple((
            take_while_m_n(0, 1, |c| "-_.".contains(c)),
            post_l,
            take_while_m_n(0, 1, |c| "-_.".contains(c)),
            opt(digit1),
        ))
        .map(|(_, _, _, n)| ("post".to_string(), n.map_or(0u64, |s| s.parse().unwrap()))),
    ))(input)
}

pub fn dev(input: &str) -> IResult<&str, (String, u64)> {
    tuple((
        take_while_m_n(0, 1, |c| "-_.".contains(c)),
        tag_no_case("dev"),
        take_while_m_n(0, 1, |c| "-_.".contains(c)),
        opt(digit1),
    ))
    .map(|(_, _, _, n)| {
        (
            "dev".to_string(),
            n.map_or(0u64, |s: &str| s.parse().unwrap()),
        )
    })
    .parse(input)
}

// see _parse_local_version from https://github.com/pypa/packaging/blob/main/src/packaging/version.py
pub fn local(input: &str) -> IResult<&str, Vec<LocalVersionPart>> {
    preceded(
        nomchar('+'),
        alphanumeric1
            .and(many0(preceded(
                satisfy(|c| "-_.".contains(c)),
                alphanumeric1,
            )))
            .map(|(first, mut rest)| {
                rest.insert(0, first);
                rest.into_iter()
                    .map(|s: &str| match s.parse::<u64>() {
                        Ok(n) => LocalVersionPart::Num(n),
                        Err(_) => LocalVersionPart::LowerStr(s.to_lowercase()),
                    })
                    .collect()
            }),
    )(input)
}

pub fn version_scheme(input: &str) -> IResult<&str, Version> {
    tuple((
        opt(nomchar('v')),
        opt(epoch),
        release,
        opt(pre),
        opt(post),
        opt(dev),
        opt(local),
    ))
    .map(|(_, e, r, p, po, de, lo)| Version {
        epoch: e.unwrap_or(0u64),
        release: r,
        pre: p,
        post: po,
        dev: de,
        local: lo,
    })
    .parse(input)
}
