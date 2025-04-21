pub struct PackInfo {
    description: String,
    // TODO: Make this properly typed
    filter: Vec<String>,
    
    type_specific_data: PackTypeData,
}

pub enum PackTypeData {
    DataPack(DatapackInfo),
    ResourcePack(ResourcepackInfo),
    Combined(DatapackInfo, ResourcepackInfo),
}

pub struct DatapackInfo {
    // TODO: Make this properly typed
    experimental_features: Vec<String>,
}

pub struct ResourcepackInfo {
    // TODO: Make this properly typed
    language: Vec<String>,
}