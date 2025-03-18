# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

execute if blocks 0 0 0 1 2 3 -1 -2 -3 all run say hi
execute if blocks ~1 ~2 ~3 ~-1 ~-2 ~-3 ~0 ~0 ~0 all run say hi
execute if blocks ^1 ^2 ^3 ^-1 ^-2 ^-3 ^0 ^0 ^0 all run say hi

# invalid, mixed coords
execute if blocks 1 ^2 ^3 ^-1 ^-2 ^-3 ^0 ^0 ^0 all run say hi
execute if blocks ~1 ^2 ^3 ^-1 ^-2 ^-3 ^0 ^0 ^0 all run say hi
