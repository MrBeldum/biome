---
"@biomejs/biome": patch
---

Fixed [#12349](https://github.com/biomejs/biome/issues/12349). HTML entities in `style` attribute values are now unescaped before the CSS content is parsed, so a value such as `font-family: RotisSansCyrillic, &quot;Rotis Sans Cyrillic&quot;, sans-serif` no longer reports spurious CSS parse diagnostics. This also applies to `style` attributes in SVG files.

As a consequence, quick fixes and code actions are not offered for `style` attributes whose value changed when entities were unescaped, and diagnostic positions after an entity in such an attribute may be approximate.
