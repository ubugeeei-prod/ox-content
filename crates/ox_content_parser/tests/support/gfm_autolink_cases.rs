//! Autolink inputs and the HTML they rendered to before the candidate scan
//! was rewritten, with the renderer's external-link attributes left out.
//!
//! Each group sits next to a rule the rewrite touched, so a changed verdict
//! shows up as the one line that moved.

pub const CASES: &[(&str, &str)] = &[
    // A `)` is trimmed only while the link closes more than it opens.
    ("www.a.b)", r#"<p><a href="http://www.a.b">www.a.b</a>)</p>"#),
    ("(www.a.b)", r#"<p>(<a href="http://www.a.b">www.a.b</a>)</p>"#),
    ("www.a.b/(x)", r#"<p><a href="http://www.a.b/(x)">www.a.b/(x)</a></p>"#),
    ("www.a.b/(x))", r#"<p><a href="http://www.a.b/(x)">www.a.b/(x)</a>)</p>"#),
    ("www.a.b/((x)", r#"<p><a href="http://www.a.b/((x)">www.a.b/((x)</a></p>"#),
    ("www.a.b/(x))).", r#"<p><a href="http://www.a.b/(x)">www.a.b/(x)</a>)).</p>"#),
    ("www.a.b/x)(", r#"<p><a href="http://www.a.b/x)(">www.a.b/x)(</a></p>"#),
    ("www.a.b/x)()", r#"<p><a href="http://www.a.b/x)(">www.a.b/x)(</a>)</p>"#),
    ("www.a.b/))((", r#"<p><a href="http://www.a.b/))((">www.a.b/))((</a></p>"#),
    ("www.a.b/(()))", r#"<p><a href="http://www.a.b/(())">www.a.b/(())</a>)</p>"#),
    ("www.a.b/x).", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>).</p>"#),
    ("www.a.b/x.)", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>.)</p>"#),
    ("www.a.b/x);", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>);</p>"#),
    // An entity-like `&name;` suffix goes whole; anything less loses the `;`.
    ("www.a.b/x&foo;", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>&amp;foo;</p>"#),
    ("www.a.b/x&foo;&bar;", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>&amp;foo;&amp;bar;</p>"#),
    ("www.a.b/x&;", r#"<p><a href="http://www.a.b/x&amp;">www.a.b/x&amp;</a>;</p>"#),
    ("www.a.b/x&foo;)", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>&amp;foo;)</p>"#),
    (
        "www.a.b/(x&foo;)",
        r#"<p><a href="http://www.a.b/(x&amp;foo;)">www.a.b/(x&amp;foo;)</a></p>"#,
    ),
    ("www.a.b/x&f-o;", r#"<p><a href="http://www.a.b/x&amp;f-o">www.a.b/x&amp;f-o</a>;</p>"#),
    ("www.a.b/x;", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>;</p>"#),
    ("www.a.b/x;;;", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>;;;</p>"#),
    ("www.a.b/a;b&c;", r#"<p><a href="http://www.a.b/a;b">www.a.b/a;b</a>&amp;c;</p>"#),
    ("www.a.b/&foo;", r#"<p><a href="http://www.a.b/">www.a.b/</a>&amp;foo;</p>"#),
    ("www.a.b&foo;", r#"<p><a href="http://www.a.b">www.a.b</a>&amp;foo;</p>"#),
    ("www.a.b/x&amp;amp;", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>&amp;amp;</p>"#),
    ("www.a.b/x&amp;", r#"<p><a href="http://www.a.b/x&amp;">www.a.b/x&amp;</a></p>"#),
    ("www.a.b/x&foo;;", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>&amp;foo;;</p>"#),
    ("www.a.b/x&foo", r#"<p><a href="http://www.a.b/x&amp;foo">www.a.b/x&amp;foo</a></p>"#),
    ("www.a.b/x&foo;.", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>&amp;foo;.</p>"#),
    (
        "www.a.b/x&a;&;",
        r#"<p><a href="http://www.a.b/x&amp;a;&amp;">www.a.b/x&amp;a;&amp;</a>;</p>"#,
    ),
    ("www.a.b/x?!.,:'", r#"<p><a href="http://www.a.b/x">www.a.b/x</a>?!.,:&#39;</p>"#),
    // A host needs two segments, the last two non-empty and without `_`.
    ("www.a.b_", "<p>www.a.b_</p>"),
    ("www.a_b.c", "<p>www.a_b.c</p>"),
    ("www.a_b.c.d", r#"<p><a href="http://www.a_b.c.d">www.a_b.c.d</a></p>"#),
    ("www.a.b.", r#"<p><a href="http://www.a.b">www.a.b</a>.</p>"#),
    ("www.a..b", "<p>www.a..b</p>"),
    ("www.a", "<p>www.a</p>"),
    ("www..a", "<p>www..a</p>"),
    ("www.a.b_c.d", "<p>www.a.b_c.d</p>"),
    // Several `www.` in one run of domain bytes share the run's tail.
    ("www.a.b_www.c.d", r#"<p><a href="http://www.a.b_www.c.d">www.a.b_www.c.d</a></p>"#),
    ("xwww.a.b_www.c.d", r#"<p>xwww.a.b_<a href="http://www.c.d">www.c.d</a></p>"#),
    ("www.a_www.b", "<p>www.a_www.b</p>"),
    ("www.a.b_www.c_d.e", "<p>www.a.b_www.c_d.e</p>"),
    ("www.a.b_www.c.d_", "<p>www.a.b_www.c.d_</p>"),
    // An email that starts on the same byte as a `www.` URL wins.
    ("www.a.b@c.d", r#"<p><a href="mailto:www.a.b@c.d">www.a.b@c.d</a></p>"#),
    (
        "www.a.b@c.d(www.e.f@g.h",
        r#"<p><a href="mailto:www.a.b@c.d">www.a.b@c.d</a>(<a href="mailto:www.e.f@g.h">www.e.f@g.h</a></p>"#,
    ),
    (
        "www.a.b@c.d www.e.f",
        r#"<p><a href="mailto:www.a.b@c.d">www.a.b@c.d</a> <a href="http://www.e.f">www.e.f</a></p>"#,
    ),
    // What follows a link is read as text of its own: it starts on a
    // boundary, and a local part cannot reach back across it.
    ("a@b.c+d@e.f", r#"<p><a href="mailto:a@b.c">a@b.c</a><a href="mailto:+d@e.f">+d@e.f</a></p>"#),
    (
        "a@b.c.+d@e.f",
        r#"<p><a href="mailto:a@b.c">a@b.c</a><a href="mailto:.+d@e.f">.+d@e.f</a></p>"#,
    ),
    ("a@b.c_d@e.f", r#"<p><a href="mailto:a@b.c_d">a@b.c_d</a>@e.f</p>"#),
    ("www.a.b<www.c.d", r#"<p><a href="http://www.a.b">www.a.b</a>&lt;www.c.d</p>"#),
    (
        "http://a.b,http://c.d",
        r#"<p><a href="http://a.b,http://c.d">http://a.b,http://c.d</a></p>"#,
    ),
    (
        "http://a.b/日本語。www.c.d",
        r#"<p><a href="http://a.b/日本語">http://a.b/日本語</a>。www.c.d</p>"#,
    ),
    // Emails.
    ("a@b.c-", "<p>a@b.c-</p>"),
    ("a@b.c_", "<p>a@b.c_</p>"),
    ("a@b", "<p>a@b</p>"),
    ("a@@b.c", "<p>a@@b.c</p>"),
    ("x,a@b.c", "<p>x,a@b.c</p>"),
    ("a.@b.c", r#"<p><a href="mailto:a.@b.c">a.@b.c</a></p>"#),
    ("+a@b.c", r#"<p><a href="mailto:+a@b.c">+a@b.c</a></p>"#),
    ("a@b.c.", r#"<p><a href="mailto:a@b.c">a@b.c</a>.</p>"#),
    // Links among other nodes keep their order.
    (
        "a@b.c http://d.e www.f.g",
        r#"<p><a href="mailto:a@b.c">a@b.c</a> <a href="http://d.e">http://d.e</a> <a href="http://www.f.g">www.f.g</a></p>"#,
    ),
    (
        "www.a.b *x* a@b.c `c` http://d.e",
        r#"<p><a href="http://www.a.b">www.a.b</a> <em>x</em> <a href="mailto:a@b.c">a@b.c</a> <code>c</code> <a href="http://d.e">http://d.e</a></p>"#,
    ),
    (
        "**www.a.b** ~~a@b.c~~ *http://d.e*",
        r#"<p><strong><a href="http://www.a.b">www.a.b</a></strong> <del><a href="mailto:a@b.c">a@b.c</a></del> <em><a href="http://d.e">http://d.e</a></em></p>"#,
    ),
    (
        "see www.a.b, a@b.co and http://c.d/e.",
        r#"<p>see <a href="http://www.a.b">www.a.b</a>, <a href="mailto:a@b.co">a@b.co</a> and <a href="http://c.d/e">http://c.d/e</a>.</p>"#,
    ),
    (
        "ftp://a.b https://c.d/e?f=g&amp;h=i.",
        r#"<p><a href="ftp://a.b">ftp://a.b</a> <a href="https://c.d/e?f=g&amp;h=i">https://c.d/e?f=g&amp;h=i</a>.</p>"#,
    ),
    ("&amp; www.a.b x", r#"<p>&amp; <a href="http://www.a.b">www.a.b</a> x</p>"#),
];

/// Inputs and the tree they parsed to, for what HTML does not show.
pub const TREES: &[(&str, &str)] = &[
    // A `scheme://` host is held to the same rule as a `www.` one.
    (
        "x http://a_b.c http://a.b_c ftp://d",
        "Document [0..35]\n  Paragraph [0..35]\n    \
         Text \"x http://a_b.c http://a.b_c ftp://d\" [0..35]\n",
    ),
    // Links on both sides of another node, the first one behind a rejected
    // candidate.
    (
        "xhttp://a.b http://c.d *e* www.f.g",
        "Document [0..34]\n  Paragraph [0..34]\n    \
         Text \"xhttp://a.b \" [0..12]\n    \
         Link url=\"http://c.d\" title=None [12..22]\n      Text \"http://c.d\" [12..22]\n    \
         Text \" \" [22..23]\n    \
         Emphasis [23..26]\n      Text \"e\" [24..25]\n    \
         Text \" \" [26..27]\n    \
         Link url=\"http://www.f.g\" title=None [27..34]\n      Text \"www.f.g\" [27..34]\n",
    ),
    // Three links out of one text node: text before, between and after.
    (
        "see www.a.b, a@b.co and https://c.d/e.",
        "Document [0..38]\n  Paragraph [0..38]\n    \
         Text \"see \" [0..4]\n    \
         Link url=\"http://www.a.b\" title=None [4..11]\n      Text \"www.a.b\" [4..11]\n    \
         Text \", \" [11..13]\n    \
         Link url=\"mailto:a@b.co\" title=None [13..19]\n      Text \"a@b.co\" [13..19]\n    \
         Text \" and \" [19..24]\n    \
         Link url=\"https://c.d/e\" title=None [24..37]\n      Text \"https://c.d/e\" [24..37]\n    \
         Text \".\" [37..38]\n",
    ),
    // Spans count bytes of the decoded value from the node's start, and the
    // last piece keeps the node's own end.
    (
        "&amp; www.a.b x&foo; y",
        "Document [0..22]\n  Paragraph [0..22]\n    \
         Text \"& \" [0..2]\n    \
         Link url=\"http://www.a.b\" title=None [2..9]\n      Text \"www.a.b\" [2..9]\n    \
         Text \" x&foo; y\" [9..22]\n",
    ),
];
