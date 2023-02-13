use std::{fmt::Display, str::FromStr};

use thiserror::Error;

use crate::DltMessage;

#[derive(Debug, Clone)]
pub enum Filter {
    Ecu {
        ecu_id: String,
    },
    App {
        app_id: String,
    },
    Ctx {
        ctx_id: String,
    },
    And {
        left: Box<Filter>,
        right: Box<Filter>,
    },
    Or {
        left: Box<Filter>,
        right: Box<Filter>,
    },
}

impl Filter {
    fn keep(&self, message: &DltMessage<'_>) -> bool {
        match self {
            Filter::Ecu { ecu_id } => message.storage_header.ecu_id == ecu_id,
            Filter::App { app_id } => message
                .extended_header
                .as_ref()
                .map_or(false, |hdr| hdr.application_id == app_id),
            Filter::Ctx { ctx_id } => message
                .extended_header
                .as_ref()
                .map_or(false, |hdr| hdr.context_id == ctx_id),
            Filter::And { left, right } => left.keep(message) && right.keep(message),
            Filter::Or { left, right } => left.keep(message) || right.keep(message),
        }
    }
}

impl FromStr for Filter {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Ecu { ecu_id } => write!(f, "ecu == \"{ecu_id}\""),
            Filter::App { app_id } => write!(f, "app == \"{app_id}\""),
            Filter::Ctx { ctx_id } => write!(f, "ctx == \"{ctx_id}\""),
            Filter::And { left, right } => write!(f, "({left}) && ({right})"),
            Filter::Or { left, right } => write!(f, "({left}) || ({right})"),
        }
    }
}

#[derive(Debug, Error)]
enum FilterParseError {
    #[error("BadNode")]
    BadNode,
    #[error("Unknown Ident {0}")]
    UnknownIdent(String),
    #[error("GenericError")]
    Generic,
}

mod parse {
    use nom::IResult;

    use super::Filter;

    pub fn filter(input: &str) -> IResult<&str, Filter> {
        todo!()
    }
}

/*#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_ecu_filter() {
        let filter: Filter = "ecu == \"TEST\"".parse().unwrap();
        assert_eq!(format!("{filter}"), "ecu == \"TEST\"")
    }
    #[test]
    fn parse_app_filter() {
        let filter: Filter = "app == \"TEST\"".parse().unwrap();
        assert_eq!(format!("{filter}"), "app == \"TEST\"")
    }
    #[test]
    fn parse_ctx_filter() {
        let filter: Filter = "ctx == \"TEST\"".parse().unwrap();
        assert_eq!(format!("{filter}"), "ctx == \"TEST\"")
    }

    #[test]
    fn parse_and_filter() {
        let filter: Filter = "ecu == \"ECU\" && app == \"APP\"".parse().unwrap();
        assert_eq!(format!("{filter}"), "(ecu == \"ECU\") && (app == \"APP\")")
    }

    #[test]
    fn parse_double_and_filter() {
        let filter: Filter = "ecu == \"ECU\" && app == \"APP\" && ctx == \"CTX\""
            .parse()
            .unwrap();
        assert_eq!(
            format!("{filter}"),
            "((ecu == \"ECU\") && (app == \"APP\")) && (ctx == \"CTX\")"
        )
    }

    #[test]
    fn parse_or_filter() {
        let filter: Filter = "ecu == \"ECU\" || app == \"APP\"".parse().unwrap();
        assert_eq!(format!("{filter}"), "(ecu == \"ECU\") || (app == \"APP\")")
    }

    #[test]
    fn parse_double_or_filter() {
        let filter: Filter = "ecu == \"ECU\"  || app == \"APP\" || ctx == \"CTX\""
            .parse()
            .unwrap();
        assert_eq!(
            format!("{filter}"),
            "((ecu == \"ECU\") || (app == \"APP\")) || (ctx == \"CTX\")"
        )
    }
}
*/
