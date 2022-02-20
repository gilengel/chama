pub mod ui;

use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum PluginExecutionBehaviour {
    NonRestricted,
    Exclusive,
    Always,
    Once   
}

impl FromStr for PluginExecutionBehaviour {

    type Err = ();

    fn from_str(input: &str) -> Result<PluginExecutionBehaviour, Self::Err> {
        match input {
            "NonRestricted"  => Ok(PluginExecutionBehaviour::NonRestricted),
            "Exclusive"  => Ok(PluginExecutionBehaviour::Exclusive),
            "Always"  => Ok(PluginExecutionBehaviour::Always),
            "Once" => Ok(PluginExecutionBehaviour::Once),
            _      => Err(()),
        }
    }
}