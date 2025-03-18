# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# just ellipsis
execute if score @s foo matches .. run say hello

# decimals
execute if score @s foo matches 0.5 run say hello
execute if score @s foo matches 0.5.. run say hello
execute if score @s foo matches ..0.5 run say hello
execute if score @s foo matches -0.5..0.5 run say hello

# technically invalid, but hard to detect with regex
execute if score @s foo matches 1..-1 run say hello
