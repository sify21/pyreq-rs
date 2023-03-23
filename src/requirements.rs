use std::fmt::Display;

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
    fn compare_compatible(&self, prospective: &Version, spec: &str) -> bool {
        if let Ok(("", v)) = version_scheme(spec) {
            // ignore suffix segments(only contains epoch and release)
            self.compare_greater_than_equal(prospective, spec)
                && self.compare_equal(prospective, &v.prefix_str())
        } else {
            // 按道理spec必能解析为Version，但这里做个容错
            false
        }
    }
    // spec中允许包含wildcard(prefix match)和local versions
    fn compare_equal(&self, prospective: &Version, spec: &str) -> bool {
        // prefix matching
        // 按解析的语法, spec只能是[epoch]release.*的格式
        // 在判断prefix match忽略prospective的local segment
        // 我这里的实现跟python不同，没用version_split，是先判断epoch是否相等，再判断release
        if spec.ends_with(".*") {
            if let Ok(("", spec_v)) = version_scheme(&spec[..spec.len() - 2]) {
                if prospective.epoch != spec_v.epoch {
                    return false;
                }
                // 0-pad the prospective version
                // python中的_pad_version是在_version_split数组的release后边加"0"元素，使两个数组长度相同
                for i in 0..prospective.release.len().min(spec_v.release.len()) {
                    if prospective.release[i] != spec_v.release[i] {
                        return false;
                    }
                }
                // prospective.release更多不用处理，因为只要前缀匹配就可以
                // spec_v.release更多的情况, 多出来的部分必须全是0(符合python中的0-pad)
                if spec_v.release.len() > prospective.release.len() {
                    return spec_v.release[prospective.release.len()..spec_v.release.len()]
                        .iter()
                        .all(|&i| i == 0);
                }
                true
            } else {
                false
            }
        } else {
            if let Ok(("", mut spec_v)) = version_scheme(spec) {
                if spec_v.local.is_none() && prospective.local.is_some() {
                    spec_v.local = prospective.local.clone();
                }
                prospective.eq(&spec_v)
            } else {
                false
            }
        }
    }
    fn compare_not_equal(&self, prospective: &Version, spec: &str) -> bool {
        !self.compare_equal(prospective, spec)
    }
    fn compare_less_than_equal(&self, prospective: &Version, spec: &str) -> bool {
        true
    }
    fn compare_greater_than_equal(&self, prospective: &Version, spec: &str) -> bool {
        true
    }
    fn compare_less_than(&self, prospective: &Version, spec: &str) -> bool {
        true
    }
    fn compare_greater_than(&self, prospective: &Version, spec: &str) -> bool {
        true
    }
    fn compare_arbitrary(&self, prospective: &Version, spec: &str) -> bool {
        prospective.to_string().eq_ignore_ascii_case(spec)
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

#[derive(Debug, PartialEq, Clone)]
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

// this is a version identifier, defined in pep 440, it is not the same as the string used in VersionSpec
// TODO impl Ord for sorting Versions
// public version identifier = [N!]N(.N)*[{a|b|rc}N][.postN][.devN]
// local version identifier = <public version identifier>[+<local version label>]
#[derive(Debug, PartialEq, Default)]
pub struct Version {
    pub epoch: u64,
    pub release: Vec<u64>,
    pub pre: Option<(String, u64)>,
    pub post: Option<(String, u64)>,
    pub dev: Option<(String, u64)>,
    pub local: Option<Vec<LocalVersionPart>>,
}

impl Version {
    // 用于compare_compatible
    pub fn prefix_str(&self) -> String {
        let mut parts = String::new();
        // epoch
        if self.epoch != 0 {
            parts.push_str(&format!("{}!", self.epoch));
        }
        // 忽略 release 最后一位，用'.*'替代
        if self.release.len() > 1 {
            for i in &self.release[..self.release.len() - 1] {
                parts.push_str(&format!("{}.", i));
            }
        }
        parts.push_str(".*");
        parts
    }

    // The public portion of the version.(without local)
    pub fn public_str(&self) -> String {
        self.canonicalize_str(false, false)
    }

    // canonicalize_version at https://github.com/pypa/packaging/blob/main/src/packaging/utils.py
    // strip_trailing_zero: 不包含release后边的'.0'. 用VersionSpec的哈希和相等比较，见Specifier中的_canonical_spec, __hash__, __eq__. https://github.com/pypa/packaging/blob/main/src/packaging/specifiers.py
    // with_local: public_str 不包含local version part
    pub fn canonicalize_str(&self, strip_trailing_zero: bool, with_local: bool) -> String {
        let mut parts = String::new();
        // epoch
        if self.epoch != 0 {
            parts.push_str(&format!("{}!", self.epoch));
        }
        // release
        if strip_trailing_zero {
            for i in self
                .release
                .iter()
                .rev()
                .skip_while(|&&r| r == 0)
                .collect::<Vec<&u64>>()
                .iter()
                .rev()
            {
                parts.push_str(&format!("{}.", i));
            }
        } else {
            for i in self.release.iter() {
                parts.push_str(&format!("{}.", i));
            }
        }
        parts.truncate(parts.len() - 1);
        // pre-release
        if let Some((l, n)) = self.pre.as_ref() {
            parts.push_str(&format!("{}{}", l, n));
        }
        // post-release
        if let Some((_, n)) = self.post.as_ref() {
            parts.push_str(&format!(".post{}", n));
        }
        // dev-release
        if let Some((_, n)) = self.dev.as_ref() {
            parts.push_str(&format!(".dev{}", n));
        }
        // local version segment
        if with_local {
            if let Some(local) = self.local.as_ref() {
                parts.push_str("+");
                for i in local.iter() {
                    parts.push_str(&format!("{}.", i));
                }
                parts.truncate(parts.len() - 1);
            }
        }
        parts
    }
}

// refer to Version.__str__ from https://github.com/pypa/packaging/blob/main/src/packaging/version.py
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.canonicalize_str(false, true))
    }
}
