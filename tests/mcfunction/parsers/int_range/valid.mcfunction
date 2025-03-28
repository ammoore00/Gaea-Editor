# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# exact numbers
execute if score @s foo matches -1 run say hello
execute if score @s foo matches 0 run say hello
execute if score @s foo matches 1 run say hello

# minimum
execute if score @s foo matches 1.. run say hello
execute if score @s foo matches 0.. run say hello
execute if score @s foo matches -1.. run say hello

# maximum
execute if score @s foo matches ..1 run say hello
execute if score @s foo matches ..0 run say hello
execute if score @s foo matches ..-1 run say hello

# min and max
execute if score @s foo matches -1..1 run say hello
