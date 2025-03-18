# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# bad decimals with absolutes
execute rotated 10 . run say hello
execute rotated 10 5. run say hello
execute rotated 10 - run say hello
execute rotated 10 -. run say hello
execute rotated 10 -5. run say hello

# bad decimals with relatives
execute rotated ~10 ~. run say hello
execute rotated ~10 ~5. run say hello
execute rotated ~10 ~- run say hello
execute rotated ~10 ~-. run say hello
execute rotated ~10 ~-5. run say hello

# can't use locals with rotation
execute rotated ^10 ^ run say hello
execute rotated ^10 ^10 run say hello
execute rotated ^10 ^0.5 run say hello
execute rotated ^10 ^.5 run say hello
execute rotated ^10 ^-10 run say hello
execute rotated ^10 ^-0.5 run say hello
execute rotated ^10 ^-.5 run say hello
