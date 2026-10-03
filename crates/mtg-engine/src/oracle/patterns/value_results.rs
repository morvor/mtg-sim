//! Value grammar II: amounts that refer to what happened earlier, in the same ability or
//! earlier in the turn (CR 107, 608.2c, 608.2h).
//!
//! Hooks into the value grammar (`value_grammar::count`, `value_grammar::atom_ext`).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Whether `rest` ends a word (so a prefix match is a whole phrase).
fn word_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.', ';', '"'])
}

/// After "the number of" / "for each": what's counted. Tried before the value grammar's
/// own readings.
pub fn count_ext(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = this_way_count(r, b) {
        return Some(v);
    }
    history_count(r, b)
}

/// A whole value phrase ("the amount of damage dealt to you this turn"). Tried before the
/// value grammar's own readings.
pub fn atom_ext(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = result_value(s, b) {
        return Some(v);
    }
    history_amount(s, b)
}

/// "the number of [s]" / "for each [s]" read as a whole phrase about this turn's history
/// (for readers outside instructions: cost changes, static abilities).
pub fn whole_history_count(s: &str) -> Option<Value> {
    let v = super::value_grammar::whole_count(s, Some(&Sel::This))?;
    matches!(v, Value::EventsThisTurn(..)).then_some(v)
}

fn events(cond: TriggerCond, t: Tally) -> Value {
    Value::EventsThisTurn(Box::new(cond), t)
}

/// A player the text names as the subject of a past action, as a relation: "you",
/// "your opponents", "an opponent", "they" / "that player" (the player the ability is
/// about), "target player". Returns the relation and the rest.
fn player_subject<'a>(s: &'a str, b: &Builder) -> Option<(PlayerRel, &'a str)> {
    let s = s.trim_start();
    let about = || -> Option<PlayerRel> {
        match &b.it_player {
            PlayerRef::TriggerPlayer => Some(PlayerRel::TriggerPlayer),
            PlayerRef::Target(t) => Some(PlayerRel::Target(*t)),
            PlayerRef::Iterated => Some(PlayerRel::Iterated),
            _ => None,
        }
    };
    for (p, rel) in [
        ("you", Some(PlayerRel::You)),
        ("your opponents", Some(PlayerRel::Opponent)),
        ("an opponent", Some(PlayerRel::Opponent)),
        ("opponents", Some(PlayerRel::Opponent)),
        ("they", about()),
        ("that player", about()),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if word_end(r) || r.starts_with("'ve") {
                return Some((rel?, r));
            }
        }
    }
    None
}

/// "[player] [have|has|'ve] [verb]" (or the simple past "[player] [verb]"): the player
/// and the rest after the auxiliary.
fn player_perfect<'a>(s: &'a str, b: &Builder) -> Option<(PlayerRel, &'a str)> {
    let (rel, r) = player_subject(s, b)?;
    let r = r
        .strip_prefix("'ve ")
        .or_else(|| r.strip_prefix(" have "))
        .or_else(|| r.strip_prefix(" has "))
        .or_else(|| r.strip_prefix(" "))?;
    Some((rel, r))
}

/// Strips "this turn" (and returns the rest), requiring a word end after it.
fn this_turn(s: &str) -> Option<&str> {
    let r = s.trim_start().strip_prefix("this turn")?;
    word_end(r).then_some(r)
}

/// History counts: "creatures that died under your control this turn", "nontoken
/// creatures put into your graveyard from the battlefield this turn", "creatures that
/// attacked this turn", "cards you've discarded this turn", "spells your opponents have
/// cast this turn", "permanents you've sacrificed this turn", "tokens you created this
/// turn", "opponents who lost life this turn", "2 life your opponents have lost this turn".
fn history_count(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = players_who(r, b) {
        return Some(v);
    }
    if let Some(v) = life_in_groups(r, b) {
        return Some(v);
    }
    // "cards [player] [have] drawn this turn", "cards you've discarded this turn",
    // "spells your opponents have cast this turn": the noun, then the player.
    for (noun, card) in [
        ("cards ", true),
        ("card ", true),
        ("spells ", false),
        ("spell ", false),
    ] {
        if let Some(x) = r.strip_prefix(noun) {
            if let Some(v) = player_action(x, Filter::Any, card, b) {
                return Some(v);
            }
        }
    }
    if let Some(v) = object_history(r) {
        return Some(v);
    }
    // "noncreature spells they've cast this turn".
    let (f, _plural, rest) = parse_object_phrase(r)?;
    if is_spell_filter(&f) {
        return player_action(rest, f, false, b);
    }
    None
}

fn is_spell_filter(f: &Filter) -> bool {
    match f {
        Filter::Spell => true,
        Filter::And(v) => v.iter().any(|x| matches!(x, Filter::Spell)),
        _ => false,
    }
}

/// "[player] [have] drawn / discarded / cycled or discarded / cast this turn" after
/// "cards" or "spells".
fn player_action(x: &str, f: Filter, card: bool, b: &Builder) -> Option<(Value, String)> {
    let (who, r) = player_perfect(x, b)?;
    let (cond, r) = if card {
        if let Some(r) = r.strip_prefix("drawn").or_else(|| r.strip_prefix("drew")) {
            (TriggerCond::Draws { who }, r)
        } else if let Some(r) = r
            // Cycling a card discards it (CR 702.29a): a card cycled and discarded is
            // one card.
            .strip_prefix("cycled or discarded")
            .or_else(|| r.strip_prefix("discarded"))
        {
            (TriggerCond::Discards { who, filter: f }, r)
        } else {
            return None;
        }
    } else {
        let r = r.strip_prefix("cast")?;
        let filter = if matches!(f, Filter::Any) {
            Filter::Spell
        } else {
            f
        };
        (TriggerCond::CastSpell { who, filter }, r)
    };
    let rest = this_turn(r)?;
    Some((events(cond, Tally::Events), rest.to_string()))
}

/// "[objects] that died [under your control] this turn", "... put into your graveyard
/// from the battlefield this turn", "... that attacked this turn", "... you attacked with
/// this turn", "... you've sacrificed this turn", "... sacrificed this turn", "... you
/// created this turn": the object phrase before the verb, which must read all of it.
fn object_history(r: &str) -> Option<(Value, String)> {
    let you = || Some(Filter::ControlledBy(PlayerRel::You));
    let yours = || Some(Filter::OwnedBy(PlayerRel::You));
    // CR 700.4: "dies" means is put into a graveyard from the battlefield.
    let markers: [(&str, fn(Filter) -> TriggerCond, Option<Filter>); 12] = [
        (" that died under your control", TriggerCond::Dies, you()),
        (" that died", TriggerCond::Dies, None),
        (
            " that were put into your graveyard from the battlefield",
            TriggerCond::Dies,
            yours(),
        ),
        (
            " put into your graveyard from the battlefield",
            TriggerCond::Dies,
            yours(),
        ),
        (
            " that were put into graveyards from the battlefield",
            TriggerCond::Dies,
            None,
        ),
        (
            " put into graveyards from the battlefield",
            TriggerCond::Dies,
            None,
        ),
        (
            " put into a graveyard from the battlefield",
            TriggerCond::Dies,
            None,
        ),
        (" that attacked", TriggerCond::Attacks, None),
        (" you attacked with", TriggerCond::Attacks, you()),
        (" you've sacrificed", TriggerCond::YouSacrifice, None),
        (" sacrificed", TriggerCond::Sacrificed, None),
        (" you created", TriggerCond::TokenCreated, you()),
    ];
    for (m, mk, extra) in markers {
        let Some(i) = r.find(m) else { continue };
        let Some(rest) = this_turn(&r[i + m.len()..]) else {
            continue;
        };
        let Some((f, _, tail)) = parse_object_phrase(&r[..i]) else {
            continue;
        };
        if !tail.trim().is_empty() {
            continue;
        }
        let f = match extra {
            Some(e) => Filter::and(vec![f, e]),
            None => f,
        };
        return Some((events(mk(f), Tally::Events), rest.to_string()));
    }
    None
}

/// "opponents who lost life this turn", "player who lost life this turn", "opponent who
/// was dealt damage this turn", "opponents who were dealt combat damage this turn".
fn players_who(r: &str, _b: &Builder) -> Option<(Value, String)> {
    let (who, x) = [
        ("opponents who ", PlayerRel::Opponent),
        ("opponent who ", PlayerRel::Opponent),
        ("your opponents who ", PlayerRel::Opponent),
        ("players who ", PlayerRel::Any),
        ("player who ", PlayerRel::Any),
    ]
    .iter()
    .find_map(|(p, rel)| r.strip_prefix(p).map(|x| (*rel, x)))?;
    let (cond, x) = if let Some(x) = x.strip_prefix("lost life") {
        (TriggerCond::LosesLife { who }, x)
    } else if let Some(x) = x.strip_prefix("gained life") {
        (TriggerCond::GainsLife { who }, x)
    } else if let Some(x) = x
        .strip_prefix("was dealt combat damage")
        .or_else(|| x.strip_prefix("were dealt combat damage"))
    {
        (
            TriggerCond::PlayerDealtDamage {
                who,
                combat_only: true,
            },
            x,
        )
    } else if let Some(x) = x
        .strip_prefix("was dealt damage")
        .or_else(|| x.strip_prefix("were dealt damage"))
    {
        (
            TriggerCond::PlayerDealtDamage {
                who,
                combat_only: false,
            },
            x,
        )
    } else {
        return None;
    };
    let rest = this_turn(x)?;
    Some((events(cond, Tally::Players), rest.to_string()))
}

/// "2 life your opponents have lost this turn", "1 life you gained this turn" (counted in
/// groups: "for each 2 life ...").
fn life_in_groups(r: &str, b: &Builder) -> Option<(Value, String)> {
    let (n, x) = parse_number(r)?;
    let k = n.as_const().filter(|k| *k > 0)?;
    let x = x.trim_start().strip_prefix("life ")?;
    let (who, x) = player_perfect(x, b)?;
    let (cond, x) = if let Some(x) = x.strip_prefix("lost") {
        (TriggerCond::LosesLife { who }, x)
    } else if let Some(x) = x.strip_prefix("gained") {
        (TriggerCond::GainsLife { who }, x)
    } else {
        return None;
    };
    let rest = this_turn(x)?;
    let total = events(cond, Tally::Amount);
    let v = if k == 1 {
        total
    } else {
        Value::Div(Box::new(total), k, false)
    };
    Some((v, rest.to_string()))
}

/// "the life [player] lost this turn", "the total amount of life your opponents lost this
/// turn", "the total life lost by all players this turn", "the damage dealt to your
/// opponents this turn", "the damage already dealt to that player this turn", "the total
/// amount of noncombat damage dealt to your opponents this turn".
fn history_amount(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let x = s
        .strip_prefix("the total amount of ")
        .or_else(|| s.strip_prefix("the amount of "))
        .or_else(|| s.strip_prefix("the total "))
        .or_else(|| s.strip_prefix("the "))?;
    // Life.
    if let Some(y) = x.strip_prefix("life ") {
        if let Some(z) = y.strip_prefix("lost by all players") {
            let rest = this_turn(z)?;
            return Some((
                events(
                    TriggerCond::LosesLife {
                        who: PlayerRel::Any,
                    },
                    Tally::Amount,
                ),
                rest.to_string(),
            ));
        }
        let (who, z) = player_perfect(y, b)?;
        let (cond, z) = if let Some(z) = z.strip_prefix("lost") {
            (TriggerCond::LosesLife { who }, z)
        } else if let Some(z) = z.strip_prefix("gained") {
            (TriggerCond::GainsLife { who }, z)
        } else {
            return None;
        };
        let rest = this_turn(z)?;
        return Some((events(cond, Tally::Amount), rest.to_string()));
    }
    // Damage dealt to players.
    let (combat_only, noncombat, y) = if let Some(y) = x.strip_prefix("damage ") {
        (false, false, y)
    } else if let Some(y) = x.strip_prefix("combat damage ") {
        (true, false, y)
    } else if let Some(y) = x.strip_prefix("noncombat damage ") {
        (false, true, y)
    } else {
        return None;
    };
    let y = y.strip_prefix("already ").unwrap_or(y);
    let y = y.strip_prefix("dealt to ")?;
    let (who, z) = player_subject(y, b)?;
    let z = z
        .trim_start()
        .strip_prefix("so far ")
        .unwrap_or(z.trim_start());
    let rest = this_turn(z)?;
    if noncombat {
        return Some((
            Value::Custom(format!("{NONCOMBAT_DAMAGE_TO}{}", rel_code(who)?).into()),
            rest.to_string(),
        ));
    }
    Some((
        events(
            TriggerCond::PlayerDealtDamage { who, combat_only },
            Tally::Amount,
        ),
        rest.to_string(),
    ))
}

/// `Value::Custom` prefix: noncombat damage dealt this turn to the players of a relation
/// (see `kw/value_results.rs`).
pub const NONCOMBAT_DAMAGE_TO: &str = "noncombat damage dealt this turn to:";

fn rel_code(r: PlayerRel) -> Option<&'static str> {
    Some(match r {
        PlayerRel::You => "you",
        PlayerRel::Opponent => "opponents",
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Results of earlier instructions ("this way", "the sacrificed creature")
// ---------------------------------------------------------------------------

/// The relation "they"/"their"/"that player" names, if any.
fn their_rel(b: &Builder) -> Option<PlayerRel> {
    match &b.it_player {
        PlayerRef::TriggerPlayer => Some(PlayerRel::TriggerPlayer),
        PlayerRef::Target(t) => Some(PlayerRel::Target(*t)),
        PlayerRef::Iterated => Some(PlayerRel::Iterated),
        PlayerRef::You => Some(PlayerRel::You),
        _ => None,
    }
}

/// "[objects] [verb] this way": the objects an earlier instruction of the ability acted
/// on (CR 608.2c), as the instruction left them: the cards exiled, discarded, milled,
/// returned or countered (as the new objects they became, CR 400.7), the permanents
/// destroyed, sacrificed or tapped (as they last existed on the battlefield). Qualifiers:
/// "creatures you controlled that were destroyed this way", "cards discarded this way",
/// "land cards put into their graveyard this way".
pub fn this_way_sel(r: &str, b: &Builder) -> Option<(Sel, String)> {
    use crate::discard_rules::DISCARDED;
    use crate::kw::value_results::DESTROYED;
    let i = r.find(" this way")?;
    let rest = &r[i + " this way".len()..];
    if !word_end(rest) {
        return None;
    }
    let head = &r[..i];
    let yours = Some(Filter::OwnedBy(PlayerRel::You));
    let theirs = their_rel(b).map(Filter::OwnedBy);
    let verbs: [(&str, Var, Option<Filter>); 17] = [
        (" destroyed", DESTROYED, None),
        (" sacrificed", vars::SACRIFICED, None),
        (" exiled", vars::IT, None),
        (" discarded", DISCARDED, None),
        (" milled", vars::IT, None),
        (" put into your graveyard", vars::IT, yours.clone()),
        (" put into their graveyard", vars::IT, theirs),
        (" put into a graveyard", vars::IT, None),
        (" put into graveyards", vars::IT, None),
        (" returned to your hand", vars::IT, yours),
        (" returned to its owner's hand", vars::IT, None),
        (" returned to their owner's hand", vars::IT, None),
        (" returned to their owners' hands", vars::IT, None),
        (" returned", vars::IT, None),
        (" countered", vars::IT, None),
        (" tapped", vars::TAPPED, None),
        (" drawn", vars::REVEALED, None),
    ];
    let (noun, mut var, extra) = verbs.iter().find_map(|(v, var, extra)| {
        head.strip_suffix(v).map(|n| (n, *var, extra.clone()))
    })?;
    let noun = noun
        .strip_suffix(" that were")
        .or_else(|| noun.strip_suffix(" that was"))
        .unwrap_or(noun);
    let mut parts = vec![];
    // "creatures you controlled that were destroyed this way", "artifacts they controlled
    // that were put into a graveyard this way": who controlled them as they last existed
    // on the battlefield.
    let mut noun = noun;
    for (p, rel) in [
        (" you controlled", Some(PlayerRel::You)),
        (" they controlled", their_rel(b)),
        (" that player controlled", their_rel(b)),
    ] {
        if let Some(n) = noun.strip_suffix(p) {
            parts.push(Filter::ControlledBy(rel?));
            noun = n;
            if var == vars::IT && head.contains(" put into ") {
                var = DESTROYED;
            }
            if var != DESTROYED && var != vars::SACRIFICED && var != vars::TAPPED {
                return None;
            }
        }
    }
    let (f, _, tail) = parse_object_phrase(noun)?;
    if !tail.trim().is_empty() {
        return None;
    }
    parts.insert(0, f);
    parts.extend(extra);
    let f = Filter::and(parts);
    Some((Sel::Matching(Box::new(Sel::Var(var)), f), rest.to_string()))
}

/// Whether "for each [s]" counts objects an earlier instruction acted on, read by
/// [`this_way_sel`] as a whole.
pub fn reads_this_way(s: &str, b: &Builder) -> bool {
    let s = end(s);
    let s = s
        .strip_prefix("card types among ")
        .or_else(|| s.strip_prefix("card type among "))
        .unwrap_or(s);
    this_way_sel(s, b).is_some_and(|(_, rest)| rest.trim().is_empty())
}

/// Whether a whole value phrase is about the results of earlier instructions ("the
/// greatest number of cards a player discarded this way", "the number of creatures
/// destroyed this way"), as this grammar reads it.
pub fn reads_result_value(s: &str, b: &mut Builder) -> bool {
    let s = end(s);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let v = match s.strip_prefix("the number of ") {
        Some(r) => this_way_count(r, b),
        None => result_value(s, b),
    };
    b.targets.truncate(saved.0);
    b.it = saved.1;
    b.it_player = saved.2;
    v.is_some_and(|(_, rest)| rest.trim().is_empty())
}

/// "the number of [objects] [verb] this way", "card types among cards discarded this way",
/// "the greatest number of cards a player discarded this way".
fn this_way_count(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    for p in ["card types among ", "card type among "] {
        if let Some(x) = r.strip_prefix(p) {
            let (sel, rest) = this_way_sel(x, b)?;
            return Some((Value::DistinctAmong(Among::CardTypes, Box::new(sel)), rest));
        }
    }
    let (sel, rest) = this_way_sel(r, b)?;
    Some((Value::CountSel(Box::new(sel)), rest))
}

/// An object (or objects) a result phrase names: "the sacrificed creature(s)", "the
/// discarded card(s)", "[the] [objects] [verb] this way".
fn result_ref(s: &str, b: &Builder) -> Option<(Sel, String)> {
    for (p, var) in [
        ("the sacrificed creatures", vars::SACRIFICED),
        ("the sacrificed creature", vars::SACRIFICED),
        ("the sacrificed artifacts", vars::SACRIFICED),
        ("the sacrificed artifact", vars::SACRIFICED),
        ("the sacrificed permanents", vars::SACRIFICED),
        ("the sacrificed permanent", vars::SACRIFICED),
        ("the sacrificed land", vars::SACRIFICED),
        ("the sacrificed enchantment", vars::SACRIFICED),
        ("the discarded cards", crate::discard_rules::DISCARDED),
        ("the discarded card", crate::discard_rules::DISCARDED),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) || rest.starts_with('\'') {
                return Some((Sel::Var(var), rest.to_string()));
            }
        }
    }
    let x = s.strip_prefix("the ").unwrap_or(s);
    this_way_sel(x, b)
}

/// Amounts about the results of earlier instructions: "the sacrificed creature's power",
/// "the total power of the creatures sacrificed this way", "the greatest mana value among
/// cards discarded this way", "the mana value of the permanent exiled this way", "the
/// number of red mana symbols in the sacrificed creature's mana cost", "the number of
/// card types the discarded card has", "the greatest number of cards a player discarded
/// this way".
fn result_value(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    use super::value_grammar::{of_referent, stat_word};
    // "the greatest number of cards a player discarded this way" (Windfall): the most
    // any one player discarded.
    for (p, pf) in [
        ("the greatest number of cards a player discarded this way", PlayerFilter::Any),
        ("the greatest number of cards an opponent discarded this way", PlayerFilter::Opponent),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            // The discarded cards are in their owners' graveyards (or wherever a
            // replacement effect put them): each player's own.
            let theirs = Sel::Matching(
                Box::new(Sel::Var(crate::discard_rules::DISCARDED)),
                Filter::OwnedBy(PlayerRel::Iterated),
            );
            return Some((
                Value::OverPlayers(AggOp::Max, pf, Box::new(Value::CountSel(Box::new(theirs)))),
                rest.to_string(),
            ));
        }
    }
    for (p, op) in [
        ("the total ", AggOp::Sum),
        ("the greatest ", AggOp::Max),
        ("the highest ", AggOp::Max),
        ("the least ", AggOp::Min),
        ("the lowest ", AggOp::Min),
    ] {
        if let Some(x) = s.strip_prefix(p) {
            let Some((stat, y)) = stat_word(x) else { continue };
            let link = if op == AggOp::Sum { " of " } else { " among " };
            let Some(y) = y.strip_prefix(link) else { continue };
            let Some((sel, rest)) = result_ref(y, b) else { continue };
            return Some((Value::Aggregate(op, stat, Box::new(sel)), rest));
        }
    }
    // "the power of the card returned this way", "the mana value of the sacrificed
    // artifact".
    if let Some(x) = s.strip_prefix("the ") {
        if let Some((stat, y)) = stat_word(x) {
            if let Some(y) = y.strip_prefix(" of ") {
                if let Some((sel, rest)) = result_ref(y, b) {
                    return Some((of_referent(stat, sel), rest));
                }
            }
        }
    }
    // "the number of red mana symbols in the sacrificed creature's mana cost" (CR 107.4e:
    // hybrid symbols of the color count).
    if let Some(x) = s.strip_prefix("the number of ") {
        let (w, y) = split_word(x);
        if let Some(color) = crate::types::Color::from_word(w) {
            if let Some(y) = y.strip_prefix("mana symbols in ") {
                if let Some((sel, r)) = result_ref(y, b) {
                    if let Some(rest) = r.strip_prefix("'s mana cost") {
                        return Some((
                            Value::Aggregate(AggOp::Sum, Stat::ManaSymbols(color), Box::new(sel)),
                            rest.to_string(),
                        ));
                    }
                }
            }
        }
        // "the number of card types the discarded card has".
        if let Some(y) = x.strip_prefix("card types ") {
            if let Some((sel, r)) = result_ref(y, b) {
                if let Some(rest) = r.strip_prefix(" has").filter(|r| word_end(r)) {
                    return Some((
                        Value::DistinctAmong(Among::CardTypes, Box::new(sel)),
                        rest.to_string(),
                    ));
                }
            }
        }
    }
    // "the sacrificed creature's power", "the discarded card's mana value".
    let (sel, r) = result_ref(s, b)?;
    let r = r.strip_prefix("'s ")?;
    let (stat, rest) = stat_word(r)?;
    if !word_end(rest) {
        return None;
    }
    Some((of_referent(stat, sel), rest.to_string()))
}

/// An activated ability whose cost sacrifices one permanent other than its source:
/// "that creature" ("that artifact", ...) in its effect is the sacrificed permanent (its
/// last known information, CR 608.2h), which the effect calls "the sacrificed [noun]".
pub fn cost_sacrificed_text(cost: &Cost, effect: &str) -> Option<String> {
    let sacs: Vec<&CostPart> = cost
        .parts
        .iter()
        .filter(|p| matches!(p, CostPart::Sacrifice { .. }))
        .collect();
    let [CostPart::Sacrifice {
        count: Value::Const(1),
        ..
    }] = sacs.as_slice()
    else {
        return None;
    };
    let mut text = effect.to_string();
    let mut changed = false;
    for noun in ["creature", "artifact", "permanent", "land", "enchantment"] {
        for (that, the) in [("that", "the"), ("That", "The")] {
            let from = format!("{that} {noun}");
            if text.contains(&format!("{from}'s")) {
                text = text.replace(&format!("{from}'s"), &format!("{the} sacrificed {noun}'s"));
                changed = true;
            }
        }
    }
    changed.then_some(text)
}

/// "Target player discards a card. ~ deals damage to that player equal to that card's
/// mana value.": after one player discards one card, "that card" / "it" is the discarded
/// card (the new object it became, CR 400.7j).
pub fn note_discarded(e: &Effect, b: &mut Builder) {
    let single = |who: &PlayerRef| {
        !matches!(
            who,
            PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
        )
    };
    let one = match e {
        Effect::Discard { who, n, .. } => single(who) && matches!(n, Value::Const(1)),
        Effect::AsPlayer { who, effect } => {
            single(who)
                && matches!(&**effect, Effect::Discard { who: PlayerRef::You, n: Value::Const(1), .. })
        }
        _ => false,
    };
    if one {
        b.it = Sel::Var(crate::discard_rules::DISCARDED);
    }
}

/// "Target player discards a number of cards equal to [value]", "mill a number of cards
/// equal to [value]": the same as "cards equal to [value]".
fn a_number_of_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" a number of cards equal to ")?;
    let (head, tail) = (&l[..i], &l[i + " a number of cards equal to ".len()..]);
    let verb = head.rsplit(' ').next()?;
    if !matches!(verb, "discard" | "discards" | "mill" | "mills" | "draw" | "draws") {
        return None;
    }
    crate::oracle::effects::parse_clause(&format!("{head} cards equal to {tail}"), b)
}

inventory::submit! { super::EffectPattern { name: "value results: [verb] a number of cards equal to [value]", priority: 400, parse: a_number_of_cards } }

/// "Each player discards their hand, then draws cards equal to the greatest number of
/// cards a player discarded this way." (Windfall): every player discards, and then every
/// player draws the same number of cards, determined once all the discarding is done (CR
/// 608.2c, 101.4); not each player in turn.
fn each_discards_then_draws(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = [
        ("each player discards their hand, then draws cards equal to ", PlayerRef::EachPlayer),
    ]
    .into_iter()
    .find_map(|(p, w)| l.strip_prefix(p).map(|r| (w, r)))?;
    if !reads_result_value(r, b) {
        return None;
    }
    let (v, rest) = result_value(r, b)?;
    if !rest.trim().is_empty() {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::DiscardHand { who: who.clone() },
        Effect::Draw { who, n: v },
    ]))
}

inventory::submit! { super::EffectPattern { name: "value results: each player discards their hand, then draws cards equal to [result]", priority: 5, parse: each_discards_then_draws } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Layout;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;

    /// Developer probe: `VR_PROBE=<file>` with lines `V: <value phrase>`, `E: <effect
    /// text>` or `C: <card name>`; prints what each compiles to.
    #[test]
    fn probe() {
        let Ok(path) = std::env::var("VR_PROBE") else {
            return;
        };
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let tl = TypeLine::parse("Creature — Test");
        let stl = TypeLine::parse("Sorcery");
        let ctx = CompileContext {
            card_name: "Probe",
            full_name: "Probe",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        for line in text.lines() {
            if let Some(v) = line.strip_prefix("V: ") {
                let mut b = Builder::new(&ctx);
                let r = super::super::value_grammar::parse_value(&v.to_lowercase(), &mut b);
                println!("V {v}\n  => {r:?}");
            } else if let Some(c) = line.strip_prefix("U: ") {
                let def = crate::card::card(c);
                for u in def.unsupported_text() {
                    println!("U {c}: {u}");
                }
            } else if let Some(c) = line.strip_prefix("C: ") {
                let def = crate::card::card(c);
                for f in &def.faces {
                    for a in &f.chars.abilities {
                        println!("C {c}: {:#?}", a.kind);
                    }
                }
            } else if let Some((t, tl)) = line
                .strip_prefix("T: ")
                .map(|t| (t, &stl))
                .or_else(|| line.strip_prefix("A: ").map(|t| (t, &tl)))
            {
                let c2 = CompileContext {
                    card_name: "Probe",
                    full_name: "Probe",
                    type_line: tl,
                    layout: Layout::Normal,
                    face_index: 0,
                    keywords: &[],
                    power: None,
                    toughness: None,
                };
                let comp = crate::oracle::compile(t, &c2);
                println!("T {t}\n  unsupported={:?}", comp.unsupported);
                for a in &comp.abilities {
                    println!("  {:?}", a.kind);
                }
            }
        }
    }
}
