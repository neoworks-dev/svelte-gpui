//! Selectors: the compound parts of a rule's selector, how they combine, and
//! whether they match an element of the tree. It covers what Tailwind and its
//! typography plugin write: tags, classes, attribute tests, descendant and child
//! combinators, `:where()`, `:is()` and `:not()`, and the pseudo-classes the host
//! can model. Anything else makes the whole selector unsupported, and it is left out.

use crate::tree::{Kind, Node, NodeId, Tree};

#[derive(Debug, Clone, PartialEq)]
pub enum Pseudo {
    None,
    Hover,
    /// `:hover` on an ancestor carrying this class, as Tailwind's `group-hover:` writes it.
    GroupHover(String),
    Focus,
    Disabled,
    Checked,
    /// A pseudo-class this engine does not model, so the rule never applies.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone)]
pub enum AttributeOperator {
    Exists,
    Equals,
    /// `~=`: one of the space-separated words.
    Word,
    Prefix,
    Suffix,
    Contains,
}

#[derive(Debug, Clone)]
pub struct AttributeTest {
    pub name: String,
    pub operator: AttributeOperator,
    pub value: String,
}

/// Where an element sits among its siblings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    First,
    Last,
    Only,
}

#[derive(Debug, Clone, Default)]
pub struct Compound {
    pub tag: Option<String>,
    pub classes: Vec<String>,
    pub attributes: Vec<AttributeTest>,
    pub positions: Vec<Position>,
    /// Compounds that must not match, from `:not()`.
    pub excluded: Vec<Compound>,
}

#[derive(Debug, Clone)]
pub struct Selector {
    /// From the outermost ancestor to the element itself.
    pub compounds: Vec<Compound>,
    /// `combinators[i]` joins `compounds[i]` to `compounds[i + 1]`.
    pub combinators: Vec<Combinator>,
    /// The pseudo-class on the element itself.
    pub pseudo: Pseudo,
}

/// Attributes whose absence and `"false"` mean the same thing.
const BOOLEAN: &[&str] = &["checked", "disabled", "selected", "readonly", "required", "multiple", "hidden"];

fn attribute<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
    let value = node.attributes.get(name)?;
    if value == "false" && BOOLEAN.contains(&name) {
        return None;
    }
    Some(value.as_str())
}

/// The element children of a node's parent, and where the node is among them.
fn place_among_siblings(tree: &Tree, id: NodeId) -> Option<(usize, usize)> {
    let parent = tree.nodes.get(&tree.nodes.get(&id)?.parent?)?;
    let siblings: Vec<NodeId> = parent
        .children
        .iter()
        .copied()
        .filter(|child| matches!(tree.nodes.get(child).map(|node| &node.kind), Some(Kind::Element { .. })))
        .collect();
    let index = siblings.iter().position(|sibling| *sibling == id)?;
    Some((index, siblings.len()))
}

impl Compound {
    fn matches(&self, tree: &Tree, id: NodeId, node: &Node) -> bool {
        let Kind::Element { tag } = &node.kind else {
            return false;
        };
        if !self.positions.is_empty() {
            let Some((index, total)) = place_among_siblings(tree, id) else {
                return false;
            };
            for position in &self.positions {
                let holds = match position {
                    Position::First => index == 0,
                    Position::Last => index + 1 == total,
                    Position::Only => total == 1,
                };
                if !holds {
                    return false;
                }
            }
        }
        if let Some(required) = &self.tag {
            if required != tag {
                return false;
            }
        }
        if !self.classes.is_empty() {
            let present = node.attributes.get("class").map(String::as_str).unwrap_or("");
            for class in &self.classes {
                if !present.split_whitespace().any(|name| name == class) {
                    return false;
                }
            }
        }
        for test in &self.attributes {
            let Some(actual) = attribute(node, &test.name) else {
                return false;
            };
            let holds = match test.operator {
                AttributeOperator::Exists => true,
                AttributeOperator::Equals => actual == test.value,
                AttributeOperator::Word => actual.split_whitespace().any(|word| word == test.value),
                AttributeOperator::Prefix => actual.starts_with(&test.value),
                AttributeOperator::Suffix => actual.ends_with(&test.value),
                AttributeOperator::Contains => actual.contains(&test.value),
            };
            if !holds {
                return false;
            }
        }
        !self.excluded.iter().any(|excluded| excluded.matches(tree, id, node))
    }

    fn is_empty(&self) -> bool {
        self.tag.is_none()
            && self.classes.is_empty()
            && self.attributes.is_empty()
            && self.positions.is_empty()
            && self.excluded.is_empty()
    }
}

impl Selector {
    /// The compound for the element the rule is about.
    pub fn subject(&self) -> &Compound {
        &self.compounds[self.compounds.len() - 1]
    }

    /// Whether the element `id` matches, looking up through its ancestors as needed.
    pub fn matches(&self, tree: &Tree, id: NodeId) -> bool {
        let Some(node) = tree.nodes.get(&id) else {
            return false;
        };
        if !self.subject().matches(tree, id, node) {
            return false;
        }
        let mut current = node.parent;
        let mut index = self.compounds.len() - 1;
        while index > 0 {
            let combinator = self.combinators[index - 1];
            let wanted = &self.compounds[index - 1];
            let Some(found) = find_ancestor(tree, current, wanted, combinator) else {
                return false;
            };
            current = tree.nodes.get(&found).and_then(|node| node.parent);
            index -= 1;
        }
        true
    }

    /// Reads a selector, or None when it uses something the engine does not model.
    pub fn parse(text: &str) -> Option<Selector> {
        if text.contains("::") || text.contains('&') {
            return None;
        }
        // Tailwind wraps some selectors whole: `:where(.a > :not(:last-child))`.
        if let Some(inner) = text.trim().strip_prefix(":where(").and_then(|rest| rest.strip_suffix(')')) {
            if crate::css::split_top_level(inner, ',').len() == 1 && !inner.starts_with(':') {
                return Selector::parse(inner);
            }
        }
        if let Some(grouped) = parse_group_hover(text) {
            return Some(grouped);
        }
        let (pieces, combinators) = split_compounds(text)?;
        let mut compounds = Vec::new();
        let mut pseudo = Pseudo::None;
        let last = pieces.len() - 1;
        for (index, piece) in pieces.iter().enumerate() {
            let (compound, found) = parse_compound(piece, index == last)?;
            if index == last {
                pseudo = found;
            }
            compounds.push(compound);
        }
        if compounds[last].is_empty() {
            return None;
        }
        Some(Selector {
            compounds,
            combinators,
            pseudo,
        })
    }
}

/// The ancestor, starting at `from`, that matches. A child combinator looks only at
/// `from`; a descendant combinator looks all the way up.
fn find_ancestor(tree: &Tree, from: Option<NodeId>, wanted: &Compound, combinator: Combinator) -> Option<NodeId> {
    let mut candidate = from;
    while let Some(id) = candidate {
        let node = tree.nodes.get(&id)?;
        if wanted.matches(tree, id, node) {
            return Some(id);
        }
        if combinator == Combinator::Child {
            return None;
        }
        candidate = node.parent;
    }
    None
}

/// Tailwind's `.x:is(:where(.group):hover *)` is `.x` inside a hovered `.group`.
fn parse_group_hover(text: &str) -> Option<Selector> {
    let start = text.find(":is(:where(.")?;
    let after = &text[start + ":is(:where(.".len()..];
    let group_end = after.find(')')?;
    let group = after[..group_end].to_string();
    if !after[group_end..].starts_with("):hover *)") {
        return None;
    }
    let (compound, _) = parse_compound(&text[..start], true)?;
    if compound.is_empty() {
        return None;
    }
    Some(Selector {
        compounds: vec![compound],
        combinators: Vec::new(),
        pseudo: Pseudo::GroupHover(group),
    })
}

/// Splits a selector into its compounds and the combinators between them.
fn split_compounds(text: &str) -> Option<(Vec<String>, Vec<Combinator>)> {
    let mut pieces: Vec<String> = Vec::new();
    let mut combinators: Vec<Combinator> = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut pending: Option<Combinator> = None;
    let mut escaped = false;
    for character in text.trim().chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' => {
                current.push(character);
                escaped = true;
            }
            '(' | '[' => {
                depth += 1;
                current.push(character);
            }
            ')' | ']' => {
                depth -= 1;
                current.push(character);
            }
            '+' | '~' if depth == 0 => return None,
            '>' if depth == 0 => {
                flush(&mut pieces, &mut combinators, &mut current, &mut pending);
                pending = Some(Combinator::Child);
            }
            character if character.is_whitespace() && depth == 0 => {
                flush(&mut pieces, &mut combinators, &mut current, &mut pending);
                if pending.is_none() && !pieces.is_empty() {
                    pending = Some(Combinator::Descendant);
                }
            }
            _ => current.push(character),
        }
    }
    flush(&mut pieces, &mut combinators, &mut current, &mut pending);
    if pieces.is_empty() || pending.is_some() && pieces.len() == combinators.len() {
        return None;
    }
    Some((pieces, combinators))
}

fn flush(
    pieces: &mut Vec<String>,
    combinators: &mut Vec<Combinator>,
    current: &mut String,
    pending: &mut Option<Combinator>,
) {
    if current.is_empty() {
        return;
    }
    if !pieces.is_empty() {
        combinators.push(pending.take().unwrap_or(Combinator::Descendant));
    }
    pieces.push(std::mem::take(current));
    *pending = None;
}

/// Reads one compound selector. `subject` says whether it is the element itself, which
/// alone can carry a pseudo-class the host models.
fn parse_compound(text: &str, subject: bool) -> Option<(Compound, Pseudo)> {
    let characters: Vec<char> = text.chars().collect();
    let mut compound = Compound::default();
    let mut pseudo = Pseudo::None;
    let mut at = 0;
    while at < characters.len() {
        match characters[at] {
            '*' => at += 1,
            '.' => {
                at += 1;
                let (name, next) = read_identifier(&characters, at);
                compound.classes.push(name);
                at = next;
            }
            '#' => {
                at += 1;
                let (name, next) = read_identifier(&characters, at);
                compound.attributes.push(AttributeTest {
                    name: "id".into(),
                    operator: AttributeOperator::Equals,
                    value: name,
                });
                at = next;
            }
            '[' => {
                let close = matching(&characters, at, '[', ']')?;
                let inner: String = characters[at + 1..close].iter().collect();
                compound.attributes.push(parse_attribute(&inner)?);
                at = close + 1;
            }
            ':' => {
                at += 1;
                let (name, next) = read_identifier(&characters, at);
                at = next;
                let mut argument = String::new();
                if characters.get(at) == Some(&'(') {
                    let close = matching(&characters, at, '(', ')')?;
                    argument = characters[at + 1..close].iter().collect();
                    at = close + 1;
                }
                apply_pseudo(&name, &argument, subject, &mut compound, &mut pseudo)?;
            }
            _ => {
                let (name, next) = read_identifier(&characters, at);
                if name.is_empty() {
                    return None;
                }
                compound.tag = Some(name);
                at = next;
            }
        }
    }
    Some((compound, pseudo))
}

fn apply_pseudo(
    name: &str,
    argument: &str,
    subject: bool,
    compound: &mut Compound,
    pseudo: &mut Pseudo,
) -> Option<()> {
    match name {
        "where" | "is" => {
            let alternatives = crate::css::split_top_level(argument, ',');
            if alternatives.len() != 1 {
                return None;
            }
            let (inner, _) = parse_compound(alternatives[0].trim(), false)?;
            merge(compound, inner);
        }
        "not" => {
            let alternatives = crate::css::split_top_level(argument, ',');
            // A `:not()` over something more than a single compound is assumed to hold.
            if alternatives.len() == 1 && !alternatives[0].trim().contains(' ') {
                if let Some((inner, _)) = parse_compound(alternatives[0].trim(), false) {
                    compound.excluded.push(inner);
                }
            }
        }
        "checked" | "disabled" if !subject => {
            compound.attributes.push(AttributeTest {
                name: name.into(),
                operator: AttributeOperator::Exists,
                value: String::new(),
            });
        }
        "first-child" => compound.positions.push(Position::First),
        "last-child" => compound.positions.push(Position::Last),
        "only-child" => compound.positions.push(Position::Only),
        "hover" | "focus" | "focus-visible" | "focus-within" | "active" if !subject => return None,
        "hover" => *pseudo = Pseudo::Hover,
        "focus" | "focus-visible" | "focus-within" => *pseudo = Pseudo::Focus,
        "disabled" => *pseudo = Pseudo::Disabled,
        "checked" => *pseudo = Pseudo::Checked,
        _ => *pseudo = Pseudo::Other,
    }
    Some(())
}

fn merge(into: &mut Compound, from: Compound) {
    if into.tag.is_none() {
        into.tag = from.tag;
    }
    into.classes.extend(from.classes);
    into.attributes.extend(from.attributes);
    into.positions.extend(from.positions);
    into.excluded.extend(from.excluded);
}

fn parse_attribute(inner: &str) -> Option<AttributeTest> {
    let operators = [
        ("~=", AttributeOperator::Word),
        ("^=", AttributeOperator::Prefix),
        ("$=", AttributeOperator::Suffix),
        ("*=", AttributeOperator::Contains),
        ("=", AttributeOperator::Equals),
    ];
    for (symbol, operator) in operators {
        if let Some((name, value)) = inner.split_once(symbol) {
            let value = value.trim().trim_matches(['"', '\'']).to_string();
            return Some(AttributeTest {
                name: name.trim().to_string(),
                operator,
                value,
            });
        }
    }
    Some(AttributeTest {
        name: inner.trim().to_string(),
        operator: AttributeOperator::Exists,
        value: String::new(),
    })
}

/// An identifier with its backslash escapes undone.
fn read_identifier(characters: &[char], mut at: usize) -> (String, usize) {
    let mut name = String::new();
    while at < characters.len() {
        match characters[at] {
            '\\' => {
                at += 1;
                if let Some(escaped) = characters.get(at) {
                    name.push(*escaped);
                }
            }
            character if character.is_alphanumeric() || matches!(character, '-' | '_') => name.push(character),
            _ => break,
        }
        at += 1;
    }
    (name, at)
}

/// The index of the bracket that closes the one at `open`.
fn matching(characters: &[char], open: usize, opener: char, closer: char) -> Option<usize> {
    let mut depth = 0;
    let mut escaped = false;
    for (offset, character) in characters[open..].iter().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if *character == '\\' {
            escaped = true;
        } else if *character == opener {
            depth += 1;
        } else if *character == closer {
            depth -= 1;
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_descendant_selector_with_where_and_not() {
        let selector = Selector::parse(
            r#".prose :where(a):not(:where([class~="not-prose"], [class~="not-prose"] *))"#,
        )
        .expect("supported");
        assert_eq!(selector.compounds.len(), 2);
        assert_eq!(selector.compounds[0].classes, vec!["prose"]);
        assert_eq!(selector.subject().tag.as_deref(), Some("a"));
    }

    #[test]
    fn reads_group_hover_and_unescapes_classes() {
        let selector = Selector::parse(r".group-hover\:block:is(:where(.group):hover *)").expect("supported");
        assert_eq!(selector.pseudo, Pseudo::GroupHover("group".into()));
        assert_eq!(selector.subject().classes, vec!["group-hover:block"]);
    }

    #[test]
    fn leaves_out_siblings_and_pseudo_elements() {
        assert!(Selector::parse("a + b").is_none());
        assert!(Selector::parse(".a::before").is_none());
    }
}
