//! Parse `file-tree` fence bodies into a nested tree.

#[derive(Debug, PartialEq, Eq)]
pub(super) struct TreeNode {
    pub name: String,
    pub is_dir: bool,
    pub highlight: bool,
    pub children: Vec<TreeNode>,
}

struct TreeItem {
    depth: usize,
    name: String,
    is_dir: bool,
    highlight: bool,
}

pub(super) fn parse_tree(body: &str) -> Vec<TreeNode> {
    nest(parse_items(body))
}

fn parse_items(body: &str) -> Vec<TreeItem> {
    let mut items = Vec::new();
    let mut previous_depth = 0usize;
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some(item) = parse_item(line, previous_depth, items.is_empty()) else {
            continue;
        };
        previous_depth = item.depth;
        items.push(item);
    }
    items
}

fn nest(items: Vec<TreeItem>) -> Vec<TreeNode> {
    let mut roots = Vec::new();
    let mut stack: Vec<(usize, TreeNode)> = Vec::new();

    for item in items {
        let node = TreeNode {
            name: item.name,
            is_dir: item.is_dir,
            highlight: item.highlight,
            children: Vec::new(),
        };
        while stack.last().is_some_and(|(depth, _)| *depth >= item.depth) {
            let Some((_, finished)) = stack.pop() else {
                break;
            };
            attach(finished, &mut stack, &mut roots);
        }
        stack.push((item.depth, node));
    }

    while let Some((_, finished)) = stack.pop() {
        attach(finished, &mut stack, &mut roots);
    }
    roots
}

fn attach(node: TreeNode, stack: &mut [(usize, TreeNode)], roots: &mut Vec<TreeNode>) {
    if let Some((_, parent)) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}

fn parse_item(line: &str, previous_depth: usize, first: bool) -> Option<TreeItem> {
    let spaces = leading_spaces(line);
    let rest = line[spaces..].trim_start_matches('\t');
    let name_src = rest.strip_prefix("- ").or_else(|| rest.strip_prefix("-\t"))?;
    if name_src.trim().is_empty() {
        return None;
    }
    let (name, highlight) = parse_name(name_src);
    if name.is_empty() {
        return None;
    }
    let depth = if first { 0 } else { (spaces / 2).min(previous_depth + 1) };
    let is_dir = name.ends_with('/');
    Some(TreeItem { depth, name, is_dir, highlight })
}

fn parse_name(raw: &str) -> (String, bool) {
    let trimmed = raw.trim();
    let (body, trailing) = match trimmed.strip_suffix(" **") {
        Some(body) => (body.trim_end(), true),
        None => (trimmed, false),
    };
    if let Some(inner) = unwrap_bold(body) {
        (inner.to_string(), true)
    } else {
        (body.to_string(), trailing)
    }
}

fn unwrap_bold(value: &str) -> Option<&str> {
    let inner = value.strip_prefix("**")?.strip_suffix("**")?;
    (!inner.is_empty() && !inner.contains("**")).then_some(inner)
}

fn leading_spaces(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
}
