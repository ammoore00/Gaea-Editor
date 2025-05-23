# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

execute as @s run execute as @s run say hello
execute as 0-0-0-0-0 run execute as @s run say hello
execute as f7a39418-72ca-4bf2-bc7e-ba9df67a4707 run execute as @s run say hello
execute as Arcensoth run execute as @s run say hello
execute as some_guy run execute as @s run say hello
execute as @s[] run execute as @s run say hello
execute as @s[ ] run execute as @s run say hello
execute as @s[tag=foo] run execute as @s run say hello
execute as @s[tag=!foo] run execute as @s run say hello
execute as @s[ tag = foo ] run execute as @s run say hello
execute as @s[ tag = !foo ] run execute as @s run say hello
execute as @s[ tag = ! foo ] run execute as @s run say hello
execute as @s[tag=foo,tag=bar] run execute as @s run say hello
execute as @s[ tag=foo,tag=bar] run execute as @s run say hello
execute as @s[tag=foo,tag=bar ] run execute as @s run say hello
execute as @s[tag=foo ,tag=bar] run execute as @s run say hello
execute as @s[tag=foo, tag=bar] run execute as @s run say hello
execute as @s[tag=foo , tag=bar] run execute as @s run say hello
execute as @s[ tag = foo , tag = bar ] run execute as @s run say hello
execute as @s[tag = foo ,tag = bar ] run execute as @s run say hello
execute as @s[tag = foo , tag = bar ] run execute as @s run say hello
execute as @s[ tag = foo ,tag  = bar ] run execute as @s run say hello
execute as @s[ tag = foo , tag = bar ] run execute as @s run say hello
execute as @s[tag   =   foo    ,tag   =   bar   ] run execute as @s run say hello
execute as @s[tag   =   foo,tag   =   bar   ] run execute as @s run say hello
execute as @s[tag   =   foo,   tag   =   bar   ] run execute as @s run say hello
execute as @s[   tag   =   foo    ,   tag   =   bar   ] run execute as @s run say hello
execute as @s[tag   =   foo    ,   tag   =   bar] run execute as @s run say hello
execute as @s[ tag = foo , tag = bar , tag = baz ] run execute as @s run say hello
execute as @s[nbt={custom: true}] as @s run say hello
execute as @s[nbt={foo: {bar: []}}] as @s run say hello
execute as @s[nbt={foo: {bar: [{baz: {}}]}}] as @s run say hello
execute as @s[nbt=!{custom: true}] as @s run say hello
execute as @e[tag=foo, sort=nearest, nbt={custom: true}] run execute as @s run execute as @s run say hello

execute as @e[sort=nearest] run say hello
execute as @e[ sort = nearest ] run say hello
execute as @e[  sort  =  nearest  ] run say hello

execute as @e[limit=1] run say hello
execute as @e[limit=01] run say hello
execute as @e[ limit = 1 ] run say hello
execute as @e[  limit  =  1  ] run say hello

execute as @e[team=] run say hi
execute as @e[team=!] run say hi
execute as @e[team=foo] run say hi
execute as @e[team=!foo] run say hi
execute as @e[team=foo.bar] run say hi
execute as @e[team=!foo.bar] run say hi

execute as @e[tag=] run say hi
execute as @e[tag=!] run say hi
execute as @e[tag=foo] run say hi
execute as @e[tag=!foo] run say hi
execute as @e[tag=foo.bar] run say hi
execute as @e[tag=!foo.bar] run say hi

execute as @e[name=] run say hi
execute as @e[name=!] run say hi
execute as @e[name=foo] run say hi
execute as @e[name=!foo] run say hi
execute as @e[name=""] run say hi
execute as @e[name="foo bar"] run say hi
execute as @e[name=!"foo bar"] run say hi
execute as @e[name="foo \" bar"] run say hi
execute as @e[name="foo ' bar"] run say hi
execute as @e[name=''] run say hi
execute as @e[name='foo bar'] run say hi
execute as @e[name=!'foo bar'] run say hi
execute as @e[name='foo \' bar'] run say hi
execute as @e[name='foo " bar'] run say hi

# double quoted keys
execute as @e["quoted_key"=foo] run say hi
execute as @e["quoted key"=foo] run say hi
execute as @e[foo=bar,"quoted key"=foo] run say hi
execute as @e["quoted key"=foo,foo=bar] run say hi
execute as @e[foo=bar,"quoted key"=foo,foo=bar] run say hi
execute as @e[foo=bar,  "quoted key"=foo,  foo=bar] run say hi
execute as @e["quoted key"=foo, "another quoted key"=foo] run say hi
execute as @e["quoted 'x key"=foo] run say hi
execute as @e["quoted 'x' key"=foo] run say hi

# single quoted keys
execute as @e['quoted_key'=foo] run say hi
execute as @e['quoted key'=foo] run say hi
execute as @e[foo=bar,'quoted key'=foo] run say hi
execute as @e['quoted key'=foo,foo=bar] run say hi
execute as @e[foo=bar,'quoted key'=foo,foo=bar] run say hi
execute as @e[foo=bar,  'quoted key'=foo,  foo=bar] run say hi
execute as @e['quoted key'=foo, 'another quoted key'=foo] run say hi
execute as @e['quoted "x key'=foo] run say hi
execute as @e['quoted "x" key'=foo] run say hi
