---
"@biomejs/biome": patch
---

Fixed [#12373](https://github.com/biomejs/biome/issues/12373): `biome migrate eslint` no longer maps the `one-var` rule to [`useSingleVarDeclarator`](https://biomejs.dev/linter/rules/use-single-var-declarator/) unless the rule is configured with `never`. The `one-var` rule now migrates only when its option requires every declaration to use a single declarator.
