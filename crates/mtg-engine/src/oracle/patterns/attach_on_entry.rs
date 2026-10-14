//! Cards put onto the battlefield attached to something (CR 303.4f-g, 301.5c), and
//! descriptions of what could be attached: "an Aura card that could enchant that
//! creature" (Sovereigns of Lost Alara, Auratouched Mage) — a card that could legally be
//! attached to the object as it currently exists (its enchant ability, protection; the
//! object isn't targeted, so shroud and hexproof don't matter).

use super::{EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_clause, Builder};
use crate::oracle::phrases::end;

/// "that could enchant it", "that could enchant that creature".
fn could_enchant<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("that could enchant ")?;
    let (sel, rest) = super::filters_relational::object_in(r)?;
    Some((Filter::CouldAttachTo(Box::new(sel)), rest))
}

inventory::submit! { FilterSuffixPattern { name: "attach: that could enchant [object]", priority: 100, parse: could_enchant } }

/// "[return it to the battlefield], then attach it to that creature" (Sword of the Meek,
/// Ratonhnhaké:ton): the permanent the first instruction put onto the battlefield is
/// attached to the object (CR 701.3a; nothing happens if it can't be, CR 701.3b).
fn then_attach_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first, to) = l.rsplit_once(", then attach it to ")?;
    if !(first.contains(" to the battlefield") || first.contains(" onto the battlefield")) {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let parsed = (|| {
        let e = parse_clause(first, b)?;
        // "that creature" / "that token": what the text named before ("it" is the
        // permanent returned).
        let it_before = b.it.clone();
        b.it = saved.1.clone();
        let (to, rest) = object_ref(to, b)?;
        b.it = it_before;
        if !rest.trim().is_empty() {
            return None;
        }
        Some(Effect::seq(vec![
            e,
            Effect::Attach {
                what: Sel::Var(vars::IT),
                to,
            },
        ]))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    parsed
}

inventory::submit! { EffectPattern { name: "attach: [put onto the battlefield], then attach it to [object]", priority: 100, parse: then_attach_it } }

/// The player chosen by "under an opponent's control".
const CHOSEN_OPPONENT: Var = vars::USER + 2340;

/// Who controls or what is attached to a permanent put onto the battlefield, said after
/// "to the battlefield".
enum EntryQualifier {
    Controller(PlayerRef),
    AttachedTo(Sel),
}

/// Sets the qualifier on every battlefield destination in `e` (as JSON: a destination
/// is the object with `zone` and `attached_to`); returns how many it set.
fn qualify(v: &mut serde_json::Value, q: &serde_json::Value, key: &str) -> usize {
    match v {
        serde_json::Value::Object(m) => {
            let is_dest = m.contains_key("attached_to")
                && m.contains_key("with_counters")
                && m.get("zone").and_then(|z| z.as_str()) == Some("Battlefield");
            if is_dest {
                // The text says who controls it (a default the instruction without the
                // qualifier read is replaced), or what it's attached to.
                if key == "attached_to" && !m.get(key).is_none_or(|x| x.is_null()) {
                    return usize::MAX / 2;
                }
                m.insert(key.to_string(), q.clone());
                return 1;
            }
            m.values_mut().map(|x| qualify(x, q, key)).sum()
        }
        serde_json::Value::Array(a) => a.iter_mut().map(|x| qualify(x, q, key)).sum(),
        _ => 0,
    }
}

/// "Return ~ to the battlefield under that player's control at the beginning of their
/// next upkeep." (Plague Reaver), "return it to the battlefield attached to you at the
/// beginning of the next end step" (Lynde, Cheerful Tormentor), "return it to the
/// battlefield under an opponent's control" (Sol'Kanar the Tainted): the instruction
/// without the qualifier, with it set on where the permanent goes (CR 110.2a, 303.4f).
fn battlefield_qualifier(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let phrases = [
        " under that player's control",
        " under target opponent's control",
        " under target player's control",
        " under an opponent's control",
        " attached to you",
    ];
    let (at, phrase) = phrases
        .iter()
        .find_map(|p| l.find(p).map(|i| (i, *p)))?;
    let before = &l[..at];
    if !(before.ends_with(" the battlefield") || before.ends_with(" the battlefield tapped")) {
        return None;
    }
    if l.matches(phrase).count() != 1 {
        return None;
    }
    let stripped = format!("{before}{}", &l[at + phrase.len()..]);
    let saved = (b.targets.len(), b.it_player.clone());
    let parsed = (|| {
        let mut pre = None;
        let q = match phrase {
            " under that player's control" => match &b.it_player {
                PlayerRef::Target(_) | PlayerRef::TriggerPlayer => {
                    EntryQualifier::Controller(b.it_player.clone())
                }
                // "Choose target opponent. Return ~ ... under that player's control."
                _ => {
                    let players: Vec<usize> = b
                        .targets
                        .iter()
                        .enumerate()
                        .filter(|(_, t)| matches!(t.what, TargetKind::Player(_)))
                        .map(|(i, _)| i)
                        .collect();
                    match players[..] {
                        [i] => EntryQualifier::Controller(PlayerRef::Target(i as u8)),
                        _ => return None,
                    }
                }
            },
            " under target opponent's control" | " under target player's control" => {
                let (pf, text) = if phrase.contains("opponent") {
                    (PlayerFilter::Opponent, "target opponent")
                } else {
                    (PlayerFilter::Any, "target player")
                };
                let slot = b.add_target(TargetSpec::player(pf, text), text);
                EntryQualifier::Controller(PlayerRef::Target(slot))
            }
            " under an opponent's control" => {
                pre = Some(crate::kw::choice_grammar::choose_player_effect(
                    CHOSEN_OPPONENT,
                    PlayerFilter::Opponent,
                )?);
                EntryQualifier::Controller(PlayerRef::Var(CHOSEN_OPPONENT))
            }
            _ => EntryQualifier::AttachedTo(Sel::Players(PlayerRef::You)),
        };
        let e = parse_clause(&stripped, b)?;
        let mut json = serde_json::to_value(&e).ok()?;
        let n = match &q {
            EntryQualifier::Controller(p) => {
                qualify(&mut json, &serde_json::to_value(p).ok()?, "controller")
            }
            EntryQualifier::AttachedTo(s) => {
                qualify(&mut json, &serde_json::to_value(s).ok()?, "attached_to")
            }
        };
        if n != 1 {
            return None;
        }
        let e: Effect = serde_json::from_value(json).ok()?;
        Some(match pre {
            Some(p) => Effect::seq(vec![p, e]),
            None => e,
        })
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
    }
    parsed
}

inventory::submit! { EffectPattern { name: "attach/control: to the battlefield under [player]'s control / attached to you", priority: 100, parse: battlefield_qualifier } }
