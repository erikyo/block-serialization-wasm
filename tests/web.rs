//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use wasm_bindgen_test::*;
use block_serialization_wasm::parse;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse() {
    let _result = parse(r#"<!-- wp:columns {"columns":3} -->
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

// let result: serde_json::Value = serde_wasm_bindgen::from_value(result).unwrap();

// assert_eq!(result["blockName"], "core/columns");
// assert_eq!(result["attrs"]["columns"], 3);
// assert_eq!(result["innerHTML"], "\n<div class=\"wp-block-columns has-3-columns\">\n\n\n\n</div>\n");

// let inner_blocks: &Vec<serde_json::Value> = result["innerBlocks"].as_array().unwrap();

// assert_eq!(inner_blocks[0]["blockName"], "core/column");
// assert_eq!(inner_blocks[0]["attrs"], json!(null));
// assert_eq!(inner_blocks[0]["innerHTML"], "\n<div class=\"wp-block-column\">\n\n\n\n</div>\n");
}
