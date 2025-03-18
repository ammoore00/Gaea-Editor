# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# test scoreboard players operation
scoreboard players operation @s foo %= @s bar
scoreboard players operation @s foo *= @s bar
scoreboard players operation @s foo += @s bar
scoreboard players operation @s foo -= @s bar
scoreboard players operation @s foo /= @s bar
scoreboard players operation @s foo < @s bar
scoreboard players operation @s foo = @s bar
scoreboard players operation @s foo > @s bar
scoreboard players operation @s foo >< @s bar

# test execute if score
execute if score @s foo < @s bar run
execute if score @s foo <= @s bar run
execute if score @s foo = @s bar run
execute if score @s foo > @s bar run
execute if score @s foo >= @s bar run
