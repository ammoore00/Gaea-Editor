# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# all absolutes
execute rotated 10 0 run say hello
execute rotated 10 10 run say hello
execute rotated 10 0.5 run say hello
execute rotated 10 .5 run say hello
execute rotated 10 -10 run say hello
execute rotated 10 -0.5 run say hello
execute rotated 10 -.5 run say hello

# absolutes mixed with relatives
execute rotated 10 ~ run say hello
execute rotated 10 ~10 run say hello
execute rotated 10 ~0.5 run say hello
execute rotated 10 ~.5 run say hello
execute rotated 10 ~-10 run say hello
execute rotated 10 ~-0.5 run say hello
execute rotated 10 ~-.5 run say hello

# all relatives
execute rotated ~10 ~ run say hello
execute rotated ~10 ~10 run say hello
execute rotated ~10 ~0.5 run say hello
execute rotated ~10 ~.5 run say hello
execute rotated ~10 ~-10 run say hello
execute rotated ~10 ~-0.5 run say hello
execute rotated ~10 ~-.5 run say hello
