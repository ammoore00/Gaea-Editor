# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

execute as @s[
execute as @s[ 
execute as @s[sort=nearest,
execute as @s[sort=nearest, 
execute as @s[ run say hello
execute as @s[]run execute as @s run say hello
execute as @s[ ]run execute as @s run say hello
execute as @s [] run execute as @s run say hello
execute as @s [ ] run execute as @s run say hello
execute as @s[nearest] run execute as @s run say hello
execute as @s[nearest ] run execute as @s run say hello
execute as @s[ nearest] run execute as @s run say hello
execute as @s[ nearest ] run execute as @s run say hello
execute as @s[,] run execute as @s run say hello
execute as @s[ ,] run execute as @s run say hello
execute as @s[, ] run execute as @s run say hello
execute as @s[ , ] run execute as @s run say hello
execute as @s[,sort=nearest] run execute as @s run say hello
execute as @s[ ,sort=nearest] run execute as @s run say hello
execute as @s[ ,sort=nearest] run execute as @s run say hello
execute as @s[sort,sort=nearest] run execute as @s run say hello
execute as @s[sort ,sort=nearest] run execute as @s run say hello
execute as @s[sort=,sort=nearest] run execute as @s run say hello
execute as @s[sort=nearest,] run execute as @s run say hello
execute as @s[sort=nearest, ] run execute as @s run say hello
execute as @s[sort=nearest ,, sort=bar] run execute as @s run say hello
execute as @s[sort=nearest , , sort=bar] run execute as @s run say hello
execute as @s[sort=nearest,sort] run execute as @s run say hello
execute as @s[sort=nearest,sort=] run execute as @s run say hello
execute as @s[sort=nearest,sort,sort=bar] run execute as @s run say hello
execute as @s[sort=nearest,sort=,sort=bar] run execute as @s run say hello

execute as @e[sort] run
execute as @e[sort=] run
execute as @e[sort=foo] run
execute as @e[sort=nearestfoo] run
execute as @e[sort=foonearest] run
execute as @e[sort=!nearest] run

execute as @e[limit] run
execute as @e[limit=] run
execute as @e[limit=foo] run
execute as @e[limit=nearest] run
execute as @e[limit=0.1] run
execute as @e[limit=-1] run
execute as @e[limit=!1] run

execute as @e["quoted_key=foo] run say hi
execute as @e[quoted_key"=foo] run say hi
execute as @e["quoted_key""=foo] run say hi
execute as @e["quoted_key"bar=foo] run say hi
execute as @e[bar"quoted_key"=foo] run say hi

execute as @e['quoted_key=foo] run say hi
execute as @e[quoted_key'=foo] run say hi
execute as @e['quoted_key''=foo] run say hi
execute as @e['quoted_key'bar=foo] run say hi
execute as @e[bar'quoted_key'=foo] run say hi
