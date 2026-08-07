#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Details {
    pub location: Option<String>,
    pub description: Option<String>,
}

impl Details {
    pub fn diff(&self, other: &Details) -> Vec<String> {
        macro_rules! diff {
            ($field:ident, $format:literal) => {
                match (&self.$field, &other.$field) {
                    (None, None) => None,
                    (Some(msg1), Some(msg2)) => {
                        if msg1 == msg2 {
                            None
                        } else {
                            Some(format!("Update {}", $format))
                        }
                    }
                    (None, Some(_)) => Some(format!("Add {}", $format)),
                    (Some(_), None) => Some(format!("Remove {}", $format)),
                }
            };
        }
        vec![
            diff!(location, "location"),
            diff!(description, "description"),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
    pub fn new(location: Option<String>, description: Option<String>) -> Self {
        Self {
            location,
            description,
        }
    }
    pub fn to_strs(&self) -> (String, String) {
        (
            self.location.as_deref().unwrap_or("").to_string(),
            self.description.as_deref().unwrap_or("").to_string(),
        )
    }
    pub fn has_details(&self) -> bool {
        self.location.is_some() || self.description.is_some()
    }
}
