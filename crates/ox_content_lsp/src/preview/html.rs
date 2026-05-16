pub fn wrap_preview_html(title: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{title}</title>
    <style>
      :root {{
        color-scheme: light dark;
        --bg: #f6f8fb;
        --surface: #ffffff;
        --text: #0f172a;
        --muted: #475569;
        --border: #dbe2ea;
        --accent: #0f766e;
        --code-bg: #0f172a;
        --code-text: #e2e8f0;
      }}
      @media (prefers-color-scheme: dark) {{
        :root {{
          --bg: #0b1120;
          --surface: #111827;
          --text: #e5edf5;
          --muted: #94a3b8;
          --border: #243042;
          --accent: #34d399;
          --code-bg: #020617;
          --code-text: #dbeafe;
        }}
      }}
      * {{ box-sizing: border-box; }}
      body {{
        margin: 0;
        padding: 32px 20px 64px;
        background:
          radial-gradient(circle at top left, color-mix(in srgb, var(--accent) 12%, transparent), transparent 40%),
          var(--bg);
        color: var(--text);
        font: 16px/1.7 ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }}
      main {{
        max-width: 880px;
        margin: 0 auto;
        background: color-mix(in srgb, var(--surface) 94%, transparent);
        border: 1px solid var(--border);
        border-radius: 20px;
        padding: 40px;
        backdrop-filter: blur(12px);
        box-shadow: 0 20px 60px rgba(15, 23, 42, 0.12);
      }}
      h1, h2, h3, h4, h5, h6 {{ line-height: 1.2; margin: 1.8em 0 0.7em; }}
      h1:first-child {{ margin-top: 0; }}
      p, ul, ol, blockquote, pre, table {{ margin: 1rem 0; }}
      a {{ color: var(--accent); }}
      code {{
        padding: 0.18em 0.4em;
        border-radius: 0.45em;
        background: color-mix(in srgb, var(--accent) 10%, var(--surface));
        font: 0.92em/1.4 ui-monospace, SFMono-Regular, Menlo, monospace;
      }}
      pre {{
        overflow-x: auto;
        padding: 16px 18px;
        border-radius: 16px;
        background: var(--code-bg);
        color: var(--code-text);
      }}
      pre code {{ padding: 0; background: transparent; color: inherit; }}
      blockquote {{
        margin-inline: 0;
        padding: 14px 18px;
        border-left: 4px solid var(--accent);
        border-radius: 0 14px 14px 0;
        background: color-mix(in srgb, var(--accent) 10%, var(--surface));
      }}
      table {{ width: 100%; border-collapse: collapse; }}
      th, td {{ padding: 10px 12px; border: 1px solid var(--border); text-align: left; }}
      img {{ max-width: 100%; height: auto; }}
      hr {{ border: 0; border-top: 1px solid var(--border); margin: 2rem 0; }}
    </style>
  </head>
  <body>
    <main>{body}</main>
  </body>
</html>"#
    )
}
