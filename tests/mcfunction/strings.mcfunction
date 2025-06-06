# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# literal
# without contenxt, there is no way to differentiate
# literal arguments from literal subcommands
tag @s add mytag

# literal
tag @s add my_tag

# literal
say hello

# literals
say hello world

# unquoted string
# could be confused with an nbt path
# but nbt paths typically have at least one capital letter
tag @s add my.tag

# unquoted string
tag @s add my-tag

# maybe nbt path
data get entity @s My.Tag

# definitely nbt path
data get entity @s My.Tag[0]

# quoted string
say "hello world"
say "hello ' world"
say 'hello world'
say 'hello " world'

# quoted string escaped
say "hello \" world"
say 'hello \' world'

# player names
scoreboard players reset $mypack:some.fakeplayer
scoreboard players reset $mypack.core:another.fakeplayer

# quoted string with trailing characters
say "oh oh"bad
say 'oh oh'bad

# quoted string with terminal backslash
say "uh oh\
say 'uh oh\

# quoted string unbounded
say "uh oh
# this should be a comment, otherwise
# the quoted string is probably leaking

say 'uh oh
# same here
