use crate::Workspace;
use crate::settings::ModuleGraphResolutionKind;
use crate::test_utils::setup_workspace_and_open_project;
use crate::workspace::{
    CodeAction, FileContent, OpenFileParams, PullActionsParams, PullDiagnosticsParams,
    UpdateSettingsParams,
};
use biome_analyze::{RuleCategories, RuleCategoriesBuilder};
use biome_configuration::analyzer::AnalyzerSelector;
use biome_configuration::{Configuration, HtmlConfiguration};
use biome_diagnostics::Severity;
use biome_fs::{BiomePath, MemoryFileSystem};
use biome_parser::prelude::ParseDiagnostic;
use camino::{Utf8Path, Utf8PathBuf};
use std::str::FromStr;

fn prepare(file: &str, content: &str) -> Vec<ParseDiagnostic> {
    let fs = MemoryFileSystem::default();
    fs.insert(Utf8PathBuf::from(file), content);

    let (workspace, project_key) = setup_workspace_and_open_project(fs, "/");

    workspace
        .update_settings(UpdateSettingsParams {
            project_key,
            workspace_directory: None,
            configuration: Configuration {
                html: Some(HtmlConfiguration {
                    experimental_full_support_enabled: Some(true.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            extended_configurations: vec![],
            module_graph_resolution_kind: ModuleGraphResolutionKind::None,
        })
        .unwrap();

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: BiomePath::new(file),
            content: FileContent::FromServer,
            document_file_source: None,
            persist_node_cache: false,
            inline_config: None,
            editor_features: None,
        })
        .unwrap();

    workspace.db_get_parse_diagnostics(Utf8Path::new(file))
}

fn assert_no_diagnostics(file: &str, content: &str) {
    let diagnostics = prepare(file, content);
    assert!(
        diagnostics.is_empty(),
        "Expected no parse errors for typed Svelte snippet, got: {diagnostics:#?}"
    );
}

fn assert_diagnostics(file: &str, content: &str) {
    let diagnostics = prepare(file, content);

    assert!(
        !diagnostics.is_empty(),
        "Expected diagnostics, but none were emitted"
    );
}

#[test]
fn snippet_svelte_ts_parsing() {
    const FILE_PATH: &str = "/project/file.svelte";
    const FILE_CONTENT: &str = r#"<script lang="ts">
	let name = $state('world');
</script>

<h1>Hello {name}!</h1>

{#snippet add(a: any, b: float)}
	{a} + {b} = {a + b}
{/snippet}

{@render add(1, 2)}
"#;

    assert_no_diagnostics(FILE_PATH, FILE_CONTENT);
}

#[test]
fn snippet_svelte_incorrect() {
    const FILE_PATH: &str = "/project/file.svelte";
    const FILE_CONTENT: &str = r#"<script lang="ts">
	let name = $state('world');
</script>

<h1>Hello {name}!</h1>

{#snippet add}
	{a} + {b} = {a + b}
{/snippet}

{@render add(1, 2)}
"#;

    assert_diagnostics(FILE_PATH, FILE_CONTENT);
}

#[test]
fn svelte_each_with_correct_method_call_key() {
    const FILE_PATH: &str = "/project/file.svelte";
    const FILE_CONTENT: &str = r#"<script lang="ts">
    const numbers = [1, 2, 3, 4];
</script>
{#each numbers as number, index (number.toString())}
  <p>{number}</p>
{/each}
"#;

    assert_no_diagnostics(FILE_PATH, FILE_CONTENT);
}

#[test]
fn svelte_each_with_incorrect_method_call_key() {
    const FILE_PATH: &str = "/project/file.svelte";
    const FILE_CONTENT: &str = r#"<script lang="ts">
    const numbers = [1, 2, 3, 4];
</script>
{#each numbers as number, index (number.toString(})}
  <p>{number}</p>
{/each}
"#;

    assert_diagnostics(FILE_PATH, FILE_CONTENT);
}

#[test]
fn vue_v_on_accepts_inline_statements_and_expression_handlers() {
    // Mirrors Vue's v-on handler cases:
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L148-L161
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L163-L179
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L181-L197
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L260-L273
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L305-L320
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L347-L370
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L372-L385
    const FILE_PATH: &str = "/project/file.vue";
    const FILE_CONTENT: &str = r#"<script setup>
let counter = 0;
function foo() {}
function bar() {}
const a = { b: foo };
const c = "b";
</script>
<template>
  <button @click="counter++; counter++;"></button>
  <button @click="foo();bar()"></button>
  <button @click="
    counter++;
    counter++;
  "></button>
  <button @click="foo"></button>
  <button @click="a['b' + c]"></button>
  <button @click="$event => foo($event)"></button>
  <button @click="async $event => foo($event)"></button>
  <button @click="function($event) { foo($event) }"></button>
  <button v-on:click="counter++; counter++;"></button>
  <button @click="if (counter) foo(); else bar()"></button>
  <button @click="const next = counter + 1; foo(next)"></button>
  <button @click="if (counter) { foo() }"></button>
</template>
"#;

    assert_no_diagnostics(FILE_PATH, FILE_CONTENT);
}

#[test]
fn vue_v_on_accepts_typescript_function_handler() {
    // Mirrors Vue's TypeScript function-expression v-on handler case:
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L275-L303
    const FILE_PATH: &str = "/project/file.vue";
    const FILE_CONTENT: &str = r#"<script lang="ts">
let counter = 0;
</script>
<template>
  <button @click="($event: MouseEvent) => counter++"></button>
</template>
"#;

    assert_no_diagnostics(FILE_PATH, FILE_CONTENT);
}

#[test]
fn vue_v_on_rejects_function_handler_with_trailing_statement() {
    // Enforces Vue's split between function-expression handlers and inline
    // statement handlers by combining these upstream cases:
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L260-L273
    // https://github.com/vuejs/core/blob/86ad0764fd9f7b01cef75b4fc941b03419306bf8/packages/compiler-core/__tests__/transforms/vOn.spec.ts#L163-L179
    const FILE_PATH: &str = "/project/file.vue";
    const FILE_CONTENT: &str = r#"<template>
  <button @click="() => {counter++}; counter++;"></button>
</template>
"#;

    assert_diagnostics(FILE_PATH, FILE_CONTENT);
}

#[test]
fn vue_non_event_directives_stay_expression_only() {
    const FILE_PATH: &str = "/project/file.vue";
    const FILE_CONTENT: &str = r#"<script setup>
const isActive = true;
const foo = "foo";
const bar = "bar";
const duration = 100;
</script>
<template>
  <div :class="{ active: isActive }"></div>
  <div v-bind="{ id: foo, class: bar }"></div>
  <div>{{ { duration } }}</div>
</template>
"#;

    assert_no_diagnostics(FILE_PATH, FILE_CONTENT);
}

#[test]
fn html_style_attribute_is_parsed_as_css() {
    assert_diagnostics("/project/file.html", r#"<div style="color: rgb("></div>"#);
}

#[test]
fn html_style_attribute_rejects_qualified_rules() {
    assert_diagnostics(
        "/project/file.html",
        r#"<div style="a { color: red }"></div>"#,
    );
}

#[test]
fn html_style_attribute_reports_invalid_declaration() {
    for content in [
        r#"<div style="9fill-opacity:1;"></div>"#,
        r#"<div style="color(display-p3 0.4549 0.8745 0.5451);fill-opacity:1;"></div>"#,
    ] {
        assert_diagnostics("/project/file.html", content);
    }
}

#[test]
fn html_style_attribute_unescapes_entities() {
    // https://github.com/biomejs/biome/issues/12349
    assert_no_diagnostics(
        "/project/file.html",
        r#"<p style="font-family:RotisSansCyrillic, &quot;Rotis Sans Cyrillic&quot;, sans-serif; font-size:10px;">test</p>"#,
    );
    assert_no_diagnostics(
        "/project/file.html",
        r#"<p style="font-family:RotisSansCyrillic, &#34;Rotis Sans Cyrillic&#34;, sans-serif; font-size:10px;">test</p>"#,
    );
    assert_no_diagnostics(
        "/project/file.html",
        r#"<p style="font-family:RotisSansCyrillic, &#x22;Rotis Sans Cyrillic&#x22;, sans-serif; font-size:10px;">test</p>"#,
    );
}

#[test]
fn svg_style_attribute_unescapes_entities() {
    // https://github.com/biomejs/biome/issues/12349
    assert_no_diagnostics(
        "/project/file.svg",
        r#"<text style="font-family:RotisSansCyrillic, &quot;Rotis Sans Cyrillic&quot;, sans-serif; font-size:10px;">test</text>"#,
    );
}

#[test]
fn html_style_attribute_reports_invalid_declaration_after_unescape() {
    assert_diagnostics(
        "/project/file.html",
        r#"<div style="&quot;color: red"></div>"#,
    );
}

#[test]
fn style_attribute_entities_follow_attribute_context_rule() {
    // A bare named entity without `;` followed by an alphanumeric
    // character is left literal in attribute values.
    assert_eq!(super::decode_style_attribute_entities("&ampX"), "&ampX");
    assert_eq!(
        super::decode_style_attribute_entities("&quot;&#34;&#x22;"),
        "\"\"\""
    );
    // Decoding is a single pass: `&amp;quot;` decodes once to `&quot;`,
    // not twice to `"`.
    assert_eq!(
        super::decode_style_attribute_entities("&amp;quot;"),
        "&quot;"
    );
    assert_eq!(super::decode_style_attribute_entities("&amp;amp;"), "&amp;");
}

#[test]
fn html_style_attribute_with_encoded_colon_is_parsed_as_css() {
    // The detector runs on the decoded value, so an entity-encoded
    // colon still marks the attribute as CSS.
    assert_no_diagnostics(
        "/project/file.html",
        r#"<div style="color&#58; red"></div>"#,
    );
    assert_diagnostics(
        "/project/file.html",
        r#"<div style="color&#58; rgb("></div>"#,
    );
}

#[test]
fn html_non_style_attribute_is_not_parsed_as_css() {
    assert_no_diagnostics(
        "/project/file.html",
        r#"<div data-style="color: rgb("></div>"#,
    );
}

fn pull_all_actions(file: &str, content: &str) -> Vec<CodeAction> {
    let fs = MemoryFileSystem::default();
    fs.insert(Utf8PathBuf::from(file), content);

    let (workspace, project_key) = setup_workspace_and_open_project(fs, "/");

    workspace
        .update_settings(UpdateSettingsParams {
            project_key,
            workspace_directory: None,
            configuration: Configuration {
                html: Some(HtmlConfiguration {
                    experimental_full_support_enabled: Some(true.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            extended_configurations: vec![],
            module_graph_resolution_kind: ModuleGraphResolutionKind::None,
        })
        .unwrap();

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: BiomePath::new(file),
            content: FileContent::FromServer,
            document_file_source: None,
            persist_node_cache: false,
            inline_config: None,
            editor_features: None,
        })
        .unwrap();

    workspace
        .pull_actions(PullActionsParams {
            project_key,
            path: BiomePath::new(file),
            range: None,
            suppression_reason: None,
            only: vec![],
            skip: vec![],
            enabled_rules: vec![],
            categories: RuleCategories::all(),
            inline_config: None,
            compute_actions: true,
        })
        .unwrap()
        .actions
}

#[test]
fn html_style_attribute_plain_snippet_offers_code_actions() {
    // Control case: without entities the snippet is not decoded, so the
    // `noImportantStyles` fix is offered as a code action.
    let actions = pull_all_actions(
        "/project/file.html",
        r#"<div style="font-family: X; color: red !important"></div>"#,
    );
    assert!(
        actions.iter().any(|action| action
            .rule_name
            .as_ref()
            .is_some_and(|(_, name)| name == "noImportantStyles")),
        "Expected a noImportantStyles code action, got: {actions:#?}"
    );
}

#[test]
fn html_style_attribute_decoded_snippet_offers_no_code_actions() {
    // The snippet was decoded before parsing, so its tree positions may
    // not match host positions after the `&quot;` entities. No code
    // action may be offered: applying one would edit the wrong bytes.
    let actions = pull_all_actions(
        "/project/file.html",
        r#"<div style="font-family: &quot;X&quot;; color: red !important"></div>"#,
    );
    assert!(
        actions.is_empty(),
        "Expected no code actions for a decoded snippet, got: {actions:#?}"
    );
}

#[test]
fn html_style_element_content_is_not_decoded() {
    // `<style>` element content is raw text: an entity there stays
    // literal, so it still fails CSS parsing.
    assert_diagnostics("/project/file.html", r#"<style>&quot;color: red</style>"#);
}

fn pull_lint_diagnostics_with_code_fix(file: &str, content: &str) -> serde_json::Value {
    let fs = MemoryFileSystem::default();
    fs.insert(Utf8PathBuf::from(file), content);

    let (workspace, project_key) = setup_workspace_and_open_project(fs, "/");

    workspace
        .update_settings(UpdateSettingsParams {
            project_key,
            workspace_directory: None,
            configuration: Configuration {
                html: Some(HtmlConfiguration {
                    experimental_full_support_enabled: Some(true.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            extended_configurations: vec![],
            module_graph_resolution_kind: ModuleGraphResolutionKind::None,
        })
        .unwrap();

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: BiomePath::new(file),
            content: FileContent::FromServer,
            document_file_source: None,
            persist_node_cache: false,
            inline_config: None,
            editor_features: None,
        })
        .unwrap();

    let no_important_styles =
        AnalyzerSelector::from_str("lint/complexity/noImportantStyles").unwrap();
    let diagnostics = workspace
        .pull_diagnostics(PullDiagnosticsParams {
            project_key,
            path: BiomePath::new(file),
            categories: RuleCategoriesBuilder::default().with_lint().build(),
            only: vec![no_important_styles],
            skip: vec![],
            enabled_rules: vec![no_important_styles],
            include_code_fix: true,
            inline_config: None,
            max_diagnostics: None,
            diagnostic_level: Severity::Hint,
            enforce_assist: false,
        })
        .unwrap();
    serde_json::to_value(&diagnostics.diagnostics).unwrap()
}

#[test]
fn html_style_attribute_plain_snippet_lint_offers_code_suggestion() {
    // Control case: without entities the snippet is not decoded, so the
    // `noImportantStyles` diagnostic carries its fix suggestion.
    let json = pull_lint_diagnostics_with_code_fix(
        "/project/file.html",
        r#"<div style="font-family: X; color: red !important"></div>"#,
    );
    let text = json.to_string();
    assert!(
        text.contains("noImportantStyles"),
        "Expected a noImportantStyles diagnostic, got: {text}"
    );
    assert!(
        text.contains("codeSuggestion"),
        "Expected a code suggestion for the plain snippet, got: {text}"
    );
}

#[test]
fn html_style_attribute_decoded_snippet_lint_offers_no_code_suggestion() {
    // The snippet was decoded before parsing, so its tree positions may
    // not match host positions after the `&quot;` entities. The lint
    // diagnostic is still reported, but it must carry no fix suggestion:
    // applying one would edit the wrong bytes.
    let json = pull_lint_diagnostics_with_code_fix(
        "/project/file.html",
        r#"<div style="font-family: &quot;X&quot;; color: red !important"></div>"#,
    );
    let text = json.to_string();
    assert!(
        text.contains("noImportantStyles"),
        "Expected a noImportantStyles diagnostic, got: {text}"
    );
    assert!(
        !text.contains("codeSuggestion"),
        "Expected no code suggestion for a decoded snippet, got: {text}"
    );
}

#[test]
fn html_style_attribute_name_match_is_case_insensitive() {
    // The `style` attribute name matches case-insensitively.
    assert_no_diagnostics(
        "/project/file.html",
        r#"<p STYLE="font-family:RotisSansCyrillic, &quot;Rotis Sans Cyrillic&quot;, sans-serif; font-size:10px;">test</p>"#,
    );
}

#[test]
fn html_style_attribute_single_quoted_unescapes_entities() {
    assert_no_diagnostics(
        "/project/file.html",
        r#"<p style='font-family:RotisSansCyrillic, &quot;Rotis Sans Cyrillic&quot;, sans-serif; font-size:10px;'>test</p>"#,
    );
}

#[test]
fn vue_style_attribute_unescapes_entities() {
    // Plain `style` attributes in Vue files go through the same
    // HTML embed path as `.html` files.
    assert_no_diagnostics(
        "/project/file.vue",
        r#"<template><p style="font-family:RotisSansCyrillic, &quot;Rotis Sans Cyrillic&quot;, sans-serif; font-size:10px;">test</p></template>"#,
    );
}
