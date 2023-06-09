//! 解析(requirement specifier)[https://pip.pypa.io/en/stable/reference/requirement-specifiers]
//! refer to https://peps.python.org/pep-0508/ for the complete parsley grammar.
//! -> pythonExpression 是表示解析'->'前面的一串语法, 对应的python返回值是什么
use crate::requirements::{Comparison, MarkerExpr, MarkerOp, RequirementSpecifier, VersionSpec};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while_m_n},
    character::{
        complete::{char as nomchar, digit0, digit1, hex_digit1, satisfy, space0, space1},
        is_alphabetic, is_alphanumeric, is_digit, is_hex_digit, is_space,
    },
    combinator::{eof, map, opt, recognize},
    multi::{count, many0, many1, many_m_n},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult, Parser,
};

use super::version::{dev, epoch, local, post, pre, release};

// wsp* = space0

pub fn version_cmp(input: &str) -> IResult<&str, Comparison> {
    map(
        preceded(
            space0,
            alt((
                tag("<="),
                tag("<"),
                tag("!="),
                tag("=="),
                tag(">="),
                tag(">"),
                tag("~="),
                tag("==="),
            )),
        ),
        |s| Comparison::try_from(s).unwrap(),
    )(input)
}

// version可以进一步细化为4个分类，参考https://github.com/pypa/packaging/blob/main/src/packaging/specifiers.py
// Specifier类中的_version_regex_str
// 根据operator的不同有不同的要求

pub fn version_arbitraryequal(input: &str) -> IResult<&str, String> {
    map(
        preceded(
            space0,
            take_while(|c: char| !c.is_whitespace() && c != ';' && c != ')'),
        ),
        |s: &str| s.to_string(),
    )(input)
}

pub fn version_equal_notequal(input: &str) -> IResult<&str, String> {
    map(
        preceded(
            space0,
            recognize(tuple((
                opt(nomchar('v')),
                opt(epoch),
                release,
                opt(alt((
                    tag(".*"),
                    recognize(tuple((opt(pre), opt(post), opt(dev), opt(local)))),
                ))),
            ))),
        ),
        |s: &str| s.to_string(),
    )(input)
}

pub fn version_compatiblerelease(input: &str) -> IResult<&str, String> {
    map(
        preceded(
            space0,
            recognize(tuple((
                opt(nomchar('v')),
                opt(epoch),
                digit1.and(many1(preceded(nomchar('.'), digit1))),
                opt(pre),
                opt(post),
                opt(dev),
            ))),
        ),
        |s: &str| s.to_string(),
    )(input)
}

pub fn version_other_operator(input: &str) -> IResult<&str, String> {
    map(
        preceded(
            space0,
            recognize(tuple((
                opt(nomchar('v')),
                opt(epoch),
                release,
                opt(pre),
                opt(post),
                opt(dev),
            ))),
        ),
        |s: &str| s.to_string(),
    )(input)
}

pub fn version_one(input: &str) -> IResult<&str, VersionSpec> {
    let (input, operator) = version_cmp(input)?;
    match operator {
        Comparison::ArbitraryEqual => terminated(version_arbitraryequal, space0)
            .map(|v| (operator, v).into())
            .parse(input),
        Comparison::Equal | Comparison::NotEqual => terminated(version_equal_notequal, space0)
            .map(|v| (operator, v).into())
            .parse(input),
        Comparison::CompatibleRelease => terminated(version_compatiblerelease, space0)
            .map(|v| (operator, v).into())
            .parse(input),
        _ => terminated(version_other_operator, space0)
            .map(|v| (operator, v).into())
            .parse(input),
    }
}

pub fn version_many(input: &str) -> IResult<&str, Vec<VersionSpec>> {
    version_one
        .and(many0(preceded(space0.and(nomchar(',')), version_one)))
        .map(|(one, mut v)| {
            v.insert(0, one);
            v
        })
        .parse(input)
}

pub fn versionspec(input: &str) -> IResult<&str, Vec<VersionSpec>> {
    delimited(nomchar('('), version_many, nomchar(')'))
        .or(version_many)
        .parse(input)
}

pub fn urlspec(input: &str) -> IResult<&str, &str> {
    preceded(nomchar('@').and(space0), uri_reference)(input)
}

pub fn marker_op(input: &str) -> IResult<&str, MarkerOp> {
    alt((
        version_cmp.map(|cmp| cmp.into()),
        preceded(space0, tag("in")).map(|_| MarkerOp::In),
        preceded(space0, tag("not"))
            .and(preceded(space1, tag("in")))
            .map(|_| MarkerOp::NotIn),
    ))(input)
}

pub fn is_python_str_c(c: char) -> bool {
    is_space(c as u8) || is_alphanumeric(c as u8) || "().{}-_*#:;,/?[]!~`@$%^&=+|<>".contains(c)
}

pub fn python_str(input: &str) -> IResult<&str, &str> {
    delimited(
        nomchar('\''),
        take_while(|c| is_python_str_c(c) || c == '"'),
        nomchar('\''),
    )
    .or(delimited(
        nomchar('"'),
        take_while(|c| is_python_str_c(c) || c == '\''),
        nomchar('"'),
    ))
    .parse(input)
}

pub fn env_var(input: &str) -> IResult<&str, &str> {
    alt((
        tag("python_version"),
        tag("python_full_version"),
        tag("os_name"),
        tag("sys_platform"),
        tag("platform_release"),
        tag("platform_system"),
        tag("platform_version"),
        tag("platform_machine"),
        tag("platform_python_implementation"),
        tag("implementation_name"),
        tag("implementation_version"),
        tag("extra"),
    ))(input)
}

pub fn marker_var(input: &str) -> IResult<&str, &str> {
    preceded(space0, env_var.or(python_str))(input)
}

// 表达式优先级: op > () > and > or
// marker_expr = basic | '(' marker_or ')'
// marker_and = marker_expr 'and' marker_expr | marker_expr
// marker_or = marker_and 'or' marker_and | marker_and
// marker = marker_or
// 从下往上看更容易理解

pub fn marker_expr(input: &str) -> IResult<&str, MarkerExpr> {
    alt((
        // 不用考虑空格的问题，因为marker_var和marker_op都是只吃前边的空格，后边的空格不管
        tuple((marker_var, marker_op, marker_var))
            .map(|(left, op, right)| MarkerExpr::Basic(left.to_string(), op, right.to_string())),
        delimited(
            preceded(space0, nomchar('(')),
            marker_or,
            preceded(space0, nomchar(')')),
        ),
    ))(input)
}

pub fn marker_and(input: &str) -> IResult<&str, MarkerExpr> {
    alt((
        separated_pair(
            marker_expr,
            delimited(space0, tag("and"), space0),
            marker_expr,
        )
        .map(|(left, right)| MarkerExpr::And(Box::new(left), Box::new(right))),
        marker_expr,
    ))(input)
}

pub fn marker_or(input: &str) -> IResult<&str, MarkerExpr> {
    alt((
        separated_pair(marker_and, delimited(space0, tag("or"), space0), marker_and)
            .map(|(left, right)| MarkerExpr::Or(Box::new(left), Box::new(right))),
        marker_and,
    ))(input)
}

// marker = marker_or

pub fn quoted_marker(input: &str) -> IResult<&str, MarkerExpr> {
    preceded(nomchar(';').and(space0), marker_or)(input)
}

pub fn identifier_end(input: &str) -> IResult<&str, &str> {
    recognize(take_while(|c| "-_.".contains(c)).and(satisfy(|u| is_alphanumeric(u as u8))))(input)
}

// name = identifier
pub fn identifier(input: &str) -> IResult<&str, String> {
    recognize(satisfy(|u| is_alphanumeric(u as u8)).and(many0(identifier_end)))
        .map(|s| s.to_string())
        .parse(input)
}

pub fn extras_list(input: &str) -> IResult<&str, Vec<String>> {
    identifier
        .and(many0(preceded(
            delimited(space0, nomchar(','), space0),
            identifier,
        )))
        .map(|(first, mut rest)| {
            rest.insert(0, first);
            rest
        })
        .parse(input)
}

pub fn extras(input: &str) -> IResult<&str, Option<Vec<String>>> {
    delimited(
        nomchar('[').and(space0),
        opt(extras_list),
        space0.and(nomchar(']')),
    )
    .parse(input)
}

pub fn name_req(input: &str) -> IResult<&str, RequirementSpecifier> {
    tuple((
        identifier,
        space0,
        opt(extras),
        space0,
        opt(versionspec),
        space0,
        opt(quoted_marker),
    ))
    .map(|(i, _, e, _, v, _, m)| RequirementSpecifier {
        name: i,
        extras: if let Some(Some(j)) = e { j } else { vec![] },
        version_specs: if let Some(j) = v { j } else { vec![] },
        marker_expr: m,
        ..Default::default()
    })
    .parse(input)
}

pub fn url_req(input: &str) -> IResult<&str, RequirementSpecifier> {
    tuple((
        identifier,
        space0,
        opt(extras),
        space0,
        urlspec,
        alt((space1, eof)),
        opt(quoted_marker),
    ))
    .map(|(i, _, e, _, v, _, m)| RequirementSpecifier {
        name: i,
        extras: if let Some(Some(j)) = e { j } else { vec![] },
        urlspec: Some(v.to_string()),
        marker_expr: m,
        ..Default::default()
    })
    .parse(input)
}

pub fn specification(input: &str) -> IResult<&str, RequirementSpecifier> {
    delimited(space0, alt((url_req, name_req)), space0)(input)
}

// following is URI rules. https://www.rfc-editor.org/rfc/rfc3986#appendix-A
// ABNF syntax is at https://www.rfc-editor.org/rfc/rfc5234

pub fn uri(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        scheme,
        nomchar(':'),
        hier_part,
        opt(nomchar('?').and(query)),
        opt(nomchar('#').and(fragment)),
    )))(input)
}

pub fn hier_part(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(tuple((tag("//"), authority, path_abempty))),
        path_absolute,
        path_rootless,
        path_empty,
    ))(input)
}

pub fn uri_reference(input: &str) -> IResult<&str, &str> {
    alt((uri, relative_ref))(input)
}

pub fn absolute_uri(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        scheme,
        nomchar(':'),
        hier_part,
        opt(nomchar('?').and(query)),
    )))(input)
}

pub fn relative_ref(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        relative_part,
        opt(nomchar('?').and(query)),
        opt(nomchar('#').and(fragment)),
    )))(input)
}

pub fn relative_part(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(tuple((tag("//"), authority, path_abempty))),
        path_absolute,
        path_noscheme,
        path_empty,
    ))(input)
}

pub fn scheme(input: &str) -> IResult<&str, &str> {
    recognize(satisfy(|c| is_alphabetic(c as u8)).and(take_while(|c| {
        is_alphanumeric(c as u8) || c == '+' || c == '-' || c == '.'
    })))(input)
}

pub fn authority(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        opt(userinfo.and(nomchar('@'))),
        host,
        opt(nomchar(':').and(digit0)),
    )))(input)
}

pub fn userinfo(input: &str) -> IResult<&str, &str> {
    recognize(many0(alt((unreserved, pct_encoded, sub_delims, tag(":")))))(input)
}

pub fn host(input: &str) -> IResult<&str, &str> {
    alt((ip_literal, ipv4address, reg_name))(input)
}

// port = digit0

pub fn ip_literal(input: &str) -> IResult<&str, &str> {
    recognize(delimited(
        nomchar('['),
        alt((ipv6address, ipvfuture)),
        nomchar(']'),
    ))(input)
}

pub fn ipvfuture(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        nomchar('v'),
        hex_digit1,
        nomchar('.'),
        many1(alt((unreserved, sub_delims, tag(":")))),
    )))(input)
}

pub fn ipv6address(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(count(h16.and(nomchar(':')), 6).and(ls32)),
        recognize(tuple((tag("::"), count(h16.and(nomchar(':')), 5), ls32))),
        recognize(tuple((
            opt(h16),
            tag("::"),
            count(h16.and(nomchar(':')), 4),
            ls32,
        ))),
        recognize(tuple((
            opt(many_m_n(0, 1, h16.and(nomchar(':'))).and(h16)),
            tag("::"),
            count(h16.and(nomchar(':')), 3),
            ls32,
        ))),
        recognize(tuple((
            opt(many_m_n(0, 2, h16.and(nomchar(':'))).and(h16)),
            tag("::"),
            count(h16.and(nomchar(':')), 2),
            ls32,
        ))),
        recognize(tuple((
            opt(many_m_n(0, 3, h16.and(nomchar(':'))).and(h16)),
            tag("::"),
            h16.and(nomchar(':')),
            ls32,
        ))),
        recognize(tuple((
            opt(many_m_n(0, 4, h16.and(nomchar(':'))).and(h16)),
            tag("::"),
            ls32,
        ))),
        recognize(tuple((
            opt(many_m_n(0, 5, h16.and(nomchar(':'))).and(h16)),
            tag("::"),
            h16,
        ))),
        recognize(tuple((
            opt(many_m_n(0, 6, h16.and(nomchar(':'))).and(h16)),
            tag("::"),
        ))),
    ))(input)
}

pub fn h16(input: &str) -> IResult<&str, &str> {
    take_while_m_n(1, 4, |c| is_hex_digit(c as u8))(input)
}

pub fn ls32(input: &str) -> IResult<&str, &str> {
    alt((recognize(tuple((h16, nomchar(':'), h16))), ipv4address))(input)
}

pub fn ipv4address(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        dec_octet,
        nomchar('.'),
        dec_octet,
        nomchar('.'),
        dec_octet,
        nomchar('.'),
        dec_octet,
    )))(input)
}

pub fn dec_octet(input: &str) -> IResult<&str, &str> {
    alt((
        take_while_m_n(1, 1, |c| is_digit(c as u8)),
        recognize(tuple((
            satisfy(|c| "123456789".contains(c)),
            satisfy(|c| is_digit(c as u8)),
        ))),
        recognize(tuple((
            nomchar('1'),
            take_while_m_n(2, 2, |c| is_digit(c as u8)),
        ))),
        recognize(tuple((
            nomchar('2'),
            satisfy(|c| "01234".contains(c)),
            satisfy(|c| is_digit(c as u8)),
        ))),
        recognize(tuple((tag("25"), satisfy(|c| "012345".contains(c))))),
    ))(input)
}

pub fn reg_name(input: &str) -> IResult<&str, &str> {
    recognize(many0(alt((unreserved, pct_encoded, sub_delims))))(input)
}

pub fn path(input: &str) -> IResult<&str, &str> {
    alt((
        path_abempty,
        path_absolute,
        path_noscheme,
        path_rootless,
        path_empty,
    ))(input)
}

pub fn path_abempty(input: &str) -> IResult<&str, &str> {
    recognize(many0(nomchar('/').and(segment)))(input)
}

pub fn path_absolute(input: &str) -> IResult<&str, &str> {
    recognize(nomchar('/').and(opt(segment_nz.and(many0(nomchar('/').and(segment))))))(input)
}

pub fn path_noscheme(input: &str) -> IResult<&str, &str> {
    recognize(segment_nz_nc.and(many0(nomchar('/').and(segment))))(input)
}

pub fn path_rootless(input: &str) -> IResult<&str, &str> {
    recognize(segment_nz.and(many0(nomchar('/').and(segment))))(input)
}

pub fn path_empty(input: &str) -> IResult<&str, &str> {
    tag("")(input)
}

pub fn segment(input: &str) -> IResult<&str, &str> {
    recognize(many0(pchar))(input)
}

pub fn segment_nz(input: &str) -> IResult<&str, &str> {
    recognize(many1(pchar))(input)
}

pub fn segment_nz_nc(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((unreserved, pct_encoded, sub_delims, tag("@")))))(input)
}

pub fn pchar(input: &str) -> IResult<&str, &str> {
    alt((unreserved, pct_encoded, sub_delims, tag(":"), tag("@")))(input)
}

pub fn query(input: &str) -> IResult<&str, &str> {
    recognize(many0(alt((pchar, tag("/"), tag("?")))))(input)
}

pub fn fragment(input: &str) -> IResult<&str, &str> {
    recognize(many0(alt((pchar, tag("/"), tag("?")))))(input)
}

pub fn pct_encoded(input: &str) -> IResult<&str, &str> {
    recognize(tag("%").and(take_while_m_n(2, 2, |c| is_hex_digit(c as u8))))(input)
}

pub fn unreserved(input: &str) -> IResult<&str, &str> {
    take_while_m_n(1, 1, |c| is_alphanumeric(c as u8) || "-._~".contains(c))(input)
}

pub fn reserved(input: &str) -> IResult<&str, &str> {
    alt((gen_delims, sub_delims))(input)
}

pub fn gen_delims(input: &str) -> IResult<&str, &str> {
    alt((
        tag(":"),
        tag("/"),
        tag("?"),
        tag("#"),
        tag("["),
        tag("]"),
        tag("@"),
    ))(input)
}

pub fn sub_delims(input: &str) -> IResult<&str, &str> {
    alt((
        tag("!"),
        tag("$"),
        tag("&"),
        tag("'"),
        tag("("),
        tag(")"),
        tag("*"),
        tag("+"),
        tag(","),
        tag(";"),
        tag("="),
    ))(input)
}
