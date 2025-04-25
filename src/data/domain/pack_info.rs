pub struct PackInfo {
    description: String,
    
    type_specific_data: PackTypeData,
}

pub enum PackTypeData {
    DataPack(DatapackInfo),
    ResourcePack(ResourcepackInfo),
    Combined(DatapackInfo, ResourcepackInfo),
}

pub struct DatapackInfo {
    // TODO: add datapack specific data
}

pub struct ResourcepackInfo {
    // TODO: add resourcepack specific data
}