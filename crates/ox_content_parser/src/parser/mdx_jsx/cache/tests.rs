#![allow(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

use ox_content_allocator::Allocator;

use super::{SHORTCUTS, ScanScope, Scanner, Shortcuts};

/// No shortcuts: every scan goes through the cache, however short.
const ALWAYS_CACHED: Shortcuts = Shortcuts { slice_len: 0, nearby: 0, brace_walk: 0 };

/// Shortcuts cut so short that most tags and groups run past them.
const CUT_SHORT: [Shortcuts; 3] = [
    Shortcuts { slice_len: 0, nearby: 3, brace_walk: 2 },
    Shortcuts { slice_len: 0, nearby: 7, brace_walk: 5 },
    Shortcuts { slice_len: 6, nearby: 12, brace_walk: 9 },
];

/// Short sources that between them reach every branch of the cache: tags
/// that close, cross, nest and never close; closers hidden in attribute
/// values, brace groups, strings, comments and code spans; and brace groups
/// in each of those states themselves.
const SOURCES: &[&str] = &[
    "<A>x</A>",
    "<A><A>x</A></A> <A>y</A>",
    "<A><B></A></B>",
    "<A><A><A></A>",
    "<A></B><A></A></A>",
    "<></><>x<></></>",
    "<A.b>x</A.b></A>",
    "<A b={<A>}>x</A>}></A>",
    "<A b=\"</A>\">x</A>\"></A>",
    "<A {...a}>{</A>}</A>",
    "<A>{</A></A>",
    "<A>{\"}</A>\"}</A>",
    "<A>{'</A>}</A>'}",
    "<A>{/*</A>*/}</A>",
    "<A>{//</A>}\n}</A>",
    "<A>`</A>`</A>",
    "<A>`</A>``</A>`",
    "<A>``</A>`</A>``</A>",
    "<A>```</A>`` `</A>",
    "{{a}{b}{c{d}}}{e",
    "{{{{}}",
    "{\"{\"{\"}\"}\"}",
    "{'\\'}'}{`}`}",
    "{/*{*/}{//}\n}",
    "{\"\\\"{\\\"}\"}{\"",
    "<A b={{c:{d}}} e='}' {...f}/>}",
    "<A b={\"}\"}>x</A><A b={\"}>",
    "a <A>[x](<A>)</A> ~~<A>~~</A>",
    "<A><B><C></C></B></A></B>",
    "<A>\n<B>\n</B>\n</A>\n<A>\n",
    "<A b=\"\\\"<A b=\"\\\"\">x</A>",
    "<A/><A></A/></A>",
    "</A><A></ A></A >",
    "<A><a><A></a></A></A>",
    "<A {<A>}>{<A>}</A> {<A",
    "<A>é{é}</A>é<A é",
    "<A b c={d} e=\"f\" g='h' {...i} j />k</A>",
    // A later opener that closes exactly where an earlier walk gave up.
    "{\"{\"}",
    "{'{'}x",
    "{\"{\"\"}",
    "{/*{/*/}",
    "{/* { /*/ } */}",
    // A code-span opener that only closes once it is retried shorter.
    "<A>```a`</A>``",
    // A group whose `}` is the last one in the source.
    "<A b={}/>{}",
];

/// Everything the parser can ask about `at`, as text, from a scan of
/// `content` alone: the element with its attributes, and the brace group.
fn direct<'a>(content: &'a str, allocator: &'a Allocator, at: usize) -> String {
    let scanner = &mut Scanner { content, shared: None };
    let mut attributes = allocator.new_vec();
    let element = scanner.jsx_open(at, 0, &mut attributes).and_then(|open| {
        let close = if open.self_closing {
            None
        } else {
            Some(scanner.matching_close(at, open.end, open.name)?)
        };
        Some((open.name, open.self_closing, open.end, close))
    });
    let element = element.map(|element| format!("{element:?} {attributes:?}"));
    let group = scanner.skip_braces(at).map(|end| (&content[at + 1..end - 1], end));
    format!("{element:?} {group:?}")
}

/// The same answers, as the parser gets them.
fn cached<'a>(
    scope: &ScanScope<'a>,
    shortcuts: Shortcuts,
    content: &'a str,
    allocator: &'a Allocator,
    at: usize,
) -> String {
    let mut attributes = allocator.new_vec();
    let element = scope
        .jsx_element(content, at, 0, &mut attributes, shortcuts)
        .map(|(open, close)| (open.name, open.self_closing, open.end, close));
    let element = element.map(|element| format!("{element:?} {attributes:?}"));
    format!("{element:?} {:?}", scope.braces(content, at, shortcuts))
}

/// Every slice of `source` that starts and ends on a character boundary.
fn slices(source: &str) -> Vec<(usize, usize)> {
    let boundaries: Vec<usize> =
        (0..=source.len()).filter(|at| source.is_char_boundary(*at)).collect();
    let mut slices = Vec::new();
    for (index, start) in boundaries.iter().enumerate() {
        for end in &boundaries[index + 1..] {
            slices.push((*start, *end));
        }
    }
    slices
}

/// Scans every opener in every one of `slices` through one cache, in the
/// order given, and checks each answer against a scan of that slice alone.
fn assert_slices_match(
    source: &str,
    slices: &[(usize, usize)],
    shortcuts: Shortcuts,
    last_opener_first: bool,
) {
    let scope = ScanScope::new(source);
    let allocator = Allocator::new();
    for &(start, end) in slices {
        let content = &source[start..end];
        let mut openers: Vec<usize> = (0..content.len())
            .filter(|at| matches!(content.as_bytes()[*at], b'<' | b'{'))
            .collect();
        if last_opener_first {
            openers.reverse();
        }
        for at in openers {
            assert_eq!(
                cached(&scope, shortcuts, content, &allocator, at),
                direct(content, &allocator, at),
                "at {at} of {content:?}, a slice of {source:?}"
            );
        }
    }
}

fn assert_every_slice_matches(shortcuts: Shortcuts) {
    for source in SOURCES {
        let mut slices = slices(source);
        // Outermost first, the order the parser descends in.
        slices.sort_by_key(|&(start, end)| (start, std::cmp::Reverse(end)));
        assert_slices_match(source, &slices, shortcuts, false);
        // Shortest first, the order a probe of link text arrives in.
        slices.sort_by_key(|&(start, end)| (end - start, start));
        assert_slices_match(source, &slices, shortcuts, false);
        // Back to front, against the direction every walk runs in, so the
        // walks an outer opener takes pass over what inner ones recorded.
        slices.sort_by_key(|&(start, end)| (std::cmp::Reverse(start), end));
        assert_slices_match(source, &slices, shortcuts, true);
        slices.sort_by_key(|&(start, end)| (std::cmp::Reverse(end), std::cmp::Reverse(start)));
        assert_slices_match(source, &slices, shortcuts, true);
    }
}

#[test]
fn cached_scans_match_direct_scans_in_every_slice() {
    // A paragraph, the link text inside it and the children of an element
    // inside that are all slices of one source and all read one cache, so
    // every slice has to get the answer a scan of that slice alone gives,
    // whichever slices were scanned before it.
    assert_every_slice_matches(ALWAYS_CACHED);
}

#[test]
fn shortcuts_do_not_change_what_a_scan_finds() {
    // The direct scans that run before the cache see only the first few
    // bytes. Whatever they report has to hold for the whole slice, and
    // whatever they cannot settle has to reach the cache intact.
    for shortcuts in CUT_SHORT {
        assert_every_slice_matches(shortcuts);
    }
    assert_every_slice_matches(SHORTCUTS);
}

#[test]
fn a_slice_that_splits_a_backtick_run_is_scanned_directly() {
    // In the slice the lone backtick closes the code span and hides the
    // closing tag; in the source it is half of a longer run.
    let source = "<A>`</A>``";
    let content = &source[..9];
    let scope = ScanScope::new(source);
    let allocator = Allocator::new();
    assert_eq!(scope.base_of(content), None);
    assert_eq!(scope.base_of(source), Some(0));
    assert_eq!(cached(&scope, ALWAYS_CACHED, content, &allocator, 0), "None None");
    assert_eq!(direct(content, &allocator, 0), "None None");
}

#[test]
fn rebuilt_text_outside_the_source_is_scanned_directly() {
    let source = "<A>x</A> padding so that the copy below cannot overlap";
    let copy = String::from("<A>x</A>");
    let scope = ScanScope::new(source);
    assert_eq!(scope.base_of(&source[3..8]), Some(3));
    assert_eq!(scope.base_of(&copy), None);
}

fn walks(scope: &ScanScope<'_>) -> Option<u32> {
    scope.cache.get().map(|cache| cache.borrow().path.walks())
}

#[test]
fn a_run_of_unclosed_openers_is_walked_once() {
    let source = "<A>".repeat(200) + "</B>";
    let scope = ScanScope::new(&source);
    let allocator = Allocator::new();
    for opener in 0..200 {
        let mut attributes = allocator.new_vec();
        let element = scope.jsx_element(&source, opener * 3, 0, &mut attributes, SHORTCUTS);
        assert!(element.is_none());
    }
    assert_eq!(walks(&scope), Some(1));
}

#[test]
fn nested_elements_read_the_walk_that_closed_the_outermost() {
    let depth = 200;
    let source = "<A>".repeat(depth) + &"</A>".repeat(depth);
    let scope = ScanScope::new(&source);
    let allocator = Allocator::new();
    for level in 0..depth {
        // The children of the element one level out, as the parser cuts them.
        let (start, end) = (level * 3, source.len() - level * 4);
        let mut attributes = allocator.new_vec();
        let element = scope.jsx_element(&source[start..end], 0, 0, &mut attributes, SHORTCUTS);
        let close = element.and_then(|(_, close)| close);
        assert_eq!(close, Some((end - start - 4, end - start)));
    }
    assert_eq!(walks(&scope), Some(1));
}

#[test]
fn well_formed_elements_never_start_a_walk() {
    let source = "<A b={c}>x</A> {d} ".repeat(200);
    let scope = ScanScope::new(&source);
    let allocator = Allocator::new();
    for element in 0..200 {
        let at = element * 19;
        let mut attributes = allocator.new_vec();
        let found = scope.jsx_element(&source, at, 0, &mut attributes, SHORTCUTS);
        assert_eq!(found.and_then(|(_, close)| close), Some((at + 10, at + 14)));
        assert_eq!(scope.braces(&source, at + 15, SHORTCUTS), Some(("d", at + 18)));
    }
    assert_eq!(walks(&scope), Some(0));
}
