---
"@biomejs/biome": patch
---

Fixed [#12335](https://github.com/biomejs/biome/issues/12335): the nursery rule [`useStrictBooleanExpressions`](https://biomejs.dev/linter/rules/use-strict-boolean-expressions/) no longer reports a diagnostic when a destructured value has a default initializer, such as `const { someField = false } = options ?? {}`. A default initializer only applies when the destructured value is `undefined`, so the resulting type now excludes `undefined`.
