//! Token creation with full token descriptions (CR 111.3, 111.4):
//!
//! ```text
//! [you] create COUNT [tapped] DESC token(s) TAIL*          ("create" clause)
//! [you] create a number of DESC tokens TAIL* equal to VALUE
//! [you] create DESC token and DESC token, ...                (several kinds at once)
//! DESC  := [N/N] (color | "colorless" | "and")* supertype* card-type+ subtype*
//! TAIL  := with ABILITY ("," ABILITY)* [and ABILITY]           (keywords or quoted abilities)
//!        | named NAME | that's all colors | that's/that are tapped and attacking
//! ```
//!
//! Quoted abilities are compiled recursively; in them "this creature"/"this token" is the
//! token (normalized to `~`). A quote that names the card itself (the card's name also
//! normalizes to `~`) is left unsupported. A creature token needs a P/T, or a
//! characteristic-defining ability that sets it; a noncreature token only has a P/T when
//! it's a Vehicle (CR 301.7c).
//!
//! Follow-up sentences about the tokens just created ("It has "..."", "They gain haste
//! until end of turn", "attach ~ to it") refer to those tokens (`vars::CREATED`).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{keyword_mods, Builder};
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;
use smol_str::SmolStr;

/// The quoted texts of the face being compiled, normalized (original case).
fn normalized_quotes(ctx: &CompileContext) -> Vec<String> {
    let raw = crate::oracle::raw_text();
    let mut out = Vec::new();
    let mut rest = raw.as_str();
    while let Some(i) = rest.find(['"', '\u{201C}']) {
        let open_len = rest[i..].chars().next().map_or(1, char::len_utf8);
        let after = &rest[i + open_len..];
        let Some(j) = after.find(['"', '\u{201D}']) else {
            break;
        };
        out.push(crate::oracle::normalize(&after[..j], ctx));
        let close_len = after[j..].chars().next().map_or(1, char::len_utf8);
        rest = &after[j + close_len..];
    }
    out
}

/// Compiles a quoted ability of a token (lowercase text as it appears in the effect).
/// `~` in it means the token. Fails if the quote names the card itself, or if any part
/// isn't understood.
pub(crate) fn token_quote_abilities(
    q_lower: &str,
    types: &[CardType],
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let want = q_lower.trim();
    let orig = normalized_quotes(ctx)
        .into_iter()
        .find(|q| q.to_lowercase().trim() == want)?;
    if super::statics::quote_names_card(&orig, ctx) {
        return None;
    }
    let mut tl = TypeLine::default();
    for t in types {
        tl.card_types.insert(*t);
    }
    let tctx = CompileContext {
        card_name: "\u{1}",
        full_name: "\u{1}",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let blocks = crate::oracle::split_abilities(&orig);
    if blocks.len() != 1 {
        return None;
    }
    let v = crate::oracle::parse_ability(&blocks[0], &tctx)?;
    if v.is_empty()
        || v.iter()
            .any(|a| matches!(a.kind, AbilityKind::Unsupported(_)))
    {
        return None;
    }
    Some(v)
}

/// "flying", "flying and "~ can't block."", "toxic 1 and "..."", "haste, "A," and "B"":
/// the abilities a token is created with. Quotes in `masked` are `"#k"` placeholders for
/// `quotes[k]`.
pub(crate) fn ability_list(
    masked: &str,
    quotes: &[String],
    types: &[CardType],
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let mut out = Vec::new();
    let mut keywords = Vec::new();
    let s = masked.trim().replace(", and ", ", ").replace(" and ", ", ");
    for item in s.split(", ") {
        let item = item.trim().trim_end_matches(',');
        if item.is_empty() {
            return None;
        }
        if let Some(k) = item
            .strip_prefix("\"#")
            .and_then(|x| x.strip_suffix('"'))
            .and_then(|x| x.parse::<usize>().ok())
        {
            out.extend(token_quote_abilities(quotes.get(k)?, types, ctx)?);
        } else {
            if item.contains('"') {
                return None;
            }
            keywords.push(item.to_string());
        }
    }
    if !keywords.is_empty() {
        for m in keyword_mods(&keywords.join(", "))? {
            if let Modification::AddKeyword(k) = m {
                out.push(AbilityDef::new(
                    AbilityKind::Keyword(k.clone()),
                    k.kind.name(),
                ));
            }
        }
    }
    Some(out)
}

/// A parsed token description.
pub(crate) struct TokenDesc {
    pub spec: TokenSpec,
    /// "that's tapped and attacking".
    pub attacking: bool,
}

/// Whether an ability is a characteristic-defining ability that sets power and toughness.
fn sets_pt(a: &Ability) -> bool {
    match &a.kind {
        AbilityKind::Static(s) => match &s.effect {
            StaticEffect::Continuous { mods, .. } => mods
                .iter()
                .any(|m| matches!(m, Modification::CdaPT(Some(_), Some(_)))),
            _ => false,
        },
        _ => false,
    }
}

/// The original-case token name after "named " in the face's oracle text whose
/// normalized, lowercased form is `n` ("named Ballistic Boulder"). The card's own name
/// in a token's name normalizes to `~` ("Kobolds of Kher Keep" on Kher Keep, "Koma's
/// Coil" on Koma), so candidates are compared after normalizing them the same way; the
/// longest match wins ("Kobolds of Kher" also normalizes to "Kobolds of ~" on the
/// legendary Kher Keep, whose first word stands for it).
fn original_name(n: &str, ctx: &CompileContext) -> Option<String> {
    let raw = crate::oracle::raw_text();
    let rl = raw.to_lowercase();
    if rl.len() != raw.len() {
        return None;
    }
    let max_words = n.split(' ').count() + 4;
    let mut from = 0;
    while let Some(i) = rl[from..].find("named ") {
        let start = from + i + "named ".len();
        from = start;
        let after = &raw[start..];
        let mut end_at = 0;
        let mut best = None;
        for _ in 0..max_words {
            let word_end = after[end_at..]
                .find(char::is_whitespace)
                .map_or(after.len(), |k| end_at + k);
            let cand = after[..word_end].trim_end_matches(['.', ',', ';', '"', '\u{201D}']);
            if crate::oracle::normalize(cand, ctx).to_lowercase() == n {
                best = Some(cand.to_string());
            }
            let Some(ws) = after[word_end..].chars().next() else {
                break;
            };
            if matches!(ws, '\n' | '\r') {
                break;
            }
            end_at = word_end + ws.len_utf8();
        }
        if best.is_some() {
            return best;
        }
    }
    None
}

/// Parses "1/1 white Soldier creature token with flying named Wasp that's tapped and
/// attacking" (lowercase, quotes included).
pub(crate) fn token_desc(s: &str, ctx: &CompileContext) -> Option<TokenDesc> {
    let (masked, quotes) = super::statics::mask_quotes(s.trim())?;
    let mut rest: &str = masked.trim();
    // Power/toughness.
    let (mut power, mut toughness) = (None, None);
    let (w, r) = split_word(rest);
    if let Some((p, t)) = w.split_once('/') {
        power = Some(p.parse::<i32>().ok()?);
        toughness = Some(t.parse::<i32>().ok()?);
        rest = r;
    }
    let mut colors = ColorSet::NONE;
    let mut types: Vec<CardType> = Vec::new();
    let mut subtypes: Vec<Subtype> = Vec::new();
    let mut supertypes: Vec<Supertype> = Vec::new();
    loop {
        let (w, r) = split_word(rest);
        let w = w.trim_end_matches(',');
        if w.is_empty() {
            return None;
        }
        rest = r;
        if w == "token" || w == "tokens" {
            break;
        }
        if w == "and" || w == "colorless" {
            continue;
        }
        if let Some(c) = Color::from_word(w) {
            if !types.is_empty() || !subtypes.is_empty() {
                return None;
            }
            colors.insert(c);
        } else if let Some(st) = Supertype::from_word(w) {
            supertypes.push(st);
        } else if let Some(ct) = CardType::from_word(w) {
            types.push(ct);
        } else if let Some(sub) = subtype_word(w) {
            subtypes.push(sub);
        } else {
            return None;
        }
    }
    if types.is_empty() {
        return None;
    }
    // Subtypes must go with the card types (CR 205.3d).
    for sub in &subtypes {
        let ok = match subtype_kind(sub.as_str()) {
            Some(SubtypeKind::Creature) => types
                .iter()
                .any(|t| matches!(t, CardType::Creature | CardType::Kindred)),
            Some(SubtypeKind::Artifact) => types.contains(&CardType::Artifact),
            Some(SubtypeKind::Enchantment) => types.contains(&CardType::Enchantment),
            Some(SubtypeKind::Land) => types.contains(&CardType::Land),
            _ => false,
        };
        if !ok {
            return None;
        }
    }
    let mut abilities: Vec<Ability> = Vec::new();
    let mut name = SmolStr::default();
    let mut attacking = false;
    let mut tapped = false;
    let mut rest = rest.trim().to_string();
    // Tail parts, in any order.
    const BOUNDS: [&str; 5] = [
        " with ",
        " named ",
        " that's ",
        " that are ",
        " and that's ",
    ];
    let next_bound = |s: &str| -> usize {
        BOUNDS
            .iter()
            .filter_map(|b| s.find(b))
            .min()
            .unwrap_or(s.len())
    };
    let mut seen_with = false;
    let mut seen_name = false;
    while !rest.is_empty() {
        let padded = format!(" {rest}");
        if let Some(r) = padded.strip_prefix(" with ") {
            if seen_with {
                return None;
            }
            seen_with = true;
            let i = next_bound(r);
            abilities.extend(ability_list(&r[..i], &quotes, &types, ctx)?);
            rest = r[i..].trim().to_string();
        } else if let Some(r) = padded.strip_prefix(" named ") {
            if seen_name {
                return None;
            }
            seen_name = true;
            let i = next_bound(r);
            let n = r[..i].trim();
            if n.is_empty() || n.contains(['"', ',']) || n.split(' ').count() > 4 {
                return None;
            }
            name = SmolStr::new(original_name(n, ctx)?);
            rest = r[i..].trim().to_string();
        } else if let Some(r) = ["that's all colors", "that are all colors"]
            .iter()
            .find_map(|p| rest.strip_prefix(p))
        {
            colors = ColorSet::ALL;
            rest = r.trim().to_string();
        } else if let Some(r) = [
            "that's tapped and attacking",
            "that are tapped and attacking",
            "and that's tapped and attacking",
        ]
        .iter()
        .find_map(|p| rest.strip_prefix(p))
        {
            if attacking {
                return None;
            }
            attacking = true;
            tapped = true;
            rest = r.trim().to_string();
        } else {
            return None;
        }
    }
    let is_creature = types.contains(&CardType::Creature);
    if is_creature {
        if power.is_none() && !abilities.iter().any(sets_pt) {
            return None;
        }
    } else if power.is_some() && !subtypes.iter().any(|s| s.as_str() == "Vehicle") {
        return None;
    }
    if attacking && !is_creature {
        return None;
    }
    let _ = tapped;
    Some(TokenDesc {
        spec: TokenSpec {
            name,
            colors,
            supertypes,
            card_types: types,
            subtypes,
            power,
            toughness,
            abilities,
            scryfall_name: None,
        },
        attacking,
    })
}

/// "a Food token", "a 1/1 white Soldier creature token", "two Treasure tokens" → one
/// creation (spec, count, tapped, attacking).
fn one_creation(r: &str, ctx: &CompileContext) -> Option<(TokenSpec, Value, bool, bool)> {
    let (count, r) = parse_number(r)?;
    let r = r.trim();
    let (r, tapped) = match r.strip_prefix("tapped ") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let (w, rest) = split_word(r);
    if let Some(spec) = crate::tokens::predefined(w) {
        let rest = end(rest);
        if rest == "token" || rest == "tokens" {
            return Some((spec, count, tapped, false));
        }
    }
    let d = token_desc(r, ctx)?;
    Some((d.spec, count, tapped || d.attacking, d.attacking))
}

fn create(spec: TokenSpec, count: Value, tapped: bool, attacking: bool) -> Effect {
    Effect::CreateToken {
        spec,
        count,
        controller: PlayerRef::You,
        tapped,
        attacking,
    }
}

/// "[you] create a 1/1 black Rat creature token with "~ can't block."", "create a 1/1 red
/// Goblin creature token that's tapped and attacking", "create a number of Food tokens
/// equal to the number of opponents you have", "create a 1/1 white Human creature token
/// and a Food token".
fn create_described(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("you create ")
        .or_else(|| l.strip_prefix("create "))?;
    // "a number of [tokens] equal to [value]".
    if let Some(x) = r.strip_prefix("a number of ") {
        let (masked, _) = super::statics::mask_quotes(x)?;
        let i = masked.rfind(" equal to ")?;
        // The quotes are masked: find the same split point in the unmasked text.
        let value_s = &masked[i + " equal to ".len()..];
        if value_s.contains('"') {
            return None;
        }
        let desc = &x[..x.len() - value_s.len() - " equal to ".len()];
        let (spec, _, tapped, attacking) = one_creation(&format!("a {desc}"), b.ctx)?;
        let (v, tail) = super::r107_numbers::value_phrase(value_s, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        return Some(create(spec, v, tapped, attacking));
    }
    if let Some((spec, count, tapped, attacking)) = one_creation(r, b.ctx) {
        return Some(create(spec, count, tapped, attacking));
    }
    // Several kinds: "a 1/1 ... token, a 1/1 ... token, and a 1/1 ... token".
    let (masked, quotes) = super::statics::mask_quotes(r)?;
    let norm = masked.replace(", and ", ", ").replace(" and a ", ", a ");
    let norm = norm.replace(" and an ", ", an ");
    let parts: Vec<&str> = norm.split(", ").collect();
    if parts.len() < 2 {
        return None;
    }
    let mut out = Vec::new();
    for p in parts {
        if !(p.starts_with("a ") || p.starts_with("an ")) {
            return None;
        }
        // Restore the quotes of this part.
        let mut part = p.to_string();
        for (k, q) in quotes.iter().enumerate() {
            part = part.replace(&format!("\"#{k}\""), &format!("\"{q}\""));
        }
        let (spec, count, tapped, attacking) = one_creation(&part, b.ctx)?;
        out.push(create(spec, count, tapped, attacking));
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "tokens_copies: create described tokens", priority: 90, parse: create_described } }

// ---------------------------------------------------------------------------
// Follow-ups about the tokens just created
// ---------------------------------------------------------------------------

/// The last token-creating effect of `e`, if it's the last thing `e` does (looking
/// through sequences and "if you do" / "you may" branches).
pub(crate) fn last_create(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_create),
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. } => Some(e),
        Effect::If {
            then, otherwise, ..
        }
        | Effect::PayOptional {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_create(then),
        Effect::May { effect, .. } => last_create(effect),
        _ => None,
    }
}

fn is_create(e: &Effect) -> bool {
    matches!(
        e,
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. }
    )
}

/// Whether the last instruction of `e` created several kinds of tokens ("a 1/1 ... token
/// and a Food token"): `vars::CREATED` then holds only the last kind, so a follow-up
/// about "them" can't be expressed with it.
fn several_kinds(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => {
            v.iter().rev().take_while(|x| is_create(x)).count() > 1
                || v.last().is_some_and(several_kinds)
        }
        Effect::If { then, .. } | Effect::PayOptional { then, .. } => several_kinds(then),
        Effect::May { effect, .. } => several_kinds(effect),
        _ => false,
    }
}

/// Whether `e` creates tokens anywhere.
fn has_create(e: &Effect) -> bool {
    match e {
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. } => true,
        Effect::Seq(v) => v.iter().any(has_create),
        Effect::If { then, .. } | Effect::PayOptional { then, .. } => has_create(then),
        Effect::May { effect, .. } => has_create(effect),
        _ => false,
    }
}

/// Appends `new` right after the instructions that created the tokens, inside the same
/// "if you do" / "you may" branch.
pub(crate) fn append_after_create(e: &mut Effect, new: Effect) -> bool {
    match e {
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. } => {
            let c = std::mem::take(e);
            *e = Effect::Seq(vec![c, new]);
            true
        }
        Effect::Seq(v) => {
            if let Some(last) = v.last_mut() {
                if has_create(last) && !is_create(last) {
                    return append_after_create(last, new);
                }
            }
            if v.iter().any(has_create) {
                v.push(new);
                true
            } else {
                false
            }
        }
        Effect::If {
            then, otherwise, ..
        }
        | Effect::PayOptional {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => append_after_create(then, new),
        Effect::May { effect, .. } => append_after_create(effect, new),
        _ => false,
    }
}

/// "It has "Sacrifice ~: Add {C}."", "They have "When ~ dies, ..."", "The token has
/// flying": more abilities in the definition of the tokens just created (CR 111.3).
fn f_token_has(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = [
        "it has ",
        "they have ",
        "the token has ",
        "the tokens have ",
        "that token has ",
        "those tokens have ",
        "each of them has ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p)) else {
        return false;
    };
    if several_kinds(prev) {
        return false;
    }
    let Some(Effect::CreateToken { spec, .. }) = last_create(prev) else {
        return false;
    };
    let Some((masked, quotes)) = super::statics::mask_quotes(r) else {
        return false;
    };
    let types = spec.card_types.clone();
    let Some(abilities) = ability_list(&masked, &quotes, &types, b.ctx) else {
        return false;
    };
    spec.abilities.extend(abilities);
    b.it = Sel::Var(vars::CREATED);
    true
}

inventory::submit! { FollowupPattern { name: "tokens_copies: the token has", priority: 40, apply: f_token_has } }

/// Whether an effect refers to the tokens just created.
fn mentions_created(e: &Effect) -> bool {
    serde_json::to_string(e)
        .is_ok_and(|s| s.contains(&format!("{{\"Var\":{}}}", vars::CREATED)))
}

/// "It gains haste until end of turn", "They gain haste", "Those tokens gain flying and
/// haste until end of turn", "attach ~ to it", "put a +1/+1 counter on it": an
/// instruction right after creating tokens whose pronoun names those tokens.
fn f_created_pronoun(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if last_create(prev).is_none() || several_kinds(prev) {
        return false;
    }
    let l = end(l);
    let rewritten = if let Some(r) = l.strip_prefix("attach ~ to ") {
        if !matches!(r, "it" | "that token" | "the token") {
            return false;
        }
        let e = Effect::Attach {
            what: Sel::This,
            to: Sel::Var(vars::CREATED),
        };
        b.it = Sel::Var(vars::CREATED);
        return append_after_create(prev, e);
    } else if let Some(r) = ["it ", "that token ", "the token "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    {
        format!("it {r}")
    } else if let Some(r) = [
        "they ",
        "those tokens ",
        "the tokens ",
        "each of those tokens ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p))
    {
        format!("them {r}")
    } else if l.starts_with("put ") && {
        // "Put X +1/+1 counters on it, where X is ...": X is defined after the pronoun.
        let body = l.split_once(", where x is ").map_or(l, |(c, _)| c);
        body.ends_with(" on it") || body.ends_with(" on each of them")
    } {
        l.replace(" on each of them", " on them")
    } else {
        return false;
    };
    let saved_it = b.it.clone();
    let e = if let Some((clause, value)) = rewritten.split_once(", where x is ") {
        // X's value keeps the referents it had ("that spell's mana value" is the
        // triggering spell's); "it" in the instruction names the created tokens.
        let value = if matches!(saved_it, Sel::None | Sel::This) {
            value.to_string()
        } else {
            super::triggers_referents::that_possessives_to_its(value)
        };
        super::r107_numbers::where_x_is_parts(clause, &value, b, Sel::Var(vars::CREATED))
    } else {
        b.it = Sel::Var(vars::CREATED);
        crate::oracle::effects::parse_simple(&rewritten, b)
    };
    let Some(e) = e else {
        b.it = saved_it;
        return false;
    };
    // Also "That token attacks this combat if able" (a restriction locked onto it).
    fn about_the_tokens(e: &Effect) -> bool {
        match e {
            Effect::Modify { .. } | Effect::AddCounters { .. } | Effect::AddRestriction { .. } => {
                true
            }
            Effect::Seq(v) => !v.is_empty() && v.iter().all(about_the_tokens),
            _ => false,
        }
    }
    if !mentions_created(&e) || !about_the_tokens(&e) {
        b.it = saved_it;
        return false;
    }
    append_after_create(prev, e)
}

inventory::submit! { FollowupPattern { name: "tokens_copies: pronoun names created tokens", priority: 40, apply: f_created_pronoun } }

/// "Exile it at the beginning of the next end step", "Sacrifice those tokens at end of
/// combat" after an earlier sentence about the tokens just created ("That token gains
/// haste."): the pronoun still names those tokens (CR 603.7).
fn f_created_delayed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(b.it, Sel::Var(v) if v == vars::CREATED) || !has_create(prev) {
        return false;
    }
    let Some((verb, r, step)) = super::damage_removal::delayed_parts(end(l)) else {
        return false;
    };
    let tail = [
        "it",
        "them",
        "that token",
        "those tokens",
        "the token",
        "the tokens",
    ]
    .iter()
    .find_map(|p| {
        r.strip_prefix(p)
            .filter(|x| x.is_empty() || x.starts_with(' '))
    });
    let Some(tail) = tail else {
        return false;
    };
    let Some(e) = super::damage_removal::delayed_removal(
        verb,
        Sel::Var(vars::CREATED),
        tail.trim(),
        step,
    ) else {
        return false;
    };
    append_after_create(prev, e)
}

inventory::submit! { FollowupPattern { name: "tokens_copies: delayed removal of created tokens", priority: 45, apply: f_created_delayed } }
