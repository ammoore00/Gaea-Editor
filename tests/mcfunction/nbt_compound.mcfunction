# Copyright (c) Arcensoth 2019-2020
# This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

# test compounds
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: true }
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: true, bar: 1234, baz: "hello world" }
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: { bar: true } }
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: { bar: true, baz: 1234 } }
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: { bar: true, baz: 1234, fiz: "hello world" } }

# test lists
data modify block ~ ~ ~ RecordItem.tag.custom set value [ 1 ]
data modify block ~ ~ ~ RecordItem.tag.custom set value [ 1, 2, 3 ]
data modify block ~ ~ ~ RecordItem.tag.custom set value [ a, "hello world", b ]
data modify block ~ ~ ~ RecordItem.tag.custom set value [ [1], [2.1, 2.5, 2.9], [3] ]

# test combos
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: [ 1 ] }
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: [ 1, 2, 3 ] }
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: [ a, "hello world", b ] }
data modify block ~ ~ ~ RecordItem.tag.custom set value { foo: [ { foo: true }, { bar: 1234 }, { baz: "hello world" } ] }
data modify block ~ ~ ~ RecordItem.tag.custom set value [ { foo: true }, { bar: 1234 }, { baz: "hello world" } ]

# edge case keys
execute unless data block 0 0 0 this.block.is.something{foo_bar: true}
execute unless data block 0 0 0 this.block.is.something{Foo.Bar: true}
execute unless data block 0 0 0 this.block.is.something{foo-bar: true}
execute unless data block 0 0 0 this.block.is.something[{foo.bar: true}]
