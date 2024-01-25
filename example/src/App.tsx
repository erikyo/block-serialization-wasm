import { parse } from '@wordpress/block-serialization-default-parser';
import {parse as parseWasm} from '../../pkg/block_serialization_wasm.js';
import {useEffect} from "react";

function App() {

    useEffect(() => {
        const original = `
      <!-- wp:columns {"columns":3} -->
      <div class="wp-block-columns has-3-columns">
        <!-- wp:column -->
        <div class="wp-block-column">
          <!-- wp:paragraph -->
          <p>Left</p>
          <!-- wp:image {"src":"url"} /-->
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
      <!-- /wp:columns -->` as string

        // js
        let timestart = performance.now()
        const parsed = parse(original)
        console.log("Parse JS time = " , performance.now() - timestart)
        const dest = document.getElementById('output-def')?.querySelector('code')
        if (dest) dest.innerHTML = JSON.stringify(parsed, null, 2)
        console.table(parsed)

        // wasm
        timestart = performance.now()
        const parsedWasm = parseWasm(original)
        console.log("Parse WASM time = " ,performance.now() - timestart)
        const destWasm = document.getElementById('output-wasm')?.querySelector('code')
        if (destWasm) destWasm.innerHTML = JSON.stringify( parsedWasm, null, 2)
        console.table(parsedWasm)
    }, [])

    return null

}

export default App
