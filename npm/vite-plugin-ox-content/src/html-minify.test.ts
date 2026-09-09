import { describe, expect, it } from "vite-plus/test";
import { createHtmlMinifyContext, minifyHtmlOutput } from "./html-minify";

describe("minifyHtmlOutput", () => {
  it("minifies final HTML while preserving whitespace-sensitive content", async () => {
    const html = `<!DOCTYPE html>
<html>
  <head>
    <!-- build comment -->
    <script type="application/ld+json">
      { "@context": "https://schema.org", "name": "Ada" }
    </script>
    <script>
      const answer = 1 + 2;
      window.__answer = answer;
    </script>
    <script type="module">
      const label = "module";
      console.log(label);
    </script>
    <style>
      .badge { color: red; margin: 0px; }
    </style>
  </head>
  <body>
    <p>Alpha <strong>Beta</strong> Gamma</p>
    <pre> keep   spaces
  and newlines </pre>
    <textarea> keep   textarea spaces </textarea>
    <svg viewBox="0 0 10 10"><text>Hi</text></svg>
    <math><mi>x</mi><mo>=</mo><mn>1</mn></math>
    <!--$--><div data-hk="1">solid</div><!--/-->
  </body>
</html>`;

    const minified = await minifyHtmlOutput(html);

    expect(minified).toContain("<!doctype html>");
    expect(minified).not.toContain("build comment");
    expect(minified).toContain("<p>Alpha <strong>Beta</strong> Gamma</p>");
    expect(minified).toContain(`<pre> keep   spaces
  and newlines </pre>`);
    expect(minified).toContain("<textarea> keep   textarea spaces </textarea>");
    expect(minified).toContain('<svg viewBox="0 0 10 10"><text>Hi</text></svg>');
    expect(minified).toContain("<math><mi>x</mi><mo>=</mo><mn>1</mn></math>");
    expect(minified).toContain("window.__answer=3");
    expect(minified).toContain("console.log(label)");
    expect(minified).toContain("<style>.badge{color:red;margin:0}</style>");
    expect(minified).toContain('<!--$--><div data-hk="1">solid</div><!--/-->');

    const jsonLd = minified.match(/<script type="application\/ld\+json">(?<json>.*?)<\/script>/u)
      ?.groups?.json;
    expect(JSON.parse(jsonLd ?? "")).toEqual({
      "@context": "https://schema.org",
      name: "Ada",
    });
  });

  it("fails instead of publishing when inline JavaScript cannot be minified", async () => {
    await expect(
      minifyHtmlOutput("<!doctype html><script>function () {</script>"),
    ).rejects.toThrow();
  });

  it("reuses identical inline script and style minification within a build context", async () => {
    const context = createHtmlMinifyContext();
    const script = "const answer = 1 + 2; window.__answer = answer;";
    const style = ".badge { color: red; margin: 0px; }";

    const [first, second] = await Promise.all([
      minifyHtmlOutput(`<style>${style}</style><script>${script}</script>`, context),
      minifyHtmlOutput(`<main><style>${style}</style><script>${script}</script></main>`, context),
    ]);

    expect(first).toBe(
      "<style>.badge{color:red;margin:0}</style><script>const answer=3;window.__answer=3</script>",
    );
    expect(second).toBe(
      "<main><style>.badge{color:red;margin:0}</style><script>const answer=3;window.__answer=3</script></main>",
    );
    expect(context.scripts.size).toBe(1);
    expect(context.styles.size).toBe(1);
  });
});
