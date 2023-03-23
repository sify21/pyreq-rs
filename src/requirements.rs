use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::digit1, combinator::recognize,
    sequence::tuple, IResult, Parser,
};

use crate::parser::version::version_scheme;

// version_cmp
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Comparison {
    LessThan,
    LessThanOrEqual,
    NotEqual,
    Equal,
    GreaterThanOrEqual,
    GreaterThan,
    CompatibleRelease,
    ArbitraryEqual,
}

impl TryFrom<&str> for Comparison {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<" => Ok(Self::LessThan),
            "<=" => Ok(Self::LessThanOrEqual),
            "!=" => Ok(Self::NotEqual),
            "==" => Ok(Self::Equal),
            ">=" => Ok(Self::GreaterThanOrEqual),
            ">" => Ok(Self::GreaterThan),
            "~=" => Ok(Self::CompatibleRelease),
            "===" => Ok(Self::ArbitraryEqual),
            _ => Err(()),
        }
    }
}

// marker_op
#[derive(Debug, PartialEq)]
pub enum MarkerOp {
    Comparison(Comparison),
    In,
    NotIn,
}

// and 优先级大于 or
#[derive(Debug, PartialEq)]
pub enum MarkerExpr {
    Basic(String, MarkerOp, String),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl From<Comparison> for MarkerOp {
    fn from(c: Comparison) -> Self {
        Self::Comparison(c)
    }
}

#[derive(Debug, PartialEq)]
pub enum VersionControlSystem {
    Git,
    Mercurial,
    Subversion,
    Bazaar,
    Unknown,
}

// see regex for VersionSpecifier at https://github.com/pypa/packaging/blob/main/src/packaging/specifiers.py
#[derive(Debug, PartialEq)]
pub struct VersionSpec(pub Comparison, pub String);

impl From<(Comparison, String)> for VersionSpec {
    fn from((c, v): (Comparison, String)) -> Self {
        Self(c, v)
    }
}

impl VersionSpec {
    // refer to contains at https://github.com/pypa/packaging/blob/main/src/packaging/specifiers.py
    // 该方法默认允许pre-releases
    pub fn contains(&self, version: &str) -> bool {
        if let Ok((_, v)) = version_scheme(version) {
            match self.0 {
                Comparison::CompatibleRelease => self.compare_compatible(&v, &self.1),
                Comparison::Equal => self.compare_equal(&v, &self.1),
                Comparison::NotEqual => self.compare_not_equal(&v, &self.1),
                Comparison::LessThanOrEqual => self.compare_less_than_equal(&v, &self.1),
                Comparison::GreaterThanOrEqual => self.compare_greater_than_equal(&v, &self.1),
                Comparison::LessThan => self.compare_less_than(&v, &self.1),
                Comparison::GreaterThan => self.compare_greater_than(&v, &self.1),
                Comparison::ArbitraryEqual => self.compare_arbitrary(&v, &self.1),
            }
        } else {
            // invalid version, just return false
            false
        }
    }

    // ~=2.2 is equivalent to >=2.2,==2.*
    fn compare_compatible(&self, version: &Version, spec: &str) -> bool {
        let segs: Vec<&str> = Self::version_split(&self.1)
            .into_iter()
            .take_while(|&s| Self::is_not_suffix(s))
            .collect();
        let mut prefix = if segs.len() > 1 {
            segs[..segs.len() - 1].join(".")
        } else {
            String::new()
        };
        prefix.push_str(".*");
        self.compare_greater_than_equal(version, spec) && self.compare_equal(version, &prefix)
    }
    fn compare_equal(&self, version: &Version, spec: &str) -> bool {
        if spec.ends_with(".*") {}
        true
    }
    fn compare_not_equal(&self, version: &Version, spec: &str) -> bool {
        !self.compare_equal(version, spec)
    }
    fn compare_less_than_equal(&self, version: &Version, spec: &str) -> bool {
        true
    }
    fn compare_greater_than_equal(&self, version: &Version, spec: &str) -> bool {
        true
    }
    fn compare_less_than(&self, version: &Version, spec: &str) -> bool {
        true
    }
    fn compare_greater_than(&self, version: &Version, spec: &str) -> bool {
        true
    }
    fn compare_arbitrary(&self, version: &Version, spec: &str) -> bool {
        true
    }

    // 处理1.3.3a9的情况
    fn prefix_regex(input: &str) -> IResult<&str, (&str, &str)> {
        tuple((
            digit1,
            recognize(alt((tag("a"), tag("b"), tag("c"), tag("rc"))).and(digit1)),
        ))(input)
    }

    fn version_split(version: &str) -> Vec<&str> {
        let mut ret = vec![];
        for item in version.split('.') {
            if let Ok(("", (a, b))) = Self::prefix_regex(item) {
                ret.push(a);
                ret.push(b);
            } else {
                ret.push(item);
            }
        }
        ret
    }

    fn is_not_suffix(segment: &str) -> bool {
        !["dev", "a", "b", "rc", "post"]
            .into_iter()
            .any(|s| segment.eq(s))
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct RequirementSpecifier {
    pub name: String,
    pub extras: Vec<String>,
    pub version_specs: Vec<VersionSpec>,
    pub urlspec: Option<String>,
    pub marker_expr: Option<MarkerExpr>,
}

impl RequirementSpecifier {
    pub fn contains(&self, version: &str) -> bool {
        self.version_specs.iter().all(|spec| spec.contains(version))
    }
}

#[derive(Debug, PartialEq)]
pub enum LocalVersionPart {
    LowerStr(String),
    Num(u64),
}

impl Display for LocalVersionPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalVersionPart::LowerStr(s) => write!(f, "{}", s),
            LocalVersionPart::Num(n) => write!(f, "{}", n),
        }
    }
}

// this is version scheme, defined in pep 440.
// this is a version identifier, it is not the same as the string used in VersionSpec
// TODO impl Ord for sorting Versions
#[derive(Debug, PartialEq, Default)]
pub struct Version {
    pub epoch: u64,
    pub release: Vec<u64>,
    pub pre: Option<(String, u64)>,
    pub post: Option<(String, u64)>,
    pub dev: Option<(String, u64)>,
    pub local: Option<Vec<LocalVersionPart>>,
}

// refer to Version.__str__ from https://github.com/pypa/packaging/blob/main/src/packaging/version.py
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = vec![];
        if self.epoch != 0 {
            parts.push(format!("{}!", self.epoch));
        }
        parts.push(
            self.release
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<String>>()
                .join("."),
        );
        if let Some((l, n)) = self.pre.as_ref() {
            parts.push(format!("{}{}", l, n));
        }
        if let Some((_, n)) = self.post.as_ref() {
            parts.push(format!(".post{}", n));
        }
        if let Some((_, n)) = self.dev.as_ref() {
            parts.push(format!(".dev{}", n));
        }
        if let Some(local) = self.local.as_ref() {
            parts.push(format!(
                "+{}",
                local
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(".")
            ));
        }
        write!(f, "{}", parts.join(""))
    }
}
