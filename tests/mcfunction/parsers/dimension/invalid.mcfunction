# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# non-executable
execute in minecraft:overworld
execute in minecraft:the_nether
execute in minecraft:the_end

# bad resource location
execute in :overworld run say hello
execute in minecraft: run say hello
execute in : run say hello

# no slashes
execute in over/world run say hello
execute in minecraft:over/world run say hello
