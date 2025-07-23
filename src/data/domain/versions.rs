use mc_version::{MinecraftVersion, PackFormat};
use mc_version_macro::define_versions;

define_versions! {
    data = [
        (4, [1.13..1.13.2, 1.14..1.14.4]),
        (5, [1.15..1.15.2, 1.16..1.16.5]),
        (7, 1.17..1.17.1),
        (8, 1.18..1.18.1),
        (9, 1.18.2),
        (10, 1.19..1.19.3),
        (12, 1.19.4),
        (15, 1.20..1.20.1),
        (18, 1.20.2),
        (26, 1.20.3..1.20.4),
        (41, 1.20.5..1.20.6),
        (48, 1.21..1.21.1),
        (57, 1.21.2..1.21.3),
        (61, 1.21.4),
        (71, 1.21.5),
        (80, 1.21.6),
        (81, 1.21.7),
    ],
    resource = [
        (4, [1.13..1.13.2, 1.14..1.14.4]),
        (5, [1.15..1.15.2, 1.16..1.16.5]),
        (7, 1.17..1.17.1),
        (8, 1.18..1.18.2),
        (9, 1.19..1.19.2),
        (12, 1.19.3),
        (13, 1.19.4),
        (15, 1.20..1.20.1),
        (18, 1.20.2),
        (22, 1.20.3..1.20.4),
        (32, 1.20.5..1.20.6),
        (34, 1.21..1.21.1),
        (42, 1.21.2..1.21.3),
        (46, 1.21.4),
        (55, 1.21.5),
        (63, 1.21.6),
        (64, 1.21.7),
    ]
}

pub fn latest() -> MinecraftVersion {
    *V1_21_7
}

#[macro_export]
macro_rules! latest_data_format {
    () => {
        crate::data::domain::versions::get_datapack_format_for_version(crate::data::domain::versions::latest())
    };
}

#[macro_export]
macro_rules! latest_resource_format {
    () => {
        crate::data::domain::versions::get_resourcepack_format_for_version(crate::data::domain::versions::latest())
    };
}

// TODO: cache this as part of the macro for better performance

pub fn get_datapack_format_for_version(version: MinecraftVersion) -> &'static PackFormat {
    for format in &*DATA_FORMAT_MAP {
        let format = *format.value();

        if format.get_versions().read().unwrap().contains(&version) {
            return format
        }
    }

    // Panic because this can only result from a static bug and should never fail at runtime
    panic!("No datapack format found for version {}", version)
}

pub fn get_resourcepack_format_for_version(version: MinecraftVersion) -> &'static PackFormat {
    for format in &*RESOURCE_FORMAT_MAP {
        let format = *format.value();
        
        if format.get_versions().read().unwrap().contains(&version) {
            return format
        }
    }
    
    // Panic because this can only result from a static bug and should never fail at runtime
    panic!("No resourcepack format found for version {}", version)
}