//! Scan state one parser shares between all of its JSX and brace probes.

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use ox_content_allocator::Vec;
use ox_content_ast::MdxJsxAttributeEntry;

use super::super::Parser;
use super::braces;
use super::close::{self, Search};
use super::lex::Lex;
use super::lex_memo::LexMemo;
use super::scan::{self, JsxOpen};
use super::tag_path::{Close, TagPath};

/// An opening tag and, unless it closes itself, where its closing tag is.
pub(super) type JsxElement<'a> = (JsxOpen<'a>, Option<(usize, usize)>);

/// How much is scanned directly before the cache is involved.
///
/// The cache pays off when a scan would otherwise be repeated over a long
/// stretch. Well-formed MDX is made of scans that are short anyway, and
/// those are cheaper done again than looked up.
#[derive(Clone, Copy)]
struct Shortcuts {
    /// Slices up to this long never use the cache. Almost every paragraph
    /// is this short, and no walk in such a slice is longer than this.
    slice_len: usize,
    /// How far an opening tag, and then its closing tag, is looked for
    /// directly. Most tags fit, and most inline elements close in a line.
    nearby: usize,
    /// How far a brace group is scanned directly. Nearly every
    /// `{expression}` is this short.
    brace_walk: usize,
}

const SHORTCUTS: Shortcuts = Shortcuts { slice_len: 256, nearby: 128, brace_walk: 64 };

/// What earlier scans of one source found, and the source they ran over.
///
/// Offsets in the cache are relative to that source, so a paragraph, the
/// link text inside it and the children of an element inside that all read
/// the same entries. Nothing is allocated until a scan needs it.
pub(in crate::parser) struct ScanScope<'a> {
    source: &'a str,
    cache: OnceCell<Rc<RefCell<ScanCache>>>,
}

struct ScanCache {
    lex: LexMemo,
    path: TagPath,
}

/// JSX and brace scans over one content slice.
struct Scanner<'m, 'a> {
    content: &'a str,
    shared: Option<Shared<'m, 'a>>,
}

struct Shared<'m, 'a> {
    cache: &'m mut ScanCache,
    source: &'a str,
    /// Offset of the content slice in `source`.
    base: usize,
    nearby: usize,
}

impl Shared<'_, '_> {
    fn lex<'l>(&'l mut self, content: &'l str) -> Lex<'l> {
        Lex::shared(content.as_bytes(), &mut self.cache.lex, self.source.as_bytes(), self.base)
    }
}

impl<'a> Scanner<'_, 'a> {
    fn lex(&mut self) -> Lex<'_> {
        match &mut self.shared {
            Some(shared) => shared.lex(self.content),
            None => Lex::plain(self.content.as_bytes()),
        }
    }

    fn jsx_open(
        &mut self,
        start: usize,
        offset: usize,
        attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
    ) -> Option<JsxOpen<'a>> {
        let content = self.content;
        scan::scan_jsx_open(&mut self.lex(), content, start, offset, attributes)
    }

    /// Start and end of the tag that closes the opener at
    /// `open_start..open_end`.
    fn matching_close(
        &mut self,
        open_start: usize,
        open_end: usize,
        name: Option<&str>,
    ) -> Option<(usize, usize)> {
        let content = self.content;
        let search = |lex: &mut Lex<'_>, stop: usize| {
            close::find_matching_close(lex, content, open_end, name, stop)
        };
        let Some(shared) = &mut self.shared else {
            return search(&mut Lex::plain(content.as_bytes()), usize::MAX).found();
        };
        let base = shared.base;
        if !shared.cache.path.closes_after(shared.source, base + open_end) {
            return None;
        }
        // An opener the remembered walk has not met would start a new one.
        // Look nearby first, so a well-formed element does not pay for it.
        if !shared.cache.path.covers(base + open_start) {
            let stop = open_end.saturating_add(shared.nearby);
            match search(&mut shared.lex(content), stop) {
                Search::Unfinished => {}
                settled => return settled.found(),
            }
        }
        let (start, end, limit) = (base + open_start, base + open_end, base + content.len());
        let ScanCache { lex, path } = &mut *shared.cache;
        match path.matching_close(lex, shared.source, start, end, limit) {
            Close::Found(start, end) => Some((start - base, end - base)),
            Close::Missing => None,
            Close::Unknown => search(&mut shared.lex(content), usize::MAX).found(),
        }
    }

    /// End offset of the `{…}` group at `start`.
    fn skip_braces(&mut self, start: usize) -> Option<usize> {
        self.lex().skip_braces(start)
    }
}

impl<'a> ScanScope<'a> {
    pub(in crate::parser) fn new(source: &'a str) -> Self {
        Self { source, cache: OnceCell::new() }
    }

    /// The same scope, for a parser whose source is a slice of this one.
    pub(super) fn share(&self) -> Self {
        Self { source: self.source, cache: self.cache.clone() }
    }

    /// Scans the opening tag at `start` of `content` and, unless it closes
    /// itself, the tag that closes it. `None` when either is missing.
    fn jsx_element(
        &self,
        content: &'a str,
        start: usize,
        offset: usize,
        attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
        shortcuts: Shortcuts,
    ) -> Option<JsxElement<'a>> {
        // A tag that closes within the first few bytes is the same tag
        // however much follows it: the scan reads nothing past its `>`.
        // Only one that does not fit is scanned through the cache.
        let head = &content[..content.floor_char_boundary(start.saturating_add(shortcuts.nearby))];
        let nearby =
            scan::scan_jsx_open(&mut Lex::plain(head.as_bytes()), head, start, offset, attributes);
        if nearby.as_ref().is_some_and(|open| open.self_closing) {
            return nearby.map(|open| (open, None));
        }
        self.scan(content, shortcuts, |scanner| {
            let open = if let Some(open) = nearby {
                open
            } else {
                attributes.clear();
                scanner.jsx_open(start, offset, attributes)?
            };
            if open.self_closing {
                return Some((open, None));
            }
            let close = scanner.matching_close(start, open.end, open.name)?;
            Some((open, Some(close)))
        })
    }

    /// Inner source and end offset of the `{…}` group at `start`.
    fn braces(
        &self,
        content: &'a str,
        start: usize,
        shortcuts: Shortcuts,
    ) -> Option<(&'a str, usize)> {
        // A group that closes within the first few bytes closes there
        // however much follows it.
        let nearby = content.floor_char_boundary(start.saturating_add(shortcuts.brace_walk));
        if let Some(group) = braces::scan_balanced_braces(&content[..nearby], start) {
            return Some(group);
        }
        let end = self.scan(content, shortcuts, |scanner| scanner.skip_braces(start))?;
        Some((&content[start + 1..end - 1], end))
    }

    fn scan<R>(
        &self,
        content: &'a str,
        shortcuts: Shortcuts,
        scan: impl FnOnce(&mut Scanner<'_, 'a>) -> R,
    ) -> R {
        let Some(base) = self.base_of(content).filter(|_| content.len() > shortcuts.slice_len)
        else {
            return scan(&mut Scanner { content, shared: None });
        };
        let cache = self.cache.get_or_init(|| {
            let lex = LexMemo::new(shortcuts.brace_walk);
            Rc::new(RefCell::new(ScanCache { lex, path: TagPath::default() }))
        });
        let shared = Shared {
            cache: &mut cache.borrow_mut(),
            source: self.source,
            base,
            nearby: shortcuts.nearby,
        };
        scan(&mut Scanner { content, shared: Some(shared) })
    }

    /// Offset of `content` in the source, when its scans read the same
    /// answers the source-wide cache holds.
    fn base_of(&self, content: &str) -> Option<usize> {
        // Table cells and other rebuilt text live outside the source.
        let base = content.as_ptr().addr().checked_sub(self.source.as_ptr().addr())?;
        let limit = base + content.len();
        if limit > self.source.len() {
            return None;
        }
        // A code-span closer may not be followed by another backtick, so a
        // slice that ends inside a backtick run sees a closer the source
        // does not. Every other scan reads the same answer either way.
        let next = self.source.as_bytes().get(limit);
        let splits_run = next == Some(&b'`') && content.ends_with('`');
        (!splits_run).then_some(base)
    }
}

impl<'a> Parser<'a> {
    /// Scans the opening tag at `start` of `content` and, unless it closes
    /// itself, the tag that closes it, using whatever this parser already
    /// knows about the source.
    pub(super) fn scan_jsx_element(
        &self,
        content: &'a str,
        start: usize,
        offset: usize,
        attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
    ) -> Option<JsxElement<'a>> {
        self.mdx_scan.jsx_element(content, start, offset, attributes, SHORTCUTS)
    }

    /// Inner source and end offset of the `{…}` group at `start` of
    /// `content`.
    pub(super) fn scan_mdx_braces(
        &self,
        content: &'a str,
        start: usize,
    ) -> Option<(&'a str, usize)> {
        self.mdx_scan.braces(content, start, SHORTCUTS)
    }
}

#[cfg(test)]
mod tests;
