//! Token copies with exceptions (CR 707.9) and in other forms (CR 111.10, 707.2):
//!
//! ```text
//! [you] create COUNT [tapped [and attacking]] token(s) that's a copy of / that are
//!     copies of OBJECT [, except EXCEPTION ([,] and EXCEPTION)*] [and that's tapped and
//!     attacking]
//! EXCEPTION := it isn't legendary | it has ABILITIES | it's N/N | it's a N/N COLORS SUBTYPES
//!            | it's a [N/N] TYPES in addition to its other types | its name is NAME
//! ```
//!
//! The exceptions become part of the token's copiable values (CR 707.9a, 707.9b); an
//! exception that provides specific values for a characteristic also keeps the copied
//! object's characteristic-defining abilities for it from being copied (CR 707.9d, see
//! `copy::drop_overridden_cdas`).

use super::tokens_copies_create::ability_list;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;
use smol_str::SmolStr;

/// The subjects an exception clause can start with ("it", "the token", "they").
const SUBJECTS: [&str; 16] = [
    "it's ",
    "it isn't ",
    "it is ",
    "it has ",
    "its ",
    "she has ",
    "her ",
    "he has ",
    "his ",
    "they're ",
    "they aren't ",
    "they have ",
    "their ",
    "the token ",
    "the tokens ",
    "each of them ",
];

/// Splits "it isn't legendary and it has haste" into its clauses (quotes masked).
fn exception_clauses(masked: &str) -> Vec<String> {
    // "it's a 5/5 artifact creature in addition to its other types and has haste"
    // (Saheeli, Radiant Creator): the subject of the second clause is left out.
    let mut s = masked
        .replace(" other types and has ", " other types|it has ")
        .replace(" other types and have ", " other types|they have ")
        // "it isn't legendary and is a Mutant in addition to its other types" (The
        // Cloning of Shredder).
        .replace(" and is a ", "|it's a ")
        .replace(" and is an ", "|it's an ");
    for subj in SUBJECTS {
        for sep in [", and ", " and ", ", "] {
            s = s.replace(&format!("{sep}{subj}"), &format!("|{subj}"));
        }
    }
    s.split('|').map(|x| x.trim().to_string()).collect()
}

/// "black", "red and white": a color list replacing the copied colors (CR 105.3).
fn only_colors(s: &str) -> Option<ColorSet> {
    let mut colors = ColorSet::NONE;
    for w in s.split_whitespace().filter(|w| *w != "and") {
        colors.insert(Color::from_word(w.trim_end_matches(','))?);
    }
    (!colors.is_colorless()).then_some(colors)
}

/// "an artifact": a single card type replacing the copied ones (CR 205.1a). Not "an
/// artifact creature", which keeps the prior card types and subtypes (CR 205.1b).
fn only_card_types(s: &str) -> Option<Vec<CardType>> {
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    Some(vec![CardType::from_word(s)?])
}

/// "white Spirit creature" → ("Spirit creature", white): the color words taken out.
fn strip_colors(s: &str) -> (String, Option<ColorSet>) {
    let mut colors = ColorSet::NONE;
    let mut rest = Vec::new();
    let mut prev_color = false;
    for w in s.split_whitespace() {
        match Color::from_word(w) {
            Some(c) => {
                colors.insert(c);
                prev_color = true;
                continue;
            }
            // "black and green".
            None if w == "and" && prev_color => {}
            None => rest.push(w),
        }
        prev_color = false;
    }
    let colors = (!colors.is_colorless()).then_some(colors);
    (rest.join(" "), colors)
}

/// "flying", "flying and haste": abilities a copy has in addition (as keywords).
fn with_abilities(s: &str, quotes: &[String], ctx: &CompileContext) -> Option<Vec<Modification>> {
    let mut out = Vec::new();
    for a in ability_list(s, quotes, &[CardType::Creature], ctx)? {
        match &a.kind {
            AbilityKind::Keyword(k) => out.push(Modification::AddKeyword(k.clone())),
            _ => out.push(Modification::AddAbility(a)),
        }
    }
    Some(out)
}

/// "4/4" → (4, 4).
fn pt(w: &str) -> Option<(i32, i32)> {
    let (p, t) = w.split_once('/')?;
    Some((p.parse().ok()?, t.parse().ok()?))
}

/// "a 2/2 black Zombie" (in addition to its other colors and types): the colors are added
/// too (CR 105.3, 205.1b).
fn added_colors_and_types(s: &str) -> Option<Vec<Modification>> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let mut colors = ColorSet::NONE;
    let rest: Vec<&str> = s
        .split_whitespace()
        .filter(|w| match Color::from_word(w) {
            Some(c) => {
                colors.insert(c);
                false
            }
            None => true,
        })
        .collect();
    if colors.is_colorless() {
        return None;
    }
    let mut out = added_types(&rest.join(" "))?;
    out.push(Modification::AddColors(colors));
    Some(out)
}

/// "a 1/1 Fractal creature", "an artifact", "a Spirit" (in addition to its other types).
fn added_types(s: &str) -> Option<Vec<Modification>> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let mut out = Vec::new();
    let mut card_types = Vec::new();
    let mut subtypes = Vec::new();
    let mut supertypes = Vec::new();
    for (i, w) in s.split_whitespace().enumerate() {
        if i == 0 {
            if let Some((p, t)) = pt(w) {
                out.push(Modification::SetPT(Some(Value::c(p)), Some(Value::c(t))));
                continue;
            }
        }
        if let Some(st) = Supertype::from_word(w) {
            // "it's legendary in addition to its other types" (Sarkhan, Soul Aflame).
            supertypes.push(st);
        } else if let Some(t) = CardType::from_word(w) {
            card_types.push(t);
        } else {
            let sub = subtype_word(w)?;
            subtypes.push(sub);
        }
    }
    if card_types.is_empty() && subtypes.is_empty() && supertypes.is_empty() {
        return None;
    }
    if !supertypes.is_empty() {
        out.push(Modification::AddSupertypes(supertypes));
    }
    if !card_types.is_empty() {
        out.push(Modification::AddTypes(card_types));
    }
    if !subtypes.is_empty() {
        out.push(Modification::AddSubtypes(subtypes));
    }
    Some(out)
}

/// "a 4/4 black Zombie", "a 1/1 green Frog": power and toughness, colors, and creature
/// types instead of the copied ones.
fn replaced_characteristics(s: &str) -> Option<Vec<Modification>> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let mut words = s.split_whitespace();
    let (p, t) = pt(words.next()?)?;
    let mut out = vec![Modification::SetPT(Some(Value::c(p)), Some(Value::c(t)))];
    let mut colors = ColorSet::NONE;
    let mut subtypes = Vec::new();
    let mut creature = false;
    for w in words {
        if w == "and" && subtypes.is_empty() {
            continue;
        }
        // "a 6/6 green Dinosaur creature" (Dino DNA): a creature with only that
        // creature type (CR 205.1a).
        if w == "creature" && !subtypes.is_empty() && !creature {
            creature = true;
            continue;
        }
        if creature {
            return None;
        }
        if let Some(c) = Color::from_word(w) {
            if !subtypes.is_empty() {
                return None;
            }
            colors.insert(c);
        } else {
            let sub = subtype_word(w)?;
            if subtype_kind(sub.as_str()) != Some(SubtypeKind::Creature) {
                return None;
            }
            subtypes.push(sub);
        }
    }
    if !colors.is_colorless() {
        out.push(Modification::SetColors(colors));
    }
    if creature {
        out.push(Modification::SetTypes {
            types: vec![CardType::Creature],
            subtypes,
        });
    } else if !subtypes.is_empty() {
        out.push(Modification::RemoveAllCreatureTypes);
        out.push(Modification::AddSubtypes(subtypes));
    }
    Some(out)
}

/// The original-case name after "its name is " (lowercase `name`).
fn original_name(name: &str) -> Option<String> {
    let raw = crate::oracle::raw_text();
    let rl = raw.to_lowercase();
    if rl.len() != raw.len() {
        return None;
    }
    let needle = format!("name is {name}");
    let i = rl.find(&needle)? + "name is ".len();
    raw.get(i..i + name.len()).map(str::to_string)
}

/// Parses the exceptions of a copy effect ("it isn't legendary and it has haste").
pub(crate) fn copy_exceptions(
    masked: &str,
    quotes: &[String],
    ctx: &CompileContext,
) -> Option<Vec<Modification>> {
    let mut out = Vec::new();
    for c in exception_clauses(masked) {
        let c = c.as_str();
        if matches!(
            c,
            "it isn't legendary"
                | "it's not legendary"
                | "it is not legendary"
                | "they're not legendary"
                | "they aren't legendary"
                | "those creatures aren't legendary"
                | "the token isn't legendary"
                | "the token is not legendary"
                | "the tokens aren't legendary"
                | "the tokens are not legendary"
        ) {
            out.push(Modification::RemoveSupertypes(vec![Supertype::Legendary]));
        } else if let Some(r) = [
            "it has ",
            "they have ",
            "the token has ",
            "the tokens have ",
            "each of them has ",
            "she has ",
            "he has ",
        ]
            .iter()
            .find_map(|p| c.strip_prefix(p))
        {
            // "it has this ability", "it has flying and this ability" (CR 707.9a): the
            // ability creating the copy effect.
            let r = if r == "this ability" {
                out.push(Modification::AddThisAbility);
                ""
            } else if let Some(x) = r
                .strip_suffix(" and this ability")
                .or_else(|| r.strip_prefix("this ability and "))
            {
                // "it has this ability and \"[ability]\"" (Aurora Shifter).
                out.push(Modification::AddThisAbility);
                x
            } else {
                r
            };
            if r.contains("this ability") {
                return None;
            }
            if r.is_empty() {
                continue;
            }
            for a in ability_list(r, quotes, &[CardType::Creature], ctx)? {
                match &a.kind {
                    AbilityKind::Keyword(k) => out.push(Modification::AddKeyword(k.clone())),
                    _ => out.push(Modification::AddAbility(a)),
                }
            }
        } else if let Some(r) = ["it's ", "it is ", "they're ", "the token is ", "the tokens are "]
            .iter()
            .find_map(|p| c.strip_prefix(p))
        {
            if let Some(types) = r
                .strip_suffix(" in addition to its other types")
                .or_else(|| r.strip_suffix(" in addition to their other types"))
            {
                // "it's a 1/1 white Spirit creature with flying in addition to its other
                // types" (Kaya, Intangible Slayer): the abilities are added, and the
                // colors replace its colors (only types are "in addition", Anikthea's
                // ruling).
                let (types, with) = match types.split_once(" with ") {
                    Some((t, w)) => (t, Some(w)),
                    None => (types, None),
                };
                let (types, colors) = strip_colors(types);
                out.extend(added_types(&types)?);
                if let Some(c) = colors {
                    out.push(Modification::SetColors(c));
                }
                if let Some(w) = with {
                    out.extend(with_abilities(w, quotes, ctx)?);
                }
            } else if let Some(types) = r
                .strip_suffix(" in addition to its other creature types")
                .or_else(|| r.strip_suffix(" in addition to their other creature types"))
            {
                // "it's a Ninja in addition to its other creature types": creature
                // types only (CR 205.3d).
                let mods = added_types(types)?;
                if !mods.iter().all(|m| match m {
                    Modification::AddSubtypes(s) => s
                        .iter()
                        .all(|x| subtype_kind(x.as_str()) == Some(SubtypeKind::Creature)),
                    _ => false,
                }) {
                    return None;
                }
                out.extend(mods);
            } else if let Some(x) = r
                .strip_suffix(" in addition to its other colors and types")
                .or_else(|| r.strip_suffix(" in addition to their other colors and types"))
            {
                out.extend(added_colors_and_types(x)?);
            } else if let Some((p, t)) = pt(r) {
                out.push(Modification::SetPT(Some(Value::c(p)), Some(Value::c(t))));
            } else if let Some(st) = Supertype::from_word(r) {
                // "except it's legendary" (Adagia, Windswept Bastion).
                out.push(Modification::AddSupertypes(vec![st]));
            } else if let Some(types) = r
                .strip_suffix(" and loses all other card types")
                .and_then(only_card_types)
            {
                // "except it's an enchantment and loses all other card types" (Myrkul).
                out.push(Modification::SetTypes { types, subtypes: vec![] });
            } else if let Some(types) = only_card_types(r) {
                // "except it's an artifact" (Machine God's Effigy): its only card types
                // are these (CR 205.1a).
                out.push(Modification::SetTypes { types, subtypes: vec![] });
            } else if let Some(colors) = only_colors(r) {
                // "except the token is black" (Penumbra Umbra).
                out.push(Modification::SetColors(colors));
            } else {
                // "it's a 3/3 black Wraith with menace" (Sauron, the Necromancer).
                let (r, with) = match r.split_once(" with ") {
                    Some((t, w)) => (t, Some(w)),
                    None => (r, None),
                };
                out.extend(replaced_characteristics(r)?);
                if let Some(w) = with {
                    out.extend(with_abilities(w, quotes, ctx)?);
                }
            }
        } else if matches!(c, "its name is ~" | "her name is ~" | "his name is ~") {
            // The copy keeps this object's own name (Sunfrill Imitator, CR 707.9b).
            out.push(Modification::SetName(SmolStr::new(ctx.card_name)));
        } else if let Some(n) = c.strip_prefix("its name is ") {
            // "its name is Mishra's Warform" (normalized to "~'s warform"): the card's
            // short name as printed.
            let short;
            let n = if n.contains('~') {
                short = n.replace('~', &ctx.card_name.split(", ").next()?.to_lowercase());
                short.as_str()
            } else {
                n
            };
            if n.is_empty() || n.contains('~') || n.contains('"') || n.split(' ').count() > 4 {
                return None;
            }
            out.push(Modification::SetName(SmolStr::new(original_name(n)?)));
        } else {
            return None;
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Which card "a copy of the exiled card" ("copies of ...") means, judging by what the
/// ability says before it (the face's raw text; the first ability that says it).
enum ExiledBy {
    /// The ability doesn't exile anything: the cards a linked ability exiled (CR 607.2a).
    LinkedAbility,
    /// "Then you may exile a card from your hand. If you do, create a token that's a copy
    /// of the exiled card" (Nexus of Becoming): the card this ability exiled as it
    /// resolved, not also those it exiled earlier (CR 608.2c).
    ThisAbility,
    /// It exiles a card as a cost or much earlier in its text: not handled here.
    Unclear,
}

fn exiled_by() -> ExiledBy {
    let raw = crate::oracle::raw_text().to_lowercase();
    let exiles = |s: &str| {
        s.split(|c: char| !c.is_alphabetic())
            .any(|w| w == "exile" || w == "exiles")
    };
    let Some((line, i)) = raw
        .lines()
        .find_map(|l| l.find(" of the exiled card").map(|i| (l, i)))
    else {
        return ExiledBy::Unclear;
    };
    let before = &line[..i];
    if !exiles(before) {
        return ExiledBy::LinkedAbility;
    }
    let (cost, effect) = before.rsplit_once(": ").unwrap_or(("", before));
    // Only an exile in the effect, in the sentence saying it or the one before it.
    let sentences: Vec<&str> = effect.split(". ").collect();
    let (earlier, recent) = sentences.split_at(sentences.len().saturating_sub(2));
    if !exiles(cost) && !earlier.iter().any(|s| exiles(s)) && recent.iter().any(|s| exiles(s)) {
        ExiledBy::ThisAbility
    } else {
        ExiledBy::Unclear
    }
}

/// The object a token copies: "~", "it", "that creature", "enchanted creature", a target.
fn copied_object(r: &str, b: &mut Builder) -> Option<(Sel, String)> {
    if let Some(rest) = r.strip_prefix('~') {
        return Some((Sel::This, rest.to_string()));
    }
    // A phrase an earlier instruction named ("that card": the card it exiled).
    let named = b.named.iter().find_map(|(p, sel)| {
        let rest = r.strip_prefix(p.as_str())?;
        (rest.is_empty() || rest.starts_with(' ') || rest.starts_with(','))
            .then(|| (sel.clone(), rest.to_string()))
    });
    if named.is_some() {
        return named;
    }
    // "Whenever you cast a spell that targets only a single artifact or creature you
    // control, create a token that's a copy of that artifact or creature" (Vesuvan
    // Duplimancy): the spell's target.
    if matches!(b.it, Sel::TriggerSpell) {
        if let Some(rest) = r.strip_prefix("that artifact or creature") {
            return Some((
                Sel::All(Filter::TargetOf(Box::new(Sel::TriggerSpell))),
                rest.to_string(),
            ));
        }
    }
    for p in [
        "enchanted creature",
        "equipped creature",
        "enchanted artifact",
        "enchanted permanent",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some((Sel::AttachedTo, rest.to_string()));
        }
    }
    for p in [
        "it",
        "that creature",
        "that permanent",
        "that card",
        "that artifact",
        "that token",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with(',') {
                let of = crate::copy_rules::copied_referent(p, &b.it);
                return Some((of, rest.to_string()));
            }
        }
    }
    // "Whenever another nontoken Wizard you control enters, ... create a token that's a
    // copy of that Wizard" (Inalla): the object the trigger names, by its subtype.
    if let Some(r2) = r.strip_prefix("that ") {
        let (w, rest) = split_word(r2);
        if subtype_word(w).is_some_and(|s| subtype_kind(s.as_str()) == Some(SubtypeKind::Creature))
            && matches!(b.it, Sel::TriggerObject | Sel::TriggerLki)
            && (rest.is_empty() || rest.starts_with(' ') || rest.starts_with(','))
        {
            return Some((b.it.clone(), rest.to_string()));
        }
    }
    // "Sacrifice another Zombie: Create two tokens that are copies of the sacrificed
    // creature." (Cleaver Skaab): as it last existed on the battlefield (CR 608.2h).
    if let Some(rest) = r.strip_prefix("the sacrificed creature") {
        if rest.is_empty() || rest.starts_with(' ') || rest.starts_with(',') {
            return Some((Sel::Var(vars::SACRIFICED), rest.to_string()));
        }
    }
    // "the exiled card": the card(s) a linked ability of the permanent exiled (CR 607.2a,
    // 607.3: a token for each of them), or the card this ability just exiled (CR 608.2c).
    if !b.ctx.is_spell() {
        if let Some(rest) = r.strip_prefix("the exiled card") {
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with(',') {
                let of = match exiled_by() {
                    ExiledBy::LinkedAbility => {
                        super::imprint::exiled_card_ref("the exiled card")?
                    }
                    // Exile records the cards it exiled as "it".
                    ExiledBy::ThisAbility => Sel::Var(vars::IT),
                    ExiledBy::Unclear => return None,
                };
                return Some((of, rest.to_string()));
            }
        }
    }
    if let Some((spec, rest)) = parse_target(r) {
        let text = r[..r.len() - rest.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        return Some((Sel::Target(slot), rest.to_string()));
    }
    // "a nonlegendary enchantment you control": one chosen as the effect happens.
    let x = r
        .strip_prefix("a ")
        .or_else(|| r.strip_prefix("an "))
        .or_else(|| r.starts_with("another ").then_some(r))?;
    let (f, plural, rest) = parse_object_phrase(x)?;
    if plural || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    let f = Filter::and(vec![Filter::Permanent, f]);
    Some((
        Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        rest.to_string(),
    ))
}

/// "create a token that's a copy of target creature you control, except it isn't
/// legendary", "create a tapped and attacking token that's a copy of it", "create two
/// tokens that are copies of ~, except they're not legendary".
pub(crate) fn token_copy_with_exceptions(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("you create ")
        .or_else(|| l.strip_prefix("create "))?;
    let (count, r) = parse_number(r)?;
    let r = r.trim_start();
    let (mut tapped, mut attacking, r) = if let Some(x) = r.strip_prefix("tapped and attacking ") {
        (true, true, x)
    } else if let Some(x) = r.strip_prefix("tapped ") {
        (true, false, x)
    } else {
        (false, false, r)
    };
    let r = r
        .strip_prefix("token that's a copy of ")
        .or_else(|| r.strip_prefix("tokens that are copies of "))?;
    let (masked, quotes) = super::statics::mask_quotes(r)?;
    // "a copy of that creature except it's an artifact ..." (Faerie Artisans).
    let (obj_part, exc_part) = match masked
        .split_once(", except ")
        .or_else(|| masked.split_once(" except "))
    {
        Some((a, e)) => (a.to_string(), Some(e.to_string())),
        None => (masked.clone(), None),
    };
    // "... and that's tapped and attacking" (before or after the exceptions).
    let strip_attacking = |s: &str| -> Option<String> {
        [
            " and that's tapped and attacking",
            " and that are tapped and attacking",
            " that's tapped and attacking",
            " that are tapped and attacking",
        ]
        .iter()
        .find_map(|p| s.strip_suffix(p))
        .map(str::to_string)
    };
    let (obj_part, exc_part) = match (strip_attacking(&obj_part), &exc_part) {
        (Some(o), None) => {
            tapped = true;
            attacking = true;
            (o, None)
        }
        (_, Some(e)) => match strip_attacking(e) {
            Some(e2) => {
                tapped = true;
                attacking = true;
                (obj_part, Some(e2))
            }
            None => (obj_part, exc_part.clone()),
        },
        (None, None) => (obj_part, None),
    };
    if obj_part.contains('"') {
        return None;
    }
    let (of, tail) = copied_object(&obj_part, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let mods = match exc_part {
        Some(e) => copy_exceptions(&e, &quotes, b.ctx)?,
        None => {
            if !quotes.is_empty() {
                return None;
            }
            vec![]
        }
    };
    Some(Effect::CreateTokenCopy {
        of,
        count,
        controller: PlayerRef::You,
        tapped,
        attacking,
        mods,
    })
}

inventory::submit! { EffectPattern { name: "tokens_copies: token copy with exceptions", priority: 105, parse: token_copy_with_exceptions } }
