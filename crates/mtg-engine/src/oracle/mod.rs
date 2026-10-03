//! Oracle text compiler: turns Scryfall oracle text into [`crate::ability`] structures.
//!
//! Pipeline:
//! 1. **Normalize** ([`normalize`]): strip reminder text, replace self-references
//!    (the card's name, "this creature", "this spell", ...) with `~`, normalize dashes.
//! 2. **Split** into abilities: one per line; bullet lines (`•`) attach to the preceding
//!    modal line ("Choose one —").
//! 3. **Classify and parse** each ability ([`parse_ability`]): keyword lines, activated
//!    abilities (`cost: effect`), triggered abilities (`When/Whenever/At ...`), static
//!    abilities, and spell text (instants/sorceries).
//!
//! Anything not understood is kept as [`AbilityKind::Unsupported`] and reported in
//! [`Compiled::unsupported`], so card support can be measured precisely.

pub mod costs;
pub mod effects;
pub mod keywords;
pub mod patterns;
pub mod phrases;
pub mod render;
pub mod statics;
pub mod triggers;

use crate::ability::*;
use crate::card::Layout;
use crate::types::*;

/// Information about the card face being compiled.
pub struct CompileContext<'a> {
    pub card_name: &'a str,
    pub full_name: &'a str,
    pub type_line: &'a TypeLine,
    pub layout: Layout,
    pub face_index: usize,
    /// Scryfall's keyword list for the card (hints for keyword parsing).
    pub keywords: &'a [String],
    pub power: Option<&'a str>,
    pub toughness: Option<&'a str>,
}

impl CompileContext<'_> {
    pub fn is_spell(&self) -> bool {
        self.type_line.card_types.contains(CardType::Instant)
            || self.type_line.card_types.contains(CardType::Sorcery)
    }
    pub fn is_permanent(&self) -> bool {
        self.type_line.card_types.has_permanent_type()
    }
}

#[derive(Default)]
pub struct Compiled {
    pub abilities: Vec<Ability>,
    pub unsupported: Vec<String>,
    /// Blocks replaced by hand-written definitions ([`crate::cards`]).
    pub manual: Vec<String>,
}

thread_local! {
    static NO_MANUAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static RAW_TEXT: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    static CARD_NAME: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// The name of the card (face) being compiled on this thread, for phrases that name it
/// ("a creature named ~", CR 201.2) parsed where the compile context isn't at hand
/// (`FilterSuffixPattern`s). Empty when unknown.
pub fn card_name() -> String {
    CARD_NAME.with(|r| r.borrow().clone())
}

/// The raw (un-normalized) oracle text of the face being compiled on this thread.
/// Normalization merges the card's name and "this creature" into `~`; patterns that must
/// tell them apart (a quoted ability granted to another object, where "this creature" is
/// the object that gets it but the card's name is still the card) can look here.
pub fn raw_text() -> String {
    RAW_TEXT.with(|r| r.borrow().clone())
}

/// Runs `f` with hand-written definitions ([`crate::cards`]) disabled on this thread, so
/// that what the compiler parses on its own can be checked (`manual-check`).
pub fn without_manual<T>(f: impl FnOnce() -> T) -> T {
    let prev = NO_MANUAL.with(|c| c.replace(true));
    let r = f();
    NO_MANUAL.with(|c| c.set(prev));
    r
}

/// Compiles a face's oracle text.
pub fn compile(text: &str, ctx: &CompileContext) -> Compiled {
    RAW_TEXT.with(|r| *r.borrow_mut() = text.to_string());
    let prev_name = CARD_NAME.with(|r| r.replace(ctx.card_name.to_string()));
    let mut out = Compiled::default();
    let norm = normalize(text, ctx);
    let manual_ok = !NO_MANUAL.with(|c| c.get());
    for block in crate::oracle_ext::group_blocks(split_abilities(&norm), ctx) {
        // A genuinely unique ability written by hand (compile-first policy, see `cards`).
        if let Some(m) = manual_ok
            .then(|| crate::cards::lookup(ctx.full_name, ctx.face_index, &block))
            .flatten()
        {
            out.abilities.append(&mut (m.build)(ctx));
            out.manual.push(block);
            continue;
        }
        // A pronoun left without an antecedent, or an instruction to repeat a process
        // outside any process, means the text wasn't understood.
        let parsed = parse_ability(&block, ctx).filter(|v| {
            !v.iter().any(|a| {
                patterns::oracle_hardening_referents::has_no_referent(a)
                    || crate::repeat_process::has_stray_repeat(a)
                    || patterns::filters_relational::unresolved(a)
            })
        });
        match parsed {
            Some(mut abilities) => out.abilities.append(&mut abilities),
            None => {
                out.unsupported.push(block.clone());
                out.abilities.push(AbilityDef::new(
                    AbilityKind::Unsupported(block.clone()),
                    block,
                ));
            }
        }
    }
    // Characteristic-defining abilities implied by the type line/P/T.
    if let Some(cda) = statics::star_pt_cda(&norm, ctx) {
        // Replace an unsupported CDA line if the star P/T was handled.
        out.abilities.push(cda);
    }
    CARD_NAME.with(|r| *r.borrow_mut() = prev_name);
    out
}

/// Normalizes oracle text for parsing.
pub fn normalize(text: &str, ctx: &CompileContext) -> String {
    let mut s = strip_reminder(text);
    s = s
        .replace('\u{2212}', "-")
        .replace('\u{2014}', "—")
        .replace('\u{2019}', "'")
        .replace('\u{201C}', "\"")
        .replace('\u{201D}', "\"")
        // Older wording, still in the Oracle text of a few playtest cards (CR 202.3).
        .replace("converted mana cost", "mana value")
        // A comma inside a closing quote before "where X is" belongs to the sentence:
        // `tokens with "[ability]," where X is ...` (Vren, the Relentless).
        .replace(",\" where X is ", "\", where X is ");
    // Self references.
    let mut names: Vec<String> = vec![ctx.card_name.to_string()];
    if ctx.full_name != ctx.card_name {
        names.push(ctx.full_name.to_string());
    }
    if ctx.type_line.supertypes.contains(Supertype::Legendary) {
        if let Some((short, _)) = ctx.card_name.split_once(',') {
            names.push(short.to_string());
        }
        if let Some((short, _)) = ctx.card_name.split_once(" of ") {
            // "General Kudro" (General Kudro of Drannith): a title and a name.
            let titled = short
                .split_once(' ')
                .is_some_and(|(title, name)| is_title(title) && !name.contains(' '));
            if !short.contains(' ') || titled {
                names.push(short.to_string());
            }
        }
        // A regnal number ("King Darien XLVIII" is "King Darien").
        if let Some((short, num)) = ctx.card_name.rsplit_once(' ') {
            if short.contains(' ') && num.len() > 1 && num.chars().all(|c| "IVXLCDM".contains(c)) {
                names.push(short.to_string());
            }
        }
    }
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for n in &names {
        if !n.is_empty() {
            s = s.replace(n.as_str(), "~");
        }
    }
    // Legendary cards are also called by the first word(s) of their name ("Whenever Edgar
    // attacks" on Edgar Markov, "Zur" for Zur the Enchanter, "Jedit Ojanen" for Jedit
    // Ojanen of Efrava): the longest such prefix first.
    if let Some(first) = short_first_name(ctx) {
        let words: Vec<&str> = ctx.card_name.split(' ').collect();
        for k in (2..words.len()).rev() {
            let prefix = words[..k].join(" ");
            if words[k - 1]
                .chars()
                .next()
                .is_some_and(|c| c.is_uppercase())
            {
                s = replace_word(&s, &prefix, "~");
            }
        }
        s = replace_word(&s, first, "~");
    }
    const SELF_REFS: [&str; 23] = [
        "this creature",
        "this artifact",
        "this enchantment",
        "this land",
        "this permanent",
        "this spell",
        "this card",
        "this Aura",
        "this Equipment",
        "this Vehicle",
        "this token",
        "this planeswalker",
        "this Saga",
        "this battle",
        "this Class",
        "this Case",
        "this Room",
        "this Fortification",
        "this Spacecraft",
        "this Siege",
        "this Mount",
        "this object",
        "this scheme",
    ];
    for r in SELF_REFS {
        s = replace_ci(&s, r, "~");
    }
    s
}

/// A title before a legendary character's name ("General Kudro").
fn is_title(w: &str) -> bool {
    matches!(
        w,
        "General" | "Captain" | "Lord" | "Lady" | "King" | "Queen" | "Sir" | "Doctor"
    )
}

/// The first word of a legendary card's name when it can stand for the card: not a
/// subtype ("Ajani", "Sliver"), a title ("Captain", "General") or an article.
fn short_first_name<'a>(ctx: &CompileContext<'a>) -> Option<&'a str> {
    if !ctx.type_line.supertypes.contains(Supertype::Legendary) || ctx.card_name.contains(',') {
        return None;
    }
    let (first, _) = ctx.card_name.split_once(' ')?;
    let ok = first.chars().count() >= 3
        && first.chars().next().is_some_and(|c| c.is_uppercase())
        && first
            .chars()
            .all(|c| c.is_alphabetic() || c == '-' || c == '\'')
        && !first.ends_with("'s")
        && !matches!(
            first,
            "The"
                | "Captain"
                | "General"
                | "Lord"
                | "Lady"
                | "King"
                | "Queen"
                | "Space"
                | "Lander"
                | "Marit"
                | "Mitotic"
                | "Doctor"
                | "Professor"
                | "Sir"
        )
        && crate::types::subtype_kind(first).is_none();
    ok.then_some(first)
}

/// Replaces whole-word, case-sensitive occurrences of `word` ("Edgar" but not
/// "Edgarian"; "Edgar's" is fine).
fn replace_word(s: &str, word: &str, rep: &str) -> String {
    let is_word = |c: char| c.is_alphanumeric() || c == '-';
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(pos) = s[i..].find(word) {
        let start = i + pos;
        let end = start + word.len();
        let before = s[..start].chars().next_back();
        let after = s[end..].chars().next();
        // Part of a longer proper name ("a token named Tuktuk the Returned").
        let rest = &s[end..];
        let rest = rest
            .strip_prefix(" the ")
            .or_else(|| rest.strip_prefix(" of "))
            .or_else(|| rest.strip_prefix(' '))
            .unwrap_or("");
        let longer_name = rest.chars().next().is_some_and(|c| c.is_uppercase());
        out.push_str(&s[i..start]);
        if before.is_some_and(|c| is_word(c) || c == '\'')
            || after.is_some_and(is_word)
            || longer_name
        {
            out.push_str(word);
        } else {
            out.push_str(rep);
        }
        i = end;
    }
    out.push_str(&s[i..]);
    out
}

fn replace_ci(s: &str, pat: &str, rep: &str) -> String {
    let lower = s.to_lowercase();
    let lp = pat.to_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(pos) = lower[i..].find(&lp) {
        let start = i + pos;
        let end = start + lp.len();
        // Word boundary after.
        let next = lower[end..].chars().next();
        if next.is_some_and(|c| c.is_alphanumeric()) {
            out.push_str(&s[i..end]);
            i = end;
            continue;
        }
        out.push_str(&s[i..start]);
        out.push_str(rep);
        i = end;
    }
    out.push_str(&s[i..]);
    out
}

/// Removes parenthesized reminder text.
pub fn strip_reminder(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    // Clean up spaces left behind.
    out.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Splits normalized text into ability blocks; bullets join their modal header.
pub fn split_abilities(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        if l.starts_with('•') && !out.is_empty() {
            let last = out.last_mut().unwrap();
            last.push('\n');
            last.push_str(l);
        } else {
            out.push(l.to_string());
        }
    }
    out
}

/// Parses one ability block. Returns None if not understood.
pub fn parse_ability(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    // Ability words (CR 207.2c) have no rules meaning: "Landfall — Whenever ...".
    let text = strip_ability_word(text);
    // Pluggable whole-ability patterns (level up, class levels, sagas, ...).
    if let Some(v) = crate::oracle_ext::parse_ability_ext(text, ctx) {
        return Some(v);
    }
    // Keyword lines.
    if let Some(kws) = keywords::parse_keyword_line(text, ctx) {
        return Some(kws);
    }
    // Activated abilities: "cost: effect".
    if let Some((cost_s, eff_s)) = split_cost(text) {
        if let Some(a) = parse_activated(cost_s, eff_s, text, ctx) {
            return Some(vec![a]);
        }
        return None;
    }
    // Triggered abilities.
    let lower = text.to_lowercase();
    if lower.starts_with("when ") || lower.starts_with("whenever ") || lower.starts_with("at ") {
        if let Some(a) = triggers::parse_triggered(text, ctx) {
            return Some(vec![a]);
        }
        // An instant or sorcery's "Whenever a creature attacks this turn, ..." is a spell
        // ability creating a delayed triggered ability (CR 603.7b).
        if !ctx.is_spell() {
            return None;
        }
    }
    // Spell abilities for instants and sorceries.
    if ctx.is_spell() {
        if let Some(a) = statics::parse_spell_static(text, ctx) {
            return Some(vec![a]);
        }
        let body = effects::other_than_sole_target(effects::parse_body(text, ctx)?);
        return Some(vec![AbilityDef::new(
            AbilityKind::Spell(SpellAbility { body }),
            text,
        )]);
    }
    // Static abilities.
    statics::parse_static(text, ctx).map(|v| v.into_iter().collect())
}

/// Strips a leading ability word ("Landfall — ", "Threshold — ").
pub fn strip_ability_word(text: &str) -> &str {
    if let Some((head, rest)) = text.split_once(" — ") {
        let words = head.split_whitespace().count();
        // Flavor words can be longer ("Lord of the Pyrrhian Legions — Whenever ..."):
        // up to six words before a triggered ability (not a list of Saga chapters).
        // So can one before an activated ability with a mana cost ("I've Come Up with a
        // New Recipe! — {1}{G}{U}, {T}: ...").
        let long_flavor_word = !head.contains(',')
            && ((words <= 6 && (rest.starts_with("When") || rest.starts_with("At ")))
                || (words <= 8 && rest.starts_with('{')));
        let looks_like_word = (words <= 4 || long_flavor_word)
            // An ability word starts its line: not a mode's name on a later line
            // ("Tiered\n• Thunder — {0} — ...", CR 702.183a).
            && !head.contains('\n')
            && !head.contains(':')
            && !head.to_lowercase().starts_with("choose")
            && head.chars().next().is_some_and(|c| c.is_uppercase())
            && !head.contains('{')
            // Not a dash inside a quoted ability ("All creatures have "Boast — ...").
            && !head.contains('"')
            // "Companion — [condition]" is a keyword, not an ability word (CR 702.139a);
            // so are "Forecast — [activated ability]" (CR 702.57a) and "Max speed —
            // [ability]" (CR 702.178a).
            && head != "Companion"
            && head != "Forecast"
            && head != "Max speed"
            // Nor is "Power-up — [cost]: [effect]" (CR 702.193a).
            && head != "Power-up"
            // Nor is "Boast — [activated ability]" (CR 702.142a), nor "Exhaust —
            // [activated ability]" (CR 702.177a).
            && head != "Boast"
            && head != "Exhaust"
            // Nor are a Case's "To solve — [Condition]" and "Solved — [Ability]"
            // (CR 719.3).
            && head != "To solve"
            && head != "Solved";
        if looks_like_word {
            return rest;
        }
    }
    text
}

/// Splits "cost: effect" at the first top-level colon (not inside quotes).
pub fn split_cost(text: &str) -> Option<(&str, &str)> {
    let mut in_quote = false;
    for (i, ch) in text.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            ':' if !in_quote => {
                let (c, e) = (&text[..i], &text[i + 1..]);
                // Heuristic: costs are short and don't start with trigger words.
                let cl = c.to_lowercase();
                if cl.starts_with("when") || cl.starts_with("at ") || c.len() > 120 {
                    return None;
                }
                return Some((c.trim(), e.trim()));
            }
            _ => {}
        }
    }
    None
}

fn parse_activated(cost_s: &str, eff_s: &str, full: &str, ctx: &CompileContext) -> Option<Ability> {
    // An amount chosen as the cost is paid ("Remove one or more +1/+1 counters from ~") is
    // the ability's X (see `patterns::cost_parts`).
    let amount_x = patterns::cost_parts::amount_as_x(cost_s);
    let cost_s = amount_x.as_ref().map_or(cost_s, |(c, _)| c.as_str());
    let (cost, loyalty) = costs::parse_cost(cost_s)?;
    // Activation restrictions at the end of the effect text — or, for a modal ability,
    // at the end of its header line ("{G}: Choose one. Activate only once each turn.").
    let modal_restricted = eff_s.split_once('\n').and_then(|(head, modes)| {
        let (h, timing, max, any) = costs::split_activation_restrictions(head);
        (h.len() < head.trim().len()).then(|| (format!("{h}\n{modes}"), timing, max, any))
    });
    let (eff_text, timing, max_per_turn, any_player) = match &modal_restricted {
        Some((t, timing, max, any)) => (t.as_str(), *timing, *max, *any),
        None => costs::split_activation_restrictions(eff_s),
    };
    // "This ability costs {1} less to activate for each ..." (CR 602.2b, 601.2f).
    let (eff_text, own_cost) =
        match patterns::activation_cost_modifiers::split_own_cost_sentence(eff_text) {
            Some((head, sentence)) => (head, Some(sentence)),
            None => (eff_text, None),
        };
    // "This ability can't be copied." (CR 113.6g, 707.10).
    let (eff_text, cant_be_copied) = match patterns::rule_statics::strip_cant_be_copied(eff_text) {
        Some(rest) => (rest, true),
        None => (eff_text.to_string(), false),
    };
    let eff_text = eff_text.as_str();
    // "X can't be 0." (CR 107.3a): a condition on the value announced for X in the cost.
    let cost_has_x = cost.mana.as_ref().is_some_and(|m| m.has_x())
        || cost.parts.iter().any(crate::casting::cost_part_has_x);
    let x_not_zero = cost_has_x
        .then(|| patterns::r107_x_cant_be_zero::strip(eff_text))
        .flatten();
    let eff_text = x_not_zero.as_deref().unwrap_or(eff_text);
    // "Spend only black mana on X." (see `payment_rules.rs`): a rule about paying the X in
    // the cost.
    let x_spend = cost_has_x
        .then(|| patterns::payment_rules::strip_x_spend(eff_text))
        .flatten();
    let eff_text = x_spend.as_ref().map_or(eff_text, |(t, _)| t.as_str());
    // "Remove any number of +1/+1 counters from ~: Create that many ... tokens."
    let amount_eff = amount_x
        .as_ref()
        .and_then(|_| patterns::cost_parts::effect_with_amount(eff_text));
    let eff_text = amount_eff.as_deref().unwrap_or(eff_text);
    // CR 400.7j: "the exiled card" is the card the cost exiled.
    // CR 107.3a, 107.3k: an X in the activation cost defines X for the ability.
    let x = patterns::value_grammar::cost_has_x(cost_s);
    let body = patterns::cost_parts::with_amount_x(amount_x.is_some(), || {
        patterns::value_grammar::with_x_defined(x, || {
            // "Sacrifice another creature: Draw X cards, where X is that creature's
            // power.": the sacrificed creature (see `patterns::value_results`).
            let sac_text = patterns::value_results::cost_sacrificed_text(&cost, eff_text);
            let eff_text = sac_text.as_deref().unwrap_or(eff_text);
            match crate::zones::cost_exiled_text(&cost, eff_text) {
                Some(text) => {
                    effects::parse_body_with_it(&text, ctx, Sel::Var(crate::zones::COST_EXILED))
                }
                None => effects::parse_body(eff_text, ctx),
            }
        })
    })?;
    // CR 605.1a: no target, could add mana, not a loyalty ability, and neither its cost
    // nor its effect moves a card to or from a library.
    let is_mana = effects::is_mana_effect(&body.effect)
        && body.targets.is_empty()
        && !loyalty
        && !touches_library(&cost, &body.effect);
    // Target slots of one target each, for "if it targets ..." (0 if there are others).
    let target_slots = if body
        .targets
        .iter()
        .all(|t| t.max.as_const() == Some(1) && t.fixed_min() == Some(1))
    {
        body.targets.len()
    } else {
        0
    };
    let mut act = ActivatedAbility::new(cost, body);
    if let Some(sentence) = own_cost {
        act.own_cost_changes.push(
            patterns::activation_cost_modifiers::parse_own_cost_change_n(
                &sentence,
                target_slots,
                ctx,
            )?,
        );
    }
    act.timing = timing;
    act.max_per_turn = max_per_turn;
    act.is_loyalty = loyalty;
    act.is_mana_ability = is_mana;
    act.any_player = any_player;
    act.cant_be_copied = cant_be_copied;
    act.zone = activated_zone(cost_s, eff_text);
    if let Some((_, Some(c))) = amount_x {
        act.condition = Some(match act.condition.take() {
            Some(e) => Condition::And(vec![e, c]),
            None => c,
        });
    }
    if x_not_zero.is_some() {
        act.condition = Some(patterns::r107_x_cant_be_zero::condition(
            act.condition.take(),
        ));
    }
    if let Some((_, rule)) = x_spend {
        act.own_cost_changes.push(OwnCostChange {
            change: CostChange::Rule(rule),
            condition: None,
        });
    }
    Some(AbilityDef::new(AbilityKind::Activated(act), full))
}

/// `s` without its quoted parts: an ability granted in quotes ("creature cards in your
/// graveyard gain \"You may cast this card from your graveyard\"") says nothing about
/// where the ability granting it functions.
pub(crate) fn without_quotes(s: &str) -> String {
    s.split('"').step_by(2).collect::<Vec<_>>().join("\"\"")
}

/// Where an activated ability functions: one whose cost can be paid only from the hand
/// ("Exile this card from your hand", "Discard this card") functions from the hand
/// (CR 113.6j); one whose cost or effect moves the object out of a zone ("Return this card
/// from your graveyard to the battlefield") functions only in that zone (CR 113.6m).
fn activated_zone(cost: &str, effect: &str) -> FunctionZone {
    let (c, e) = (cost.to_lowercase(), without_quotes(&effect.to_lowercase()));
    let moves_self_from = |s: &str, zone: &str| {
        ["~", "this card", "this creature"].iter().any(|me| {
            s.match_indices(&format!("{me} from your {zone}"))
                .any(|(i, _)| {
                    // Not "counters on ~ from your graveyard" (Wishing Well: "... card
                    // with mana value equal to the number of coin counters on ~ from
                    // your graveyard"), where the zone is another card's.
                    let before = s[..i].trim_end();
                    !(before.ends_with(" on") || before.ends_with(" of"))
                })
        })
    };
    // "Put ~ from exile onto the battlefield", "Return ~ and target land card from your
    // graveyard to the battlefield" (see `patterns::zone_move_grammar`).
    if let Some(z) = patterns::zone_move_grammar::self_move_zone(&e) {
        return z;
    }
    if moves_self_from(&c, "hand") || c.contains("discard ~") || c.contains("discard this card") {
        FunctionZone::Hand
    } else if moves_self_from(&c, "graveyard") || moves_self_from(&e, "graveyard") {
        FunctionZone::Graveyard
    } else {
        FunctionZone::Battlefield
    }
}

/// Whether a cost or effect moves cards to or from a library (drawing, milling, searching,
/// surveilling, putting cards into a library, ...).
fn touches_library(cost: &Cost, effect: &Effect) -> bool {
    let text = format!(
        "{} {}",
        serde_json::to_string(cost).unwrap_or_default(),
        serde_json::to_string(effect).unwrap_or_default()
    );
    ["Library", "Draw", "Mill", "Search", "Surveil", "Explore"]
        .iter()
        .any(|w| text.contains(w))
}

/// Shifts `Sel::Target(i)` references by `offset` (used when combining spell bodies).
pub fn offset_targets(e: &Effect, offset: u8) -> Effect {
    if offset == 0 {
        return e.clone();
    }
    let json = serde_json::to_value(e).unwrap();
    let shifted = shift_json(json, offset);
    serde_json::from_value(shifted).unwrap_or_else(|_| e.clone())
}

fn shift_json(v: serde_json::Value, offset: u8) -> serde_json::Value {
    use serde_json::Value as J;
    match v {
        J::Object(mut m) => {
            if m.len() == 1 {
                if let Some(J::Number(n)) = m.get("Target") {
                    let k = n.as_u64().unwrap_or(0) + offset as u64;
                    m.insert("Target".into(), J::Number(k.into()));
                    return J::Object(m);
                }
            }
            J::Object(
                m.into_iter()
                    .map(|(k, v)| (k, shift_json(v, offset)))
                    .collect(),
            )
        }
        J::Array(a) => J::Array(a.into_iter().map(|x| shift_json(x, offset)).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_reminder_text() {
        assert_eq!(
            strip_reminder("Flying (This creature can't be blocked.)"),
            "Flying"
        );
    }

    #[test]
    fn splits_cost() {
        assert_eq!(split_cost("{T}: Add {G}."), Some(("{T}", "Add {G}.")));
        assert_eq!(split_cost("Whenever you gain life, draw: nope"), None);
    }
}
