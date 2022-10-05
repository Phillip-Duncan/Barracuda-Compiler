use std::str::FromStr;
use simple_error::SimpleError;
use regex::Regex;

#[derive(Debug)]
pub struct CLIEnvVarDescriptor {
    pub identifier: String,
    pub given_address: Option<usize>,
    pub given_value: Option<f64>
}

impl FromStr for CLIEnvVarDescriptor {
    type Err = SimpleError;

    /// Convert string to EnvVarDescriptor
    /// Syntax: identifier(:address)?(=value)?
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"(?P<identifier>[A-Za-z][a-zA-Z0-9_]*)(:(?P<address>[0-9]+))?(=(?P<value>[+-]?([0-9]*[.])?[0-9]+))?").unwrap();

        if let Some(caps) = re.captures(input) {
            let identifier = caps.name("identifier")
                .and_then(|m| Some(String::from(m.as_str())))
                .ok_or(SimpleError::new("Environment variable must have a valid identifier"))?;
            let address = caps.name("address")
                .and_then(|m| Some(String::from(m.as_str())))
                .and_then(|s| s.parse::<usize>().ok());
            let value = caps.name("value")
                .and_then(|m| Some(String::from(m.as_str())))
                .and_then(|s| s.parse::<f64>().ok());

            Ok(Self {
                identifier,
                given_address: address,
                given_value: value
            })
        } else {
            bail!("Environment variable must be of the form identifier(:address)?(=value)?")
        }
    }
}