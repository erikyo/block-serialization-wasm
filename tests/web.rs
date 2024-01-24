//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use serde_json::json;
use wasm_bindgen_test::*;
use block_serialization_wasm::parse;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse() {
    let result = parse(r#"<!-- wp:columns {"columns":3} -->
<div class="wp-block-columns has-3-columns">
	<!-- wp:column -->
	<div class="wp-block-column">
		<!-- wp:paragraph -->
		<p>Left</p>
		<!-- /wp:paragraph -->
	</div>
	<!-- /wp:column -->

	<!-- wp:column -->
	<div class="wp-block-column">
		<!-- wp:paragraph -->
		<p><strong>Middle</strong></p>
		<!-- /wp:paragraph -->
	</div>
	<!-- /wp:column -->

	<!-- wp:column -->
	<div class="wp-block-column"></div>
	<!-- /wp:column -->
</div>
<!-- /wp:columns -->"#);


    assert_eq!(result, json!([
		{
			blockName: "core/columns",
			attrs: {
				columns: 3,
			},
			innerBlocks: [
				{
					blockName: "core/column",
					attrs: null,
					innerBlocks: [
						{
							blockName: "core/paragraph",
							attrs: null,
							innerBlocks: [],
							innerHTML: "\n<p>Left</p>\n",
						},
					],
					innerHTML: "\n<div class=\"wp-block-column\"></div>\n",
				},
				{
					blockName: "core/column",
					attrs: null,
					innerBlocks: [
						{
							blockName: "core/paragraph",
							attrs: null,
							innerBlocks: [],
							innerHTML: "\n<p><strong>Middle</strong></p>\n",
						},
					],
					innerHTML: "\n<div class=\"wp-block-column\"></div>\n",
				},
				{
					blockName: "core/column",
					attrs: null,
					innerBlocks: [],
					innerHTML: "\n<div class=\"wp-block-column\"></div>\n",
				},
			],
			innerHTML:
				"\n<div class=\"wp-block-columns has-3-columns\">\n\n\n\n</div>\n",
		},
	]));
}
