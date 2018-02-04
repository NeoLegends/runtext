use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use serde_yaml;

use multi::Multi;

/// Configuration.
pub type Config = Multi<Context>;

/// Context configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Context {
    /// The actions to execute when the context is active.
    pub actions: HashMap<String, serde_yaml::Value>,

    /// The name of the context.
    #[serde(default)]
    pub name: String,

    /// Evidence sources for activating the context.
    pub triggers: HashMap<String, serde_yaml::Value>,

    /// Context activity behavior.
    #[serde(default)]
    pub trigger_behavior: TriggerBehavior,
}

/// Specifys whether _all_ evidence sources have to indicate that their
/// respective context is active or just one for the actions to be triggered.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerBehavior {
    /// All evidence sources have to indicate context activity.
    ///
    /// This is the default.
    And,

    /// Only one evidence source has to indicate context activity.
    Or
}

/// The ways a context configuration can be invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    MissingTriggers,
    MissingActions,
}

impl Config {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match *self {
            Multi::Single(ref ctx) => ctx.validate(),
            Multi::Multiple(ref ctxs) => {
                for context in ctxs {
                    if let Err(err) = context.validate() {
                        return Err(err);
                    }
                }

                Ok(())
            },
        }
    }
}

impl Context {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.triggers.len() == 0 {
            return Err(ValidationError::MissingTriggers);
        }

        if self.actions.len() == 0 {
            return Err(ValidationError::MissingActions);
        }

        Ok(())
    }
}

impl Default for TriggerBehavior {
    fn default() -> Self {
        TriggerBehavior::And
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for ValidationError {
    fn description(&self) -> &'static str {
        match *self {
            ValidationError::MissingActions => "Missing actions to execute",
            ValidationError::MissingTriggers => "Missing action triggers",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_json() {
        let cfg_single = r#"{
            "actions": { "command": "rclone -V" },
            "name": "Test",
            "triggers": { "wifi": "Wifi" },
            "trigger_behavior": "or"
        }"#;
        let cfg_list = r#"[{
            "actions": { "command": "rclone -V" },
            "name": "Test",
            "triggers": { "wifi": "Wifi" },
            "trigger_behavior": "or"
        }]"#;

        test_serialization(cfg_single, cfg_list)
    }

    #[test]
    fn deserializes_yaml() {
        let cfg_single = r#"
          actions:
            command: rclone -V
          name: Test
          triggers:
            wifi: Wifi
          trigger_behavior: or
        "#;
        let cfg_list = r#"
          - actions:
              command: rclone -V
            name: Test
            triggers:
              wifi: Wifi
            trigger_behavior: or
        "#;

        test_serialization(cfg_single, cfg_list)
    }

    fn test_serialization(single_input: &str, multi_input: &str) {
        let single: Config = serde_yaml::from_str(single_input).unwrap();
        assert!(single.is_single());
        single.validate().unwrap();

        let multiple: Config = serde_yaml::from_str(multi_input).unwrap();
        assert!(multiple.is_multiple());
        multiple.validate().unwrap();

        let single = single.unwrap_single();
        let multiple = multiple.unwrap_multiple();

        assert_eq!(single, multiple[0]);

        assert_eq!(single.name, "Test");
        assert_eq!(single.actions.len(), 1);
        assert_eq!(single.triggers.len(), 1);
        assert_eq!(single.trigger_behavior, TriggerBehavior::Or);
    }

    #[test]
    #[should_panic]
    fn validate_json_fail() {
        let cfg = r#"{
            "actions": {},
            "name": "Test",
            "triggers": { "wifi": "Wifi" },
            "trigger_behavior": "or"
        }"#;

        let cfg: Config = serde_yaml::from_str(cfg).unwrap();
        cfg.validate().unwrap();
    }

    #[test]
    #[should_panic]
    fn validate_yaml_fail() {
        let cfg = r#"
          actions: {}
          name: Test
          triggers:
            wifi: Wifi
          trigger_behavior: or
        "#;

        let cfg: Config = serde_yaml::from_str(cfg).unwrap();
        cfg.validate().unwrap();
    }
}
