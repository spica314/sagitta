use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SagittaConfigToml {
    pub ignores: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_sagitta_config_toml() {
        let toml = r#"
ignores = [
    "target",
    "Cargo.lock",
]
"#;
        let config: SagittaConfigToml = toml::from_str(toml).unwrap();
        assert_eq!(config.ignores, vec!["target", "Cargo.lock"]);
    }
}
