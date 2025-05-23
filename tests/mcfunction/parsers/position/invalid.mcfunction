# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# bad decimals with absolutes
execute positioned 10 . -10 run say hello
execute positioned 10 5. -10 run say hello
execute positioned 10 - -10 run say hello
execute positioned 10 -. -10 run say hello
execute positioned 10 -5. -10 run say hello

# bad decimals with relatives
execute positioned ~10 ~. ~-10 run say hello
execute positioned ~10 ~5. ~-10 run say hello
execute positioned ~10 ~- ~-10 run say hello
execute positioned ~10 ~-. ~-10 run say hello
execute positioned ~10 ~-5. ~-10 run say hello

# bad decimals with locals
execute positioned ^10 ^. ^-10 run say hello
execute positioned ^10 ^5. ^-10 run say hello
execute positioned ^10 ^- ^-10 run say hello
execute positioned ^10 ^-. ^-10 run say hello
execute positioned ^10 ^-5. ^-10 run say hello

# bad mixed locals
execute positioned 1 2 ^3 run say hello
execute positioned 1 ^2 3 run say hello
execute positioned ^1 2 3 run say hello
execute positioned ~1 ~2 ^3 run say hello
execute positioned ~1 ^2 ~3 run say hello
execute positioned ^1 ~2 ~3 run say hello

# not a real operator
execute positioned %1 %2 %3 run say hello
