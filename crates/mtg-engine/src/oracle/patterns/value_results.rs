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
    history_count(r, b)
}

/// A whole value phrase ("the amount of damage dealt to you this turn"). Tried before the
/// value grammar's own readings.
pub fn atom_ext(s: &str, b: &mut Builder) -> Option<(Value, String)> {
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
