import {parse} from '../../pkg/block_serialization_wasm.js';
import {useEffect} from "react";

function App() {

    useEffect(() => {
        const parsed = parse(`<!-- wp:image {"src":"url"} /-->
dfgdddgd
      <!-- wp:columns {"columns":3} -->
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
      <!-- /wp:columns -->` as string)
        console.log(parsed)
        const dest = document.getElementById('output')
        if (dest) dest.innerHTML = JSON.stringify(parsed)
    })

    return (
        <>
            <p>parse</p>
        </>
    )
}

export default App
