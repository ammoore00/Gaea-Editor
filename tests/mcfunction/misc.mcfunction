# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# root redirect
execute as @a at @s run
execute as @a at @s run 
execute as @a at @s run say
execute as @a at @s run say hello

# booleans
effect give @s minecraft:night_vision 999999 1 true
effect give @s minecraft:night_vision 999999 1 false

# crazy whitespace
execute
execute 
execute  
execute   
execute    
execute     

# non-literal characters
execute.
execute_
execute-
