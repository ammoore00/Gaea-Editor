# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# test array types
give @s minecraft:stone{foo: [B; 123b, 123b]}
give @s minecraft:stone{foo: [I; 123, 456]}
give @s minecraft:stone{foo: [L; 123L, 456L]}
give @s minecraft:stone{foo: [foo; 123L, 456L]}
