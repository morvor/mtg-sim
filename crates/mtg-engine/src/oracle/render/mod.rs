//! Renders compiled abilities ([`crate::ability`]) back into Oracle-style English.
//!
//! This is the reverse direction of the Oracle compiler, written independently of it: the
//! text is generated from what each AST node *means*, never from the parser's phrase
//! tables or from the source text kept on the AST for display (`AbilityDef::text`,
//! `TargetSpec::text`, `Mode::text`). Comparing the rendering with the card's Oracle text
//! ([`compare`]) shows whether the compiled abilities say what the card says.
//!
//! Every AST variant is handled by an exhaustive `match` (no catch-all arm). A node the
//! renderer can't put into words is recorded as a *gap* ([`Renderer::gap`]) and makes the
//! card a mismatch: nothing is silently dropped.

pub mod compare;
mod costs;
mod custom;
mod effects;
mod keywords;
mod nouns;
mod players;
mod statics;
mod triggers;
mod values;

use crate::ability::*;
use crate::card::{CardDef, FaceDef};
use crate::types::*;

/// What the renderer knows about the face whose abilities it renders.
#[derive(Clone, Debug, Default)]
pub struct FaceInfo {
    pub name: String,
    pub card_types: CardTypeSet,
    pub subtypes: Vec<Subtype>,
    /// What an Aura on this face enchants ("creature", "land"), for "enchanted [thing]".
    pub enchant: Option<String>,
}

impl FaceInfo {
    pub fn of(face: &FaceDef) -> FaceInfo {
        let mut info = FaceInfo {
            name: face.chars.name.to_string(),
            card_types: face.chars.card_types,
            subtypes: face.chars.subtypes.iter().cloned().collect(),
            enchant: None,
        };
        info.enchant = enchant_noun(&face.chars.abilities, &info);
        info
    }
    fn has_subtype(&self, s: &str) -> bool {
        self.subtypes.iter().any(|x| x == s)
    }
}

/// The rendering of one face.
#[derive(Clone, Debug, Default)]
pub struct RenderedFace {
    /// One entry per ability (keyword abilities one per keyword).
    pub lines: Vec<String>,
    /// AST nodes that couldn't be rendered.
    pub gaps: Vec<String>,
}

/// Renders every ability of a face.
pub fn render_face(face: &FaceDef) -> RenderedFace {
    let info = FaceInfo::of(face);
    render_abilities(&face.chars.abilities, &info)
}

/// Renders a list of abilities of an object described by `info`.
pub fn render_abilities(abilities: &[Ability], info: &FaceInfo) -> RenderedFace {
    let mut r = Renderer::new(info);
    let mut out = RenderedFace::default();
    let mut prev_changeling = false;
    for a in abilities {
        // CR 702.73a: a printed changeling's "is every creature type" CDA is the keyword's
        // meaning, compiled next to it; it isn't printed separately.
        if prev_changeling && is_changeling_cda(a) {
            prev_changeling = false;
            continue;
        }
        prev_changeling = matches!(&a.kind, AbilityKind::Keyword(k) if k.kind == crate::keywords::KeywordKind::Changeling);
        // Gift (CR 702.174): the keyword and the gift it gives are one line ("Gift a
        // card"); the gift is compiled as an ability that gives it if it was promised.
        if let Some(what) = gift_given(a) {
            if let Some(last) = out.lines.last() {
                if last == "Gift" {
                    out.lines.pop();
                }
            }
            out.lines.push(format!("Gift {what}"));
            continue;
        }
        let line = r.ability(a);
        out.lines.push(line);
    }
    merge_chapters(&mut out.lines);
    out.gaps = std::mem::take(&mut r.gaps);
    out
}

/// Renders a single ability (for tests and diagnostics). Gaps are returned as `Err`.
pub fn render_ability(a: &Ability, info: &FaceInfo) -> Result<String, Vec<String>> {
    let mut r = Renderer::new(info);
    let s = r.ability(a);
    if r.gaps.is_empty() {
        Ok(s)
    } else {
        Err(r.gaps)
    }
}

/// Renders all faces of a card.
pub fn render_card(def: &CardDef) -> Vec<RenderedFace> {
    def.faces.iter().map(render_face).collect()
}

/// The noun an Aura's enchant keyword names ("creature", "land", "player").
pub fn enchant_noun(abilities: &[Ability], info: &FaceInfo) -> Option<String> {
    for a in abilities {
        if let AbilityKind::Keyword(k) = &a.kind {
            if k.kind == crate::keywords::KeywordKind::Enchant {
                // "enchanted creature" for "Enchant creature you control".
                let mut k = k.clone();
                k.filter = k.filter.map(|f| effects::strip_controller(&f));
                let mut r = Renderer::new(info);
                let s = r.keyword_lower(&k);
                return s.strip_prefix("enchant ").map(|x| x.to_string());
            }
        }
    }
    None
}

/// What a gift ability gives: "a card" for `If(gift promised, Custom("gift:give:a card"))`.
fn gift_given(a: &Ability) -> Option<String> {
    let body = match &a.kind {
        AbilityKind::Spell(s) => &s.body,
        AbilityKind::Triggered(t) => &t.body,
        _ => return None,
    };
    if let Effect::If {
        cond: Condition::CostPaid(c),
        then,
        otherwise,
    } = &body.effect
    {
        if c == "gift" && matches!(otherwise.as_ref(), Effect::Noop) {
            if let Effect::Custom(n) = then.as_ref() {
                return n.strip_prefix("gift:give:").map(|s| s.to_string());
            }
        }
    }
    None
}

/// Whether a condition is "this Case is solved" (CR 719.3).
pub(crate) fn is_solved(c: &Condition) -> bool {
    matches!(c, Condition::Custom(n) if n == "case: is solved")
}

fn is_changeling_cda(a: &Ability) -> bool {
    matches!(&a.kind, AbilityKind::Static(s) if s.is_cda && matches!(&s.effect,
        StaticEffect::Continuous { mods, .. } if mods.len() == 1 && matches!(mods[0], Modification::AllCreatureTypes)))
}

/// Saga chapters with the same effect are printed on one line ("II, III — ...",
/// CR 714.2b).
fn merge_chapters(lines: &mut Vec<String>) {
    let mut out: Vec<String> = Vec::new();
    for l in lines.drain(..) {
        if let (Some(prev), Some((num, eff))) = (out.last_mut(), split_chapter(&l)) {
            if let Some((pnum, peff)) = split_chapter(prev) {
                if peff == eff {
                    *prev = format!("{pnum}, {num} — {eff}");
                    continue;
                }
            }
        }
        out.push(l);
    }
    *lines = out;
}

fn split_chapter(l: &str) -> Option<(String, String)> {
    let (head, rest) = l.split_once(" — ")?;
    head.split(", ")
        .all(|n| roman_value(n).is_some())
        .then(|| (head.to_string(), rest.to_string()))
}

pub(crate) fn roman(n: u32) -> String {
    const R: [(u32, &str); 9] = [
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
        (0, ""),
        (0, ""),
        (0, ""),
        (0, ""),
    ];
    let mut n = n;
    let mut s = String::new();
    for (v, t) in R {
        while v > 0 && n >= v {
            s.push_str(t);
            n -= v;
        }
    }
    s
}

fn roman_value(s: &str) -> Option<u32> {
    (1..=20).find(|n| roman(*n) == s)
}

/// Grammatical number of a noun phrase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Num {
    One,
    Many,
}

/// The renderer: holds the state of the ability being rendered (its targets, which of
/// them have been mentioned) and the gaps found so far.
pub struct Renderer<'a> {
    pub(crate) info: &'a FaceInfo,
    pub gaps: Vec<String>,
    /// Target slots of the body being rendered.
    pub(crate) targets: Vec<TargetSpec>,
    /// Which target slots have been introduced ("target creature") already; later
    /// mentions are anaphoric ("it").
    pub(crate) introduced: Vec<bool>,
    /// Nesting depth of quoted abilities (granted abilities refer to their own object as
    /// "this creature").
    pub(crate) quote_depth: u32,
    /// The zone the ability being rendered functions from.
    pub(crate) zone: FunctionZone,
    /// The object itself was the last object mentioned (a trigger "When ~ attacks"), so
    /// cards refer to it as "it": rendered as `~it`, which the comparison matches with
    /// either "~" or "it".
    pub(crate) self_salient: bool,
    /// Card types of the subject of a "becomes" effect, when known ("It's still a land").
    pub(crate) subject_types: Vec<CardType>,
    /// Rendering an "each [noun]" phrase: "each creature your opponents control".
    pub(crate) each_mode: bool,
    /// Alternatives in a head noun join with "and" ("for each instant and sorcery card").
    pub(crate) alt_and: bool,
    /// The noun for a filter that names no type: "source" for damage sources.
    pub(crate) default_head: Option<&'static str>,
    /// The previous instruction was a clash ("If you win, ...", CR 701.30).
    pub(crate) after_clash: bool,
    /// Selections stored in variables by the ability being rendered ("other creatures
    /// you control gain ..." stored, then modified): the first mention is the phrase.
    pub(crate) var_defs: Vec<(Var, Sel, bool)>,
    /// How the trigger's player is called in the ability being rendered ("that spell's
    /// controller" for a targeting trigger).
    pub(crate) trigger_player: Option<&'static str>,
    /// A hand was just revealed: a card chosen from it is "from it".
    pub(crate) revealed_hand: bool,
}

impl<'a> Renderer<'a> {
    pub fn new(info: &'a FaceInfo) -> Renderer<'a> {
        Renderer {
            info,
            gaps: Vec::new(),
            targets: Vec::new(),
            introduced: Vec::new(),
            quote_depth: 0,
            zone: FunctionZone::Battlefield,
            self_salient: false,
            subject_types: Vec::new(),
            each_mode: false,
            alt_and: false,
            default_head: None,
            after_clash: false,
            var_defs: Vec::new(),
            trigger_player: None,
            revealed_hand: false,
        }
    }

    /// Records a node that can't be rendered; returns a visible marker for the output.
    pub fn gap(&mut self, what: impl Into<String>) -> String {
        let w = what.into();
        let marker = format!("⟨{w}⟩");
        self.gaps.push(w);
        marker
    }

    /// Runs `f` with a fresh target context for `targets`, restoring the outer one after.
    pub(crate) fn with_targets<T>(
        &mut self,
        targets: &[TargetSpec],
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let saved_t = std::mem::replace(&mut self.targets, targets.to_vec());
        let saved_i = std::mem::replace(&mut self.introduced, vec![false; targets.len()]);
        let out = f(self);
        // Targets that the effect never mentions still have to appear.
        let missing: Vec<usize> = (0..self.targets.len())
            .filter(|i| !self.introduced[*i])
            .collect();
        for i in missing {
            let t = self.target_phrase(i as u8);
            self.gaps
                .push(format!("target slot {i} never mentioned ({t})"));
        }
        self.targets = saved_t;
        self.introduced = saved_i;
        out
    }

    /// The self-reference.
    /// The first mention in an ability is "~"; later ones may be "it" (`~it`).
    pub(crate) fn me(&mut self) -> String {
        if self.self_salient {
            "~it".to_string()
        } else {
            self.self_salient = true;
            "~".to_string()
        }
    }

    /// Renders an ability in a nested position (a quoted granted ability, an emblem's or a
    /// token's ability).
    pub(crate) fn nested_ability(&mut self, a: &Ability) -> String {
        let saved_t = std::mem::take(&mut self.targets);
        let saved_i = std::mem::take(&mut self.introduced);
        let saved_s = self.self_salient;
        let saved_v = std::mem::take(&mut self.var_defs);
        self.quote_depth += 1;
        let s = self.ability(a);
        self.quote_depth -= 1;
        self.var_defs = saved_v;
        self.targets = saved_t;
        self.introduced = saved_i;
        self.self_salient = saved_s;
        s
    }

    /// One ability.
    pub fn ability(&mut self, a: &Ability) -> String {
        self.self_salient = false;
        self.var_defs.clear();
        self.trigger_player = None;
        self.revealed_hand = false;
        match &a.kind {
            AbilityKind::Spell(s) => self.body(&s.body),
            AbilityKind::Activated(act) => {
                let s = self.activated(act);
                // "Exhaust — {2}{R}: ..." (CR 702.177a), "Power-up — ...", "Boast — ...":
                // keywords that modify the activated ability they introduce, recognized
                // the same way the engine recognizes them.
                match crate::keyword_impls::ability_from_keyword(a) {
                    Some(k) => format!("{} — {s}", k.name()),
                    None => s,
                }
            }
            AbilityKind::Triggered(t) => self.triggered(t),
            AbilityKind::Static(s) => self.static_ability(s),
            AbilityKind::Keyword(k) => self.keyword(k),
            AbilityKind::Unsupported(_) => self.gap("unsupported ability"),
        }
    }

    /// A body: targets + effect, or modes.
    pub(crate) fn body(&mut self, b: &Body) -> String {
        if let Some(m) = &b.modal {
            let head = self.with_targets(&b.targets, |r| {
                let e = r.effect_sentences(&b.effect);
                e
            });
            let modes = self.modal(m);
            if head.is_empty() {
                return modes;
            }
            return format!("{head} {modes}");
        }
        self.with_targets(&b.targets, |r| r.effect_sentences(&b.effect))
    }

    fn modal(&mut self, m: &Modal) -> String {
        let mut head = match &m.chooser {
            ModeChooser::Controller => "choose".to_string(),
            ModeChooser::Opponent => "an opponent chooses".to_string(),
            ModeChooser::Random => "choose".to_string(),
            ModeChooser::Pawprints(n) => format!("choose up to {n} {{P}} worth of modes"),
            ModeChooser::Unchosen { .. } => "choose".to_string(),
        };
        if !matches!(m.chooser, ModeChooser::Pawprints(_)) {
            let count = match (m.min.as_const(), m.max.as_const()) {
                (Some(a), Some(b)) if a == b => self.count_word(a),
                (Some(0), Some(b)) => format!("up to {}", self.count_word(b)),
                (Some(1), Some(2)) if m.modes.len() == 2 => "one or both".to_string(),
                (Some(1), Some(b)) if b as usize >= m.modes.len() && b > 1 => {
                    "one or more".to_string()
                }
                (Some(a), Some(b)) => format!("{} or {}", self.count_word(a), self.count_word(b)),
                (Some(1), None) if m.per_mode_cost => "one or more".to_string(),
                // "Choose one. If ~ was cast using teamwork, choose both instead."
                (Some(a), None) if matches!(&m.max, Value::If(..)) => {
                    let Value::If(c, yes, no) = &m.max else {
                        return self.gap("modal max");
                    };
                    let c = self.condition(c);
                    let no_s = match no.as_ref() {
                        Value::Const(n) if *n == a => self.count_word(*n),
                        other => self.value(other),
                    };
                    let yes_s = match yes.as_ref() {
                        Value::Const(2) if m.modes.len() == 2 => "both".to_string(),
                        Value::Const(n) if *n as usize >= m.modes.len() && a == 1 => {
                            "one or more".to_string()
                        }
                        Value::Const(n) => self.count_word(*n),
                        other => self.value(other),
                    };
                    format!("{no_s}. If {c}, choose {yes_s} instead")
                }
                _ => {
                    let v = self.value(&m.max);
                    format!("up to {v}")
                }
            };
            head = format!("{head} {count}");
            match &m.chooser {
                ModeChooser::Random => head.push_str(" at random"),
                ModeChooser::Unchosen { this_turn } => {
                    head.push_str(" that hasn't been chosen");
                    if *this_turn {
                        head.push_str(" this turn");
                    }
                }
                _ => {}
            }
        }
        let mut s = format!("{head} —");
        if m.allow_repeat {
            s = format!("{head}. You may choose the same mode more than once.");
        }
        for mode in &m.modes {
            let cost = mode.cost.as_ref().map(|c| self.cost(c));
            let text = self.with_targets(&mode.targets, |r| r.effect_sentences(&mode.effect));
            let pp = mode.pawprints();
            let mut prefix = String::new();
            if pp > 0 {
                prefix = format!("{} — ", "{P}".repeat(pp as usize));
            }
            match cost {
                Some(c) => s.push_str(&format!("\n• {prefix}{c} — {text}")),
                None => s.push_str(&format!("\n• {prefix}{text}")),
            }
        }
        s
    }

    /// "a"/"an"/"two"... for a count of objects.
    pub(crate) fn count_word(&self, n: i32) -> String {
        match n {
            1 => "one".into(),
            n => number_word(n),
        }
    }
}

/// Cardinal number words as printed on cards ("two", "ten"; larger numbers as digits).
pub fn number_word(n: i32) -> String {
    const W: [&str; 21] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
        "twenty",
    ];
    if (0..=20).contains(&n) {
        W[n as usize].to_string()
    } else {
        n.to_string()
    }
}

/// Ordinal words ("second", "third").
pub fn ordinal_word(n: u32) -> String {
    match n {
        1 => "first".into(),
        2 => "second".into(),
        3 => "third".into(),
        4 => "fourth".into(),
        5 => "fifth".into(),
        6 => "sixth".into(),
        7 => "seventh".into(),
        8 => "eighth".into(),
        9 => "ninth".into(),
        10 => "tenth".into(),
        n => format!("{n}th"),
    }
}

/// "a" or "an" before a word.
pub fn article(next: &str) -> &'static str {
    let w = next.trim_start_matches(['"', '(']).to_lowercase();
    let vowel_sound = w.starts_with(['a', 'e', 'i', 'o'])
        || (w.starts_with('u') && !w.starts_with("uni") && !w.starts_with("use"))
        || w.starts_with("x ")
        || w.starts_with("8")
        || w.starts_with("11")
        || w.starts_with("18");
    if vowel_sound && !w.starts_with("one") {
        "an"
    } else {
        "a"
    }
}

/// `a`/`an` + phrase.
pub fn with_article(phrase: &str) -> String {
    format!("{} {phrase}", article(phrase))
}

/// English plural of a noun (the last word of a phrase).
pub fn plural(phrase: &str) -> String {
    let (head, last) = match phrase.rsplit_once(' ') {
        Some((h, l)) => (format!("{h} "), l),
        None => (String::new(), phrase),
    };
    let p = plural_word(last);
    format!("{head}{p}")
}

fn plural_word(w: &str) -> String {
    let irregular = [
        ("Elf", "Elves"),
        ("Dwarf", "Dwarves"),
        ("Wolf", "Wolves"),
        ("Werewolf", "Werewolves"),
        ("Mouse", "Mice"),
        ("Ox", "Oxen"),
        ("Fungus", "Fungi"),
        ("Pegasus", "Pegasi"),
        ("Cyclops", "Cyclopes"),
        ("Plains", "Plains"),
        ("Equipment", "Equipment"),
        ("Sphinx", "Sphinxes"),
        ("Phenomenon", "Phenomena"),
        ("phenomenon", "phenomena"),
        ("Octopus", "Octopuses"),
        ("Mercenary", "Mercenaries"),
        ("Fish", "Fish"),
        ("Sheep", "Sheep"),
        ("Moonfolk", "Moonfolk"),
        ("Kithkin", "Kithkin"),
        ("Merfolk", "Merfolk"),
        ("Kor", "Kor"),
        ("Samurai", "Samurai"),
        ("Ninja", "Ninjas"),
        ("Thopter", "Thopters"),
        ("Homunculus", "Homunculi"),
    ];
    for (s, p) in irregular {
        if w == s {
            return p.to_string();
        }
    }
    if w.ends_with('}') || w.is_empty() || w == "~" {
        return w.to_string();
    }
    let lower = w.to_lowercase();
    if lower.ends_with('y')
        && !lower.ends_with("ay")
        && !lower.ends_with("ey")
        && !lower.ends_with("oy")
        && !lower.ends_with("uy")
    {
        return format!("{}ies", &w[..w.len() - 1]);
    }
    if lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with("ch")
        || lower.ends_with("sh")
        || lower.ends_with('z')
    {
        return format!("{w}es");
    }
    format!("{w}s")
}

/// Joins items as an English list: "a", "a or b", "a, b, or c".
pub fn join_list(items: &[String], conj: &str) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => format!("{} {conj} {}", items[0], items[1]),
        _ => {
            let (last, rest) = items.split_last().unwrap();
            format!("{}, {conj} {last}", rest.join(", "))
        }
    }
}

/// Capitalizes the first letter (sentence start).
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Color words in the order cards print them: WUBRG, except that a pair of colors two
/// steps or more apart around the color wheel starts from the later one ("green and
/// white", "red and white").
pub fn color_words(cs: ColorSet) -> Vec<String> {
    let mut v: Vec<Color> = cs.iter().collect();
    if v.len() == 2 {
        let i = |c: Color| Color::ALL.iter().position(|x| *x == c).unwrap_or(0);
        if i(v[1]) - i(v[0]) > 2 {
            v.swap(0, 1);
        }
    }
    v.iter().map(|c| c.word().to_string()).collect()
}

/// Mana type symbol.
pub fn mana_symbol(t: crate::mana::ManaType) -> String {
    use crate::mana::ManaType::*;
    match t {
        W => "{W}",
        U => "{U}",
        B => "{B}",
        R => "{R}",
        G => "{G}",
        C => "{C}",
    }
    .to_string()
}

pub fn color_word(c: Color) -> &'static str {
    c.word()
}

pub fn zone_word(z: ZoneKind) -> &'static str {
    match z {
        ZoneKind::Library => "library",
        ZoneKind::Hand => "hand",
        ZoneKind::Battlefield => "battlefield",
        ZoneKind::Graveyard => "graveyard",
        ZoneKind::Stack => "stack",
        ZoneKind::Exile => "exile",
        ZoneKind::Command => "command zone",
        ZoneKind::Ante => "ante",
        ZoneKind::Outside => "outside the game",
    }
}

/// A counter kind as printed: "+1/+1 counter", "loyalty counter".
pub fn counter_name(k: &str) -> String {
    format!("{k} counter")
}
