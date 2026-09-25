//! Oracle patterns for ending the game (CR 104): "[players] win(s)/lose(s) the game",
//! "the game is a draw", "[players] can't win/lose the game", and "When you remove the
//! last [kind] counter from ~, ...".

use super::{AbilityPattern, EffectPattern, StaticPattern};
use crate::ability::*;
use crate::types::CounterKind;
use crate::oracle::effects::{parse_body, Builder};
use crate::oracle::phrases::{end, parse_player};
use crate::oracle::CompileContext;

inventory::submit! { EffectPattern { name: "players win/lose the game", priority: 100, parse: win_or_lose } }
inventory::submit! { EffectPattern { name: "the game is a draw", priority: 100, parse: game_draw } }
inventory::submit! { EffectPattern { name: "players can't win/lose the game this turn", priority: 100, parse: cant_win_lose_this_turn } }
inventory::submit! { StaticPattern { name: "players can't win/lose the game", priority: 100, parse: cant_win_lose_static } }
inventory::submit! { AbilityPattern { name: "when you remove the last counter", priority: 100, parse: last_counter_removed } }

/// "target player loses the game", "each player wins the game", "you win the game"
/// (CR 104.2b, 104.3e).
fn win_or_lose(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = format!("{} ", end(l));
    let (who, spec, rest) = parse_player(&l)?;
    let win = match rest.trim() {
        "wins the game" | "win the game" => true,
        "loses the game" | "lose the game" => false,
        _ => return None,
    };
    let who = match spec {
        Some(spec) => {
            let text = spec.text.clone();
            PlayerRef::Target(b.add_target(spec, &text))
        }
        None => who,
    };
    Some(if win {
        Effect::WinGame { who }
    } else {
        Effect::LoseGame { who }
    })
}

/// "the game is a draw" (CR 104.4c).
fn game_draw(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "the game is a draw")
        .then(|| Effect::Custom(crate::game_end::DRAW_EFFECT.into()))
}

/// The players a "can't win/lose" clause is about.
fn subject(s: &str) -> Option<(PlayerFilter, &str)> {
    let pairs: [(&str, PlayerFilter); 5] = [
        ("you ", PlayerFilter::You),
        ("your opponents ", PlayerFilter::Opponent),
        ("each opponent ", PlayerFilter::Opponent),
        ("players ", PlayerFilter::Any),
        ("each player ", PlayerFilter::Any),
    ];
    pairs
        .into_iter()
        .find_map(|(p, f)| s.strip_prefix(p).map(|r| (f, r)))
}

/// "X can't lose the game[ or win the game]" / "X can't win the game": the restrictions.
fn cant_clause(s: &str) -> Option<Vec<Restriction>> {
    let (who, rest) = subject(s.trim())?;
    let rest = rest.strip_prefix("can't ")?;
    let mut out = Vec::new();
    for part in rest.split(" or ") {
        match part.trim() {
            "lose the game" => out.push(Restriction::CantLoseGame(who.clone())),
            "win the game" => out.push(Restriction::CantWinGame(who.clone())),
            _ => return None,
        }
    }
    Some(out)
}

/// Clauses joined by " and ".
fn cant_clauses(s: &str) -> Option<Vec<Restriction>> {
    let mut out = Vec::new();
    for c in s.split(" and ") {
        out.extend(cant_clause(c)?);
    }
    (!out.is_empty()).then_some(out)
}

/// "You can't win the game and your opponents can't lose the game." (CR 101.2).
fn cant_win_lose_static(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let rs = cant_clauses(end(l))?;
    Some(
        rs.into_iter()
            .map(|r| {
                AbilityDef::new(
                    AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
                    text,
                )
            })
            .collect(),
    )
}

/// "you can't lose the game this turn and your opponents can't win the game this turn",
/// "players can't lose the game or win the game this turn" (CR 101.2).
fn cant_win_lose_this_turn(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.ends_with(" this turn") {
        return None;
    }
    let l = l.replace(" this turn", "");
    let rs = cant_clauses(&l)?;
    Some(Effect::seq(
        rs.into_iter()
            .map(|restriction| Effect::AddRestriction {
                restriction,
                duration: Duration::ThisTurn,
            })
            .collect(),
    ))
}

/// "When you remove the last [kind] counter from ~, [effect]." — triggers when counters of
/// that kind are removed from it and none are left.
fn last_counter_removed(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let mut rest = None;
    for this in ["~", "this permanent", "this enchantment", "this artifact", "this creature"] {
        for verb in ["when you remove the last ", "when the last "] {
            let Some(r) = lower.strip_prefix(verb) else {
                continue;
            };
            let Some((kind, r)) = r.split_once(" counter ") else {
                continue;
            };
            let tail = if verb == "when the last " {
                format!("is removed from {this}, ")
            } else {
                format!("from {this}, ")
            };
            if let Some(r) = r.strip_prefix(&tail) {
                rest = Some((kind.to_string(), r.to_string()));
            }
        }
    }
    let (kind, effect) = rest?;
    if kind.contains(' ') {
        return None;
    }
    let body = parse_body(&effect, ctx)?;
    let kind: CounterKind = kind.as_str().into();
    let mut ta = TriggeredAbility::new(
        TriggerCond::CountersRemoved {
            filter: Filter::Source,
            kind: Some(kind.clone()),
        },
        body,
    );
    ta.intervening_if = Some(Condition::Compare(
        Value::CountersOn(Box::new(Sel::This), Some(kind)),
        Cmp::Eq,
        Value::Const(0),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Triggered(ta), t)])
}
