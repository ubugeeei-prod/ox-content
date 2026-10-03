# Benchmark dependency patches

`astro@7.3.3.patch` removes remote-image caching from the two local Astro
benchmark fixtures. Their Markdown and Vue pages use no remote images, so this
does not change the measured rendering workload. Optional remote images still
fetch normally and preserve ETag / Last-Modified revalidation, but expire immediately.

The matching pnpm override removes `http-cache-semantics`, which has an unpatched
high-severity advisory (GHSA-ch52-4w7c-c8xp). Regression tests cover response bytes,
private responses, conditional 304 responses, failures, and redirect validation.
Keep the exact Astro version pin until the patch can be removed or regenerated.
