use ox_content_docs::DocsNavItem;

use crate::JsDocsNavItem;

pub(super) fn map_docs_nav_item(item: DocsNavItem) -> JsDocsNavItem {
    JsDocsNavItem {
        title: item.title,
        path: item.path,
        children: item.children.map(|children| {
            children.into_iter().map(map_docs_nav_item).collect::<Vec<JsDocsNavItem>>()
        }),
    }
}

pub(super) fn convert_docs_nav_item(item: JsDocsNavItem) -> DocsNavItem {
    DocsNavItem {
        title: item.title,
        path: item.path,
        children: item.children.map(|children| {
            children.into_iter().map(convert_docs_nav_item).collect::<Vec<DocsNavItem>>()
        }),
    }
}
