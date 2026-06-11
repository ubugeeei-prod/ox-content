#[path = "support/snapshot.rs"]
mod snapshot_support;

use ox_content_parser::ParserOptions;
use ox_content_renderer::HtmlRendererOptions;
use snapshot_support::check;

#[test]
fn html_kitchen_sink_document() {
    check(
        "kitchen_sink_document",
        concat!(
            "# Project\n",
            "\n",
            "An intro with **bold**, *it*, `code`, ~~old~~, [link](https://example.com), and a hard\\\nbreak.\n",
            "\n",
            "## Install\n",
            "\n",
            "```bash\n",
            "npm install ox-content\n",
            "```\n",
            "\n",
            "## Steps\n",
            "\n",
            "1. one\n",
            "2. two\n",
            "   - nested\n",
            "3. three\n",
            "\n",
            "> Quoted tip.\n",
            "\n",
            "| col | val |\n",
            "| :-- | --: |\n",
            "| a   | 1   |\n",
            "| b   | 2   |\n",
            "\n",
            "![diagram](./img.png \"Title\")\n",
        ),
        ParserOptions::gfm(),
        HtmlRendererOptions::default(),
    );
}
