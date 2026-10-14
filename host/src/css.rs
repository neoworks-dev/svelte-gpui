//! A small CSS engine, only as big as a Tailwind stylesheet needs: it reads the
//! rules, keeps the custom properties, and evaluates `var()`, `calc()` and
//! colour functions. It does not implement a cascade beyond "later rule wins";
//! selectors it cannot match are skipped and reported once.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use gpui::Rgba;

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

#[derive(Debug, Clone)]
pub struct Selector {
    pub tag: Option<String>,
    pub classes: Vec<String>,
    pub pseudo: Pseudo,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selector: Selector,
    pub declarations: Vec<(String, String)>,
}

#[derive(Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
    by_class: HashMap<String, Vec<usize>>,
    by_tag: HashMap<String, Vec<usize>>,
    /// Custom properties from `:root`, which every `var()` falls back to.
    pub variables: HashMap<String, String>,
    /// Declarations that apply to the whole document, from `html`, `body` and `:root`.
    pub root_declarations: Vec<(String, String)>,
    skipped: HashSet<String>,
    /// Class names that appear in a selector the engine left out.
    skipped_classes: HashSet<String>,
}

impl Stylesheet {
    pub fn parse(source: &str) -> Self {
        let mut sheet = Self::default();
        let cleaned = strip_comments(source);
        sheet.parse_block(&cleaned);
        for (index, rule) in sheet.rules.iter().enumerate() {
            for class in &rule.selector.classes {
                sheet.by_class.entry(class.clone()).or_default().push(index);
            }
            if rule.selector.classes.is_empty() {
                if let Some(tag) = &rule.selector.tag {
                    sheet.by_tag.entry(tag.clone()).or_default().push(index);
                }
            }
        }
        sheet
    }

    /// Indices of rules that mention `class`, in stylesheet order.
    pub fn rules_for_class(&self, class: &str) -> &[usize] {
        match self.by_class.get(class) {
            Some(indices) => indices,
            None => &[],
        }
    }

    pub fn rules_for_tag(&self, tag: &str) -> &[usize] {
        match self.by_tag.get(tag) {
            Some(indices) => indices,
            None => &[],
        }
    }

    fn parse_block(&mut self, text: &str) {
        let mut rest = text;
        loop {
            rest = rest.trim_start();
            if rest.is_empty() {
                return;
            }
            let Some(stop) = rest.find(['{', ';']) else {
                return;
            };
            let prelude = rest[..stop].trim();
            if rest.as_bytes()[stop] == b';' {
                rest = &rest[stop + 1..];
                continue;
            }
            let Some(body_range) = matching_brace(rest, stop) else {
                return;
            };
            let body = &rest[body_range.clone()];
            rest = &rest[body_range.end + 1..];
            self.handle_rule(prelude, body);
        }
    }

    fn handle_rule(&mut self, prelude: &str, body: &str) {
        if let Some(at_rule) = prelude.strip_prefix('@') {
            let name = at_rule.split_whitespace().next().unwrap_or("");
            let wanted = match name {
                "layer" => true,
                "media" => prelude.contains("hover: hover") || prelude.contains("hover:hover"),
                _ => false,
            };
            if wanted {
                self.parse_block(body);
            }
            return;
        }
        let declarations = parse_declarations(body);
        for selector_text in split_top_level(prelude, ',') {
            self.add_selector(selector_text.trim(), &declarations);
        }
    }

    fn add_selector(&mut self, text: &str, declarations: &[(String, String)]) {
        if matches!(text, ":root" | ":host" | "html" | "body" | ":root, :host" | "html, :host") {
            self.take_root(declarations);
            return;
        }
        match parse_selector(text) {
            Some(selector) => self.rules.push(Rule {
                selector,
                declarations: declarations.to_vec(),
            }),
            None => {
                self.skipped.insert(text.to_string());
                for class in class_names(text) {
                    self.skipped_classes.insert(class);
                }
            }
        }
    }

    fn take_root(&mut self, declarations: &[(String, String)]) {
        for (name, value) in declarations {
            if name.starts_with("--") {
                self.variables.insert(name.clone(), value.clone());
                continue;
            }
            self.root_declarations.push((name.clone(), value.clone()));
        }
    }

    /// Whether a class is named by a rule that was left out.
    pub fn is_skipped_class(&self, class: &str) -> bool {
        self.skipped_classes.contains(class)
    }

    /// Whether any rule that was kept mentions the class.
    pub fn has_rule_for(&self, class: &str) -> bool {
        self.by_class.contains_key(class)
    }

    /// Selectors that were left out, for a coverage report.
    pub fn skipped_selectors(&self) -> Vec<&str> {
        let mut found: Vec<&str> = self.skipped.iter().map(String::as_str).collect();
        found.sort();
        found
    }
}

/// The class names a selector mentions, with escapes undone: `.a\:b:hover` gives `a:b`.
fn class_names(selector: &str) -> Vec<String> {
    let characters: Vec<char> = selector.chars().collect();
    let mut found = Vec::new();
    let mut at = 0;
    while at < characters.len() {
        if characters[at] != '.' {
            at += 1;
            continue;
        }
        at += 1;
        let mut name = String::new();
        while at < characters.len() {
            match characters[at] {
                '\\' => {
                    at += 1;
                    if let Some(escaped) = characters.get(at) {
                        name.push(*escaped);
                    }
                }
                character if character.is_alphanumeric() || matches!(character, '-' | '_') => {
                    name.push(character);
                }
                _ => break,
            }
            at += 1;
        }
        if !name.is_empty() {
            found.push(name);
        }
    }
    found
}

fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start..].find("*/") {
            Some(end) => rest = &rest[start + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// The range between the `{` at `open` and its matching `}`.
fn matching_brace(text: &str, open: usize) -> Option<Range<usize>> {
    let mut depth = 0;
    for (offset, character) in text[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + 1..open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// Splits on `separator` outside parentheses and brackets.
pub fn split_top_level(text: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (index, character) in text.char_indices() {
        match character {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            _ if character == separator && depth == 0 => {
                parts.push(&text[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

pub fn parse_declarations(body: &str) -> Vec<(String, String)> {
    let mut declarations = Vec::new();
    for part in split_top_level(body, ';') {
        let Some(colon) = part.find(':') else {
            continue;
        };
        let name = part[..colon].trim().to_string();
        let value = part[colon + 1..]
            .replace("!important", "")
            .trim()
            .to_string();
        if name.is_empty() || value.is_empty() || name.contains('{') {
            continue;
        }
        declarations.push((name, value));
    }
    declarations
}

/// Reads one compound selector: an optional tag, classes, and at most one pseudo-class.
fn parse_selector(text: &str) -> Option<Selector> {
    if text.contains("::") || text.contains('>') || text.contains('+') || text.contains('~') {
        return None;
    }
    let (compound, pseudo) = split_group_hover(text)?;
    if compound.contains(' ') {
        return None;
    }

    let mut selector = Selector {
        tag: None,
        classes: Vec::new(),
        pseudo,
    };
    let characters: Vec<char> = compound.chars().collect();
    let mut at = 0;
    let mut name = String::new();
    let mut mode = 't';
    let mut flush = |mode: char, name: &mut String, selector: &mut Selector| {
        if name.is_empty() {
            return;
        }
        match mode {
            't' => selector.tag = Some(name.clone()),
            '.' => selector.classes.push(name.clone()),
            ':' => selector.pseudo = pseudo_named(name),
            _ => {}
        }
        name.clear();
    };
    while at < characters.len() {
        let character = characters[at];
        match character {
            '\\' => {
                at += 1;
                if let Some(escaped) = characters.get(at) {
                    name.push(*escaped);
                }
            }
            '.' | ':' => {
                flush(mode, &mut name, &mut selector);
                mode = character;
            }
            '*' if name.is_empty() && mode == 't' => {
                return None;
            }
            '[' | '(' | ')' | ']' => return None,
            _ => name.push(character),
        }
        at += 1;
    }
    flush(mode, &mut name, &mut selector);
    if selector.pseudo == Pseudo::Focus && selector.classes.is_empty() {
        return None;
    }
    if selector.tag.is_none() && selector.classes.is_empty() {
        return None;
    }
    Some(selector)
}

fn pseudo_named(name: &str) -> Pseudo {
    match name {
        "hover" => Pseudo::Hover,
        "focus" | "focus-visible" | "focus-within" => Pseudo::Focus,
        "disabled" => Pseudo::Disabled,
        "checked" => Pseudo::Checked,
        _ => Pseudo::Other,
    }
}

/// Splits Tailwind's `.x:is(:where(.group):hover *)` into `.x` and a group-hover pseudo.
fn split_group_hover(text: &str) -> Option<(String, Pseudo)> {
    let Some(start) = text.find(":is(:where(.") else {
        return Some((text.to_string(), Pseudo::None));
    };
    let after = &text[start + ":is(:where(.".len()..];
    let group_end = after.find(')')?;
    let group = after[..group_end].to_string();
    if !after[group_end..].starts_with("):hover *)") && !after[group_end..].starts_with("):hover *") {
        return None;
    }
    Some((text[..start].to_string(), Pseudo::GroupHover(group)))
}

// ---- Values ----------------------------------------------------------------

/// A length that either has a size of its own or is a share of its parent's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f32),
    Percent(f32),
}

/// Resolves `var()` references against a set of custom properties.
pub struct Variables<'a> {
    pub root: &'a HashMap<String, String>,
}

impl Variables<'_> {
    /// Replaces every `var(--name, fallback)` in a value, repeatedly.
    pub fn substitute(&self, value: &str) -> String {
        let mut current = value.to_string();
        for _ in 0..8 {
            let Some(start) = current.find("var(") else {
                return current;
            };
            let Some(end) = matching_paren(&current, start + 3) else {
                return current;
            };
            let inner = current[start + 4..end].to_string();
            let replacement = self.resolve_reference(&inner);
            current.replace_range(start..=end, &replacement);
        }
        current
    }

    fn resolve_reference(&self, inner: &str) -> String {
        let parts = split_top_level(inner, ',');
        let name = parts[0].trim();
        if let Some(value) = self.root.get(name) {
            return value.clone();
        }
        if parts.len() > 1 {
            return inner[parts[0].len() + 1..].trim().to_string();
        }
        String::new()
    }
}

fn matching_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0;
    for (offset, character) in text[open..].char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parses a length: `12px`, `0.5rem`, `50%`, `0`, or a `calc()` of those.
pub fn parse_length(text: &str) -> Option<Length> {
    let trimmed = text.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        return percent.trim().parse().ok().map(Length::Percent);
    }
    if let Some(inner) = trimmed.strip_prefix("calc(") {
        let expression = inner.strip_suffix(')')?;
        return evaluate(expression).map(|(value, percent)| {
            if percent {
                Length::Percent(value)
            } else {
                Length::Px(value)
            }
        });
    }
    evaluate(trimmed).map(|(value, percent)| {
        if percent {
            Length::Percent(value)
        } else {
            Length::Px(value)
        }
    })
}

/// Parses a plain number, such as a line-height multiplier or a `calc(1 / 0.75)`.
pub fn parse_number(text: &str) -> Option<f32> {
    let trimmed = text.trim();
    let expression = match trimmed.strip_prefix("calc(") {
        Some(inner) => inner.strip_suffix(')')?,
        None => trimmed,
    };
    evaluate(expression).map(|(value, percent)| if percent { value / 100.0 } else { value })
}

/// Evaluates `a * b`, `a / b`, `a + b`, `a - b` and parentheses over numbers with
/// px, rem, em or % units. The flag says whether the result is a percentage.
fn evaluate(expression: &str) -> Option<(f32, bool)> {
    let tokens = tokenize(expression)?;
    let mut cursor = 0;
    let result = parse_sum(&tokens, &mut cursor)?;
    if cursor != tokens.len() {
        return None;
    }
    Some(result)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f32, bool),
    Operator(char),
    Open,
    Close,
}

fn tokenize(expression: &str) -> Option<Vec<Token>> {
    let characters: Vec<char> = expression.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0;
    while at < characters.len() {
        let character = characters[at];
        if character.is_whitespace() {
            at += 1;
            continue;
        }
        match character {
            '(' => tokens.push(Token::Open),
            ')' => tokens.push(Token::Close),
            '*' | '/' | '+' => tokens.push(Token::Operator(character)),
            '-' if !starts_number(&characters, at, &tokens) => tokens.push(Token::Operator('-')),
            _ if characters[at..].starts_with(&['i', 'n', 'f', 'i', 'n', 'i', 't', 'y']) => {
                // Tailwind's `rounded-full` is `calc(infinity * 1px)`.
                tokens.push(Token::Number(9999.0, false));
                at += "infinity".len();
                continue;
            }
            _ => {
                let (token, next) = read_number(&characters, at)?;
                tokens.push(token);
                at = next;
                continue;
            }
        }
        at += 1;
    }
    Some(tokens)
}

fn starts_number(characters: &[char], at: usize, tokens: &[Token]) -> bool {
    let previous_is_value = matches!(tokens.last(), Some(Token::Number(..) | Token::Close));
    if previous_is_value {
        return false;
    }
    matches!(characters.get(at + 1), Some(next) if next.is_ascii_digit() || *next == '.')
}

fn read_number(characters: &[char], start: usize) -> Option<(Token, usize)> {
    let mut at = start;
    let mut text = String::new();
    if characters[at] == '-' {
        text.push('-');
        at += 1;
    }
    while at < characters.len() && (characters[at].is_ascii_digit() || characters[at] == '.') {
        text.push(characters[at]);
        at += 1;
    }
    let mut unit = String::new();
    while at < characters.len() && (characters[at].is_ascii_alphabetic() || characters[at] == '%') {
        unit.push(characters[at]);
        at += 1;
    }
    let value: f32 = text.parse().ok()?;
    let (scaled, percent) = match unit.as_str() {
        "" | "px" => (value, false),
        "rem" | "em" => (value * 16.0, false),
        "%" => (value, true),
        _ => return None,
    };
    Some((Token::Number(scaled, percent), at))
}

fn parse_sum(tokens: &[Token], cursor: &mut usize) -> Option<(f32, bool)> {
    let (mut total, mut percent) = parse_product(tokens, cursor)?;
    while let Some(Token::Operator(operator)) = tokens.get(*cursor) {
        if *operator != '+' && *operator != '-' {
            break;
        }
        *cursor += 1;
        let (next, next_percent) = parse_product(tokens, cursor)?;
        percent = percent || next_percent;
        total = if *operator == '+' { total + next } else { total - next };
    }
    Some((total, percent))
}

fn parse_product(tokens: &[Token], cursor: &mut usize) -> Option<(f32, bool)> {
    let (mut total, mut percent) = parse_atom(tokens, cursor)?;
    while let Some(Token::Operator(operator)) = tokens.get(*cursor) {
        if *operator != '*' && *operator != '/' {
            break;
        }
        *cursor += 1;
        let (next, next_percent) = parse_atom(tokens, cursor)?;
        percent = percent || next_percent;
        total = if *operator == '*' { total * next } else { total / next };
    }
    Some((total, percent))
}

fn parse_atom(tokens: &[Token], cursor: &mut usize) -> Option<(f32, bool)> {
    match tokens.get(*cursor)? {
        Token::Number(value, percent) => {
            *cursor += 1;
            Some((*value, *percent))
        }
        Token::Open => {
            *cursor += 1;
            let inner = parse_sum(tokens, cursor)?;
            if tokens.get(*cursor) != Some(&Token::Close) {
                return None;
            }
            *cursor += 1;
            Some(inner)
        }
        _ => None,
    }
}

// ---- Colours -----------------------------------------------------------------

/// Parses a colour: hex, `rgb()`, `oklch()`, `transparent`, or a `color-mix()` of those.
pub fn parse_colour(text: &str) -> Option<Rgba> {
    let trimmed = text.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        return parse_hex(hex);
    }
    if trimmed.eq_ignore_ascii_case("transparent") {
        return Some(Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 });
    }
    let open = trimmed.find('(')?;
    let function = trimmed[..open].trim().to_ascii_lowercase();
    let inner = trimmed[open + 1..].strip_suffix(')')?;
    match function.as_str() {
        "rgb" | "rgba" => parse_rgb(inner),
        "oklch" => parse_oklch(inner),
        "color-mix" => parse_mix(inner),
        _ => None,
    }
}

fn parse_hex(hex: &str) -> Option<Rgba> {
    let expanded: String = match hex.len() {
        3 | 4 => hex.chars().flat_map(|c| [c, c]).collect(),
        6 | 8 => hex.to_string(),
        _ => return None,
    };
    let value = u32::from_str_radix(&expanded, 16).ok()?;
    if expanded.len() == 6 {
        return Some(gpui::rgb(value));
    }
    Some(gpui::rgba(value))
}

fn components(inner: &str) -> Vec<String> {
    inner
        .replace('/', " ")
        .replace(',', " ")
        .split_whitespace()
        .map(String::from)
        .collect()
}

fn channel(text: &str) -> Option<f32> {
    if let Some(percent) = text.strip_suffix('%') {
        return percent.parse::<f32>().ok().map(|value| value / 100.0);
    }
    text.parse::<f32>().ok().map(|value| value / 255.0)
}

fn alpha(text: Option<&String>) -> Option<f32> {
    let Some(text) = text else {
        return Some(1.0);
    };
    if let Some(percent) = text.strip_suffix('%') {
        return percent.parse::<f32>().ok().map(|value| value / 100.0);
    }
    text.parse().ok()
}

fn parse_rgb(inner: &str) -> Option<Rgba> {
    let parts = components(inner);
    if parts.len() < 3 {
        return None;
    }
    Some(Rgba {
        r: channel(&parts[0])?,
        g: channel(&parts[1])?,
        b: channel(&parts[2])?,
        a: alpha(parts.get(3))?,
    })
}

fn parse_oklch(inner: &str) -> Option<Rgba> {
    let parts = components(inner);
    if parts.len() < 3 {
        return None;
    }
    let lightness = match parts[0].strip_suffix('%') {
        Some(percent) => percent.parse::<f32>().ok()? / 100.0,
        None => parts[0].parse::<f32>().ok()?,
    };
    let chroma: f32 = parts[1].parse().ok()?;
    let hue: f32 = parts[2].trim_end_matches("deg").parse().ok()?;
    let radians = hue.to_radians();
    let (a, b) = (chroma * radians.cos(), chroma * radians.sin());

    let l = (lightness + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (lightness - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (lightness - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    let linear = [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    ];
    let encode = |value: f32| {
        let clamped = value.clamp(0.0, 1.0);
        if clamped <= 0.0031308 {
            12.92 * clamped
        } else {
            1.055 * clamped.powf(1.0 / 2.4) - 0.055
        }
    };
    Some(Rgba {
        r: encode(linear[0]),
        g: encode(linear[1]),
        b: encode(linear[2]),
        a: alpha(parts.get(3))?,
    })
}

/// `color-mix(in srgb, A 30%, B)`, mixed in sRGB with premultiplied alpha.
fn parse_mix(inner: &str) -> Option<Rgba> {
    let parts = split_top_level(inner, ',');
    if parts.len() != 3 {
        return None;
    }
    let (first, first_share) = mix_operand(parts[1])?;
    let (second, second_share) = mix_operand(parts[2])?;
    let (first_share, second_share) = match (first_share, second_share) {
        (Some(a), Some(b)) => (a, b),
        (Some(a), None) => (a, 1.0 - a),
        (None, Some(b)) => (1.0 - b, b),
        (None, None) => (0.5, 0.5),
    };
    let alpha = first.a * first_share + second.a * second_share;
    if alpha <= 0.0 {
        return Some(Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 });
    }
    let mix = |a: f32, b: f32| (a * first.a * first_share + b * second.a * second_share) / alpha;
    Some(Rgba {
        r: mix(first.r, second.r),
        g: mix(first.g, second.g),
        b: mix(first.b, second.b),
        a: alpha,
    })
}

fn mix_operand(text: &str) -> Option<(Rgba, Option<f32>)> {
    let trimmed = text.trim();
    if let Some(space) = trimmed.rfind(' ') {
        if let Some(percent) = trimmed[space + 1..].strip_suffix('%') {
            let share = percent.parse::<f32>().ok()? / 100.0;
            return Some((parse_colour(&trimmed[..space])?, Some(share)));
        }
    }
    Some((parse_colour(trimmed)?, None))
}
