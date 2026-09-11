//! The `--format json|text` value enum shared by every tool.

/// Output format selected with `--format`. Spelled `json` / `text` on the
/// command line, exactly as the tools always have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Json,
    #[default]
    Text,
}

impl Format {
    pub fn is_json(self) -> bool {
        matches!(self, Format::Json)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::Text => "text",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::ValueEnum;

    #[test]
    fn flag_spellings_are_lowercase() {
        let names: Vec<_> = Format::value_variants()
            .iter()
            .map(|v| v.to_possible_value().unwrap().get_name().to_string())
            .collect();
        assert_eq!(names, vec!["json", "text"]);
    }

    #[test]
    fn parses_from_flag_text() {
        assert_eq!(Format::from_str("json", true).unwrap(), Format::Json);
        assert_eq!(Format::from_str("text", true).unwrap(), Format::Text);
        assert!(Format::from_str("yaml", true).is_err());
    }

    #[test]
    fn default_is_text() {
        assert_eq!(Format::default(), Format::Text);
        assert!(!Format::default().is_json());
        assert!(Format::Json.is_json());
    }
}
