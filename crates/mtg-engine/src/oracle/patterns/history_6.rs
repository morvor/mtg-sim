//! Turn history VII (cards6-history-permissions): the turn's history read about a target
//! or about one object of a referent.
//!
//! * Block history anchored on a target (CR 509.1, 115.1): "Destroy all creatures that
//!   were blocked by target Wall this turn." (Glyph of Reincarnation), "Put X glyph
//!   counters on target creature that target Wall blocked this turn, where X is the power
//!   of that blocked creature." (Glyph of Delusion). The anchor is a target of its own,
//!   chosen before the targets described by it ([`Filter::BlockRelation`] with
//!   `Sel::Target`), and like every target checked again on resolution (CR 608.2b). The
//!   blocks are the turn's block history (`kw/block_history.rs`): declared blocks and
//!   blocks created by effects (CR 509.1, 509.3), kept after the creatures leave combat.
//! * "For each creature that died this way, put a creature card from the graveyard of the
//!   player who controlled that creature the last time it became blocked by that Wall onto
//!   the battlefield under its owner's control." (Glyph of Reincarnation): the controller
//!   recorded with each block (`kw/block_history::controller_when_blocked_by`); after all
//!   the creatures are destroyed, a card is chosen for each creature that died (each one
//!   a card not chosen before) and then all of them are put onto the battlefield together
//!   (CR 608.2f, [`REINCARNATED`]); they may be the creatures just destroyed (Glyph of
//!   Reincarnation rulings).
//! * "[~ deals damage to that player] equal to half the damage dealt by one of those
//!   sorcery spells this turn, rounded down" (Backdraft): after "Choose a player who cast
//!   one or more sorcery spells this turn" (`choice_grammar_players.rs`), "those sorcery
//!   spells" are the spells that player cast this turn; one of them is chosen as the
//!   effect happens (`kw/damage_history_by::CHOOSE_ONE_OF`), and the amount is the damage
//!   that spell dealt this turn (the damage history with the spell as its source, CR
//!   120.1, 120.3), halved and rounded as written (CR 107.1a).

use super::{EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_clause, parse_sentence, Builder};
use crate::oracle::phrases::end;

/// Stand-in for an anchored block relation while the sentence is parsed:
/// `MARK<rel>-<when>-<slot>`.
const MARK: &str = "\u{2}h6-block-";

fn rel_code(r: BlockRel) -> char {
    match r {
        BlockRel::Blocking => 'b',
        BlockRel::BlockedBy => 'y',
        BlockRel::BlockingOrBlockedBy => 'e',
        BlockRel::BlockedOnlyBy => 'o',
    }
}

fn when_code(w: BlockWhen) -> char {
    match w {
        BlockWhen::Now => 'n',
        BlockWhen::ThisCombat => 'c',
        BlockWhen::ThisTurn => 't',
    }
}

fn mark_filter<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s.strip_prefix(MARK)?;
    let mut it = r.chars();
    let rel = match it.next()? {
        'b' => BlockRel::Blocking,
        'y' => BlockRel::BlockedBy,
        'e' => BlockRel::BlockingOrBlockedBy,
        _ => return None,
    };
    let r = it.as_str().strip_prefix('-')?;
    let mut it = r.chars();
    let when = match it.next()? {
        'c' => BlockWhen::ThisCombat,
        't' => BlockWhen::ThisTurn,
        _ => return None,
    };
    let r = it.as_str().strip_prefix('-')?;
    let n = r.find(|c: char| !c.is_ascii_digit()).unwrap_or(r.len());
    let slot: u8 = r[..n].parse().ok()?;
    Some((
        Filter::BlockRelation {
            rel,
            other: Box::new(Sel::Target(slot)),
            when,
        },
        &r[n..],
    ))
}

inventory::submit! { FilterSuffixPattern { name: "history 6: blocked by target [creature] this turn (anchored on a target)", priority: 100, parse: mark_filter } }

/// An anchored relation found in a sentence: where it is, the anchor's noun, and the
/// relation.
struct Anchor {
    start: usize,
    end: usize,
    noun: String,
    rel: BlockRel,
    when: BlockWhen,
}

/// The windows an anchored relation may end with.
const WINDOWS: &[(&str, BlockWhen)] = &[
    (" this turn", BlockWhen::ThisTurn),
    (" this combat", BlockWhen::ThisCombat),
];

/// Finds "that were blocked by target [noun] this turn", "blocked by target [noun] this
/// turn", "that target [noun] blocked this turn", "that blocked target [noun] this turn",
/// "that blocked or were blocked by target [noun] this turn" (and "this combat").
fn find_anchor(l: &str) -> Option<Anchor> {
    // "that target Wall blocked this turn": the anchor blocked the object described.
    if let Some(i) = l.find(" that target ") {
        let after = &l[i + " that target ".len()..];
        for (w, when) in WINDOWS {
            let tail = format!(" blocked{w}");
            if let Some(j) = after.find(&tail) {
                let noun = &after[..j];
                if noun.is_empty() || noun.contains(' ') && noun.split(' ').count() > 3 {
                    continue;
                }
                return Some(Anchor {
                    start: i,
                    end: i + " that target ".len() + j + tail.len(),
                    noun: noun.to_string(),
                    rel: BlockRel::BlockedBy,
                    when: *when,
                });
            }
        }
    }
    for (head, rel) in [
        (" that blocked or were blocked by target ", BlockRel::BlockingOrBlockedBy),
        (" that blocked or was blocked by target ", BlockRel::BlockingOrBlockedBy),
        (" that were blocked by target ", BlockRel::BlockedBy),
        (" that was blocked by target ", BlockRel::BlockedBy),
        (" that blocked target ", BlockRel::Blocking),
        (" blocked by target ", BlockRel::BlockedBy),
    ] {
        let Some(i) = l.find(head) else {
            continue;
        };
        // "blocking or blocked by target creature": not a relation read here.
        if l[..i].ends_with(" or") || l[..i].ends_with(" and") {
            continue;
        }
        let after = &l[i + head.len()..];
        for (w, when) in WINDOWS {
            let Some(j) = after.find(w) else {
                continue;
            };
            let noun = &after[..j];
            let rest = &after[j + w.len()..];
            if noun.is_empty()
                || noun.split(' ').count() > 3
                || rest.starts_with(|c: char| c.is_alphanumeric())
            {
                continue;
            }
            return Some(Anchor {
                start: i,
                end: i + head.len() + j + w.len(),
                noun: noun.to_string(),
                rel,
                when: *when,
            });
        }
    }
    None
}

/// A sentence with an anchored block relation: the anchor becomes a target first (so the
/// objects it describes are chosen after it, CR 601.2c), then the sentence is read with
/// the relation to that target.
fn anchored_block_history(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let a = find_anchor(&format!(" {l}"))?;
    // Offsets in the padded sentence.
    let padded = format!(" {l}");
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let (t, rest) = object_ref(&format!("target {}", a.noun), b)?;
        let Sel::Target(slot) = t else {
            return None;
        };
        if !rest.trim().is_empty() {
            return None;
        }
        let mut rewritten = format!(
            "{} {MARK}{}-{}-{slot}{}",
            &padded[..a.start],
            rel_code(a.rel),
            when_code(a.when),
            &padded[a.end..]
        );
        // "where X is the power of that blocked creature": the creature the relation
        // describes as blocked.
        if a.rel == BlockRel::BlockedBy {
            rewritten = rewritten
                .replace(" the power of that blocked creature", " its power")
                .replace(" the toughness of that blocked creature", " its toughness");
        }
        if rewritten.contains("that blocked creature") {
            return None;
        }
        // The anchor itself is "that [noun]" for later sentences only.
        b.it = saved.1.clone();
        let e = parse_sentence(rewritten.trim(), b)?;
        // A target described by the relation ("target creature that target Wall blocked
        // this turn") is announced with the words of the text, not the stand-in.
        let token = format!("{MARK}{}-{}-{slot}", rel_code(a.rel), when_code(a.when));
        let phrase = padded[a.start..a.end].trim();
        for t in b.targets.iter_mut().skip(saved.0) {
            if t.text.contains(&token) {
                t.text = t.text.replace(&token, phrase);
            }
        }
        if b.targets.iter().any(|t| t.text.contains(MARK)) {
            return None;
        }
        let json = serde_json::to_string(&e).ok()?;
        let n = json.matches("BlockRelation").count()
            + b.targets
                .iter()
                .map(|t| {
                    serde_json::to_string(&t.what)
                        .unwrap_or_default()
                        .matches("BlockRelation")
                        .count()
                })
                .sum::<usize>();
        (n == 1 && !json.contains(MARK)).then_some(e)
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    parsed
}

inventory::submit! { EffectPattern { name: "history 6: [instruction] [objects] blocked by target [creature] this turn", priority: 120, parse: anchored_block_history } }

use crate::kw::block_history::OWNED_BY_CONTROLLER_WHEN_BLOCKED;

/// The cards chosen by "for each creature that died this way, put a creature card from
/// the graveyard of the player who controlled that creature ... onto the battlefield",
/// put onto the battlefield together once all are chosen.
pub const REINCARNATED: Var = vars::USER + 61_602;

/// "For each creature that died this way, put a creature card from the graveyard of the
/// player who controlled that creature the last time it became blocked by that Wall onto
/// the battlefield under its owner's control." (Glyph of Reincarnation).
fn for_each_died_put_from_blocked_controllers_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("for each ")?;
    let (noun, r) = r.split_once(" that died this way, put ")?;
    let (card, r) = r.split_once(
        " from the graveyard of the player who controlled that creature the last time it became blocked by that ",
    )?;
    let (anchor, tail) = r.split_once(" onto the battlefield")?;
    let owners = match tail {
        " under its owner's control" | " under their owner's control" => true,
        "" => false,
        _ => return None,
    };
    // "that Wall": the target the text named "target Wall".
    let want = format!("target {anchor}");
    let slots: Vec<u8> = b
        .targets
        .iter()
        .enumerate()
        .filter(|(_, t)| t.text == want)
        .map(|(i, _)| i as u8)
        .collect();
    let [slot] = slots.as_slice() else {
        return None;
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = parse_clause(
        &format!("for each {noun} that died this way, you put {card} from your graveyard onto the battlefield"),
        b,
    );
    let ok = (|| {
        let Effect::ForEach { sel, var, effect } = e? else {
            return None;
        };
        if b.targets.len() != saved.0 {
            return None;
        }
        let Effect::Move {
            what:
                Sel::Choose {
                    chooser,
                    filter,
                    count,
                    up_to,
                    store,
                },
            mut to,
        } = *effect
        else {
            return None;
        };
        let Filter::And(parts) = filter else {
            return None;
        };
        let mut found = 0;
        let parts: Vec<Filter> = parts
            .into_iter()
            .map(|f| match f {
                Filter::OwnedBy(PlayerRel::You) => {
                    found += 1;
                    Filter::Custom(format!("{OWNED_BY_CONTROLLER_WHEN_BLOCKED}{slot}:{var}").into())
                }
                other => other,
            })
            .collect();
        if found != 1 || to.zone != ZoneKind::Battlefield || to.controller.is_some() {
            return None;
        }
        if owners {
            to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(vars::IT))));
        } else {
            // "onto the battlefield" from another player's graveyard: under the control of
            // the player who puts it there (CR 110.2a).
            to.controller = Some(PlayerRef::You);
        }
        // The cards are chosen for each creature in turn, each one a card not chosen
        // before, and then put onto the battlefield together (CR 608.2f; Glyph of
        // Reincarnation's ruling: "choose the appropriate number of creatures from the
        // attacker's graveyard and put them onto the battlefield").
        let mut parts = parts;
        parts.push(Filter::Not(Box::new(Filter::In(Box::new(Sel::Var(
            REINCARNATED,
        ))))));
        Some(Effect::seq(vec![
            Effect::ForEach {
                sel,
                var,
                effect: Box::new(Effect::Store {
                    var: REINCARNATED,
                    sel: Sel::Union(vec![
                        Sel::Var(REINCARNATED),
                        Sel::Choose {
                            chooser,
                            filter: Filter::And(parts),
                            count,
                            up_to,
                            store,
                        },
                    ]),
                }),
            },
            Effect::Move {
                what: Sel::Var(REINCARNATED),
                to,
            },
        ]))
    })();
    if ok.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    ok
}

inventory::submit! { EffectPattern { name: "history 6: for each creature that died this way, put a card from the graveyard of the player who controlled that creature when it became blocked", priority: 120, parse: for_each_died_put_from_blocked_controllers_graveyard } }

/// The spell chosen by "one of those [spells]".
const ONE_OF_THOSE: Var = vars::USER + 61_601;

/// "[instruction with an amount] equal to [half] the damage dealt by one of those
/// [spells] this turn[, rounded down]" (Backdraft).
fn damage_dealt_by_one_of_those(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (head, r) = l.split_once(" equal to ")?;
    let (half, r) = match r.strip_prefix("half ") {
        Some(x) => (true, x),
        None => (false, r),
    };
    let r = r.strip_prefix("the damage dealt by one of ")?;
    let (those, tail) = r.split_once(" this turn")?;
    let up = match tail {
        ", rounded down" if half => Some(false),
        ", rounded up" if half => Some(true),
        "" if !half => None,
        _ => return None,
    };
    let sel = b
        .named
        .iter()
        .find(|(n, _)| n == those)
        .map(|(_, s)| s.clone())?;
    if !matches!(sel, Sel::ThisTurn(_)) {
        return None;
    }
    // "~ deals damage to that player": the instruction with the amount as X.
    let head = head.strip_prefix("~ deals damage ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = parse_clause(&format!("~ deals x damage {head}"), b);
    let dealt = Value::EventsThisTurn(
        Box::new(TriggerCond::DealsDamage {
            source: Filter::In(Box::new(Sel::Var(ONE_OF_THOSE))),
            to: DamageRecipient::Any,
            combat_only: false,
        }),
        Tally::Amount,
    );
    let amount = match up {
        Some(up) => Value::Div(Box::new(dealt), 2, up),
        None => dealt,
    };
    let ok = e
        .filter(|e| b.targets.len() == saved.0 && super::r107_numbers::uses_x(e))
        .and_then(|e| super::r107_numbers::substitute_x(&e, &amount));
    let Some(e) = ok else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        return None;
    };
    let choose = crate::kw::damage_history_by::choose_one_of(ONE_OF_THOSE, &sel)?;
    Some(Effect::seq(vec![choose, e]))
}

inventory::submit! { EffectPattern { name: "history 6: [instruction] equal to half the damage dealt by one of those spells this turn", priority: 100, parse: damage_dealt_by_one_of_those } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors() {
        for (s, rel, noun) in [
            (
                " destroy all creatures that were blocked by target wall this turn",
                BlockRel::BlockedBy,
                "wall",
            ),
            (
                " destroy all creatures blocked by target wall this turn",
                BlockRel::BlockedBy,
                "wall",
            ),
            (
                " put x glyph counters on target creature that target wall blocked this turn, where x is its power",
                BlockRel::BlockedBy,
                "wall",
            ),
            (
                " tap all creatures that blocked target creature this turn",
                BlockRel::Blocking,
                "creature",
            ),
            (
                " exile each creature that blocked or were blocked by target legendary creature this turn",
                BlockRel::BlockingOrBlockedBy,
                "legendary creature",
            ),
        ] {
            let a = find_anchor(s).unwrap_or_else(|| panic!("{s}"));
            assert_eq!(a.rel, rel, "{s}");
            assert_eq!(a.noun, noun, "{s}");
        }
        assert!(find_anchor(" destroy target creature blocked this turn").is_none());
        assert!(find_anchor(
            " destroy all creatures blocking or blocked by target creature this turn"
        )
        .is_none());
    }
}
