#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Details {
    pub location: Option<String>,
    pub description: Option<String>,
}

impl Details {
    pub fn new(location: Option<String>, description: Option<String>) -> Option<Self> {
        if location.is_none() && description.is_none() {
            None
        } else {
            Some(Self {
                location,
                description,
            })
        }
    }
}
