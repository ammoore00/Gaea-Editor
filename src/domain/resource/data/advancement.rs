use crate::domain::resource::resource::{ResourceCategory, ResourceData, ResourcePath};

#[derive(Clone, Debug)]
struct Advancement;

impl ResourcePath for Advancement {
    fn category() -> ResourceCategory {
        ResourceCategory::Data
    }

    fn root_path() -> String {
        "advancement".to_string()
    }
}

impl ResourceData for Advancement {}