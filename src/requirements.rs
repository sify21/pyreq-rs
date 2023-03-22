// version_cmp
#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Default)]
pub struct RequirementSpecifier {
    pub name: String,
    pub extras: Vec<String>,
    pub version_specs: Vec<(Comparison, String)>,
    pub urlspec: Option<String>,
    pub marker_expr: Option<MarkerExpr>,
}
