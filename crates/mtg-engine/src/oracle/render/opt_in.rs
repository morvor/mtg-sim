//! "Each [players] may [instruction]" with what follows for those who do or don't (CR
//! 101.4, 118.12): the players choose in turn order whether to take part
//! (`scry_rules::OPT_IN`), those who did perform the instruction, and then "each player
//! who doesn't loses 3 life" (The Death of Gwen Stacy), "for each opponent who does,
//! return a creature card from your graveyard to the battlefield" (Tempt with
//! Immortality), "each player who discarded a card this way draws a card" (Mog, Moogle
//! Warrior), "~ deals 3 damage to each opponent who didn't" (Zoyowa Lava-Tongue).
//!
//! The players who did it are those who accepted, or, when the instruction records it,
//! those who actually did it (a player who accepted to discard with no cards in hand
//! didn't discard).

use super::players::Case;
use super::*;

/// What a player who took part did, as "[past] this way" and its base form: "discarded a
/// card" / "discard a card".
fn act_phrases(e: &Effect) -> Option<(String, String)> {
    let parts: Vec<(&str, &str)> = match e {
        Effect::AsPlayer { effect, .. } => return act_phrases(effect),
        Effect::Discard { .. } => vec![("discarded a card", "discard a card")],
        Effect::Draw { .. } => vec![("drew a card", "draw a card")],
        Effect::Sacrifice { .. } | Effect::SacrificeObjects { .. } => {
            vec![("sacrificed a permanent", "sacrifice a permanent")]
        }
        Effect::ChooseOne { options, .. } => {
            let v: Option<Vec<(String, String)>> =
                options.iter().map(|(_, e)| act_phrases(e)).collect();
            let v = v?;
            let past: Vec<String> = v.iter().map(|(p, _)| p.clone()).collect();
            let base: Vec<String> = v.iter().map(|(_, b)| b.clone()).collect();
            return Some((join_list(&past, "or"), join_list(&base, "or")));
        }
        _ => return None,
    };
    let (p, b) = parts.first()?;
    Some((p.to_string(), b.to_string()))
}

/// The variable a per-player step records the players who did it in: `If { [done],
/// Store { x, x ∪ that player } }`.
fn did_var(e: &Effect) -> Option<Var> {
    let Effect::If {
        then, otherwise, ..
    } = e
    else {
        return None;
    };
    if !matches!(otherwise.as_ref(), Effect::Noop) {
        return None;
    }
    let Effect::Store {
        var,
        sel: Sel::Union(v),
    } = then.as_ref()
    else {
        return None;
    };
    match v.as_slice() {
        [Sel::Var(x), Sel::Players(PlayerRef::Iterated)] if x == var => Some(*var),
        _ => None,
    }
}

/// A one-player instruction about each of several players: "that player loses 3 life"
/// for each of them is "each opponent who doesn't loses 3 life"; "~ deals 3 damage to
/// that player" is "~ deals 3 damage to each opponent who didn't".
fn about_players(e: &Effect, with: &PlayerRef) -> Option<Effect> {
    let it = |p: &PlayerRef| matches!(p, PlayerRef::Iterated);
    Some(match e {
        Effect::LoseLife { who, n } if it(who) => Effect::LoseLife {
            who: with.clone(),
            n: n.clone(),
        },
        Effect::GainLife { who, n } if it(who) => Effect::GainLife {
            who: with.clone(),
            n: n.clone(),
        },
        Effect::Draw { who, n } if it(who) => Effect::Draw {
            who: with.clone(),
            n: n.clone(),
        },
        Effect::DealDamage {
            source,
            amount,
            to: Sel::Players(p),
        } if it(p) => Effect::DealDamage {
            source: source.clone(),
            amount: amount.clone(),
            to: Sel::Players(with.clone()),
        },
        _ => return None,
    })
}

/// The instruction as performed by "that player" ("you" in it is that player): only when
/// it says "you" for no one else.
fn as_performer(e: &Effect) -> Option<Effect> {
    let mut json = serde_json::to_value(e).ok()?;
    fn walk(v: &mut serde_json::Value, you: &mut bool) {
        match v {
            serde_json::Value::String(s) if s == "You" => *you = true,
            serde_json::Value::String(s) if s == "Iterated" => *s = "You".into(),
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, you)),
            serde_json::Value::Object(m) => m.values_mut().for_each(|x| walk(x, you)),
            _ => {}
        }
    }
    let mut you = false;
    walk(&mut json, &mut you);
    if you {
        return None;
    }
    serde_json::from_value(json).ok()
}

impl Renderer<'_> {
    /// At `v[i]`: an opt-in choice and what follows for those who did or didn't take
    /// part, as (how many instructions, text).
    pub(crate) fn opt_in_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        let Effect::ForEachPlayer {
            who: offered,
            effect,
        } = &v[i]
        else {
            return None;
        };
        let effect = match effect.as_ref() {
            Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect,
            } => effect,
            e => e,
        };
        let Effect::May {
            who: PlayerRef::Iterated,
            effect: opt,
        } = effect
        else {
            return None;
        };
        if !matches!(opt.as_ref(), Effect::Custom(n) if n == crate::scry_rules::OPT_IN) {
            return None;
        }
        let Some(Effect::Store {
            var: accepted,
            sel: Sel::Var(crate::scry_rules::OPTED),
        }) = v.get(i + 1)
        else {
            return None;
        };
        let mut k = i + 2;
        // The players who did it start empty.
        let mut pre_store = None;
        if let Some(Effect::Store {
            var,
            sel: Sel::None,
        }) = v.get(k)
        {
            pre_store = Some(*var);
            k += 1;
        }
        let Some(Effect::ForEachPlayer {
            who: PlayerRef::Var(x),
            effect: step,
        }) = v.get(k)
        else {
            return None;
        };
        if x != accepted {
            return None;
        }
        let steps: Vec<&Effect> = match step.as_ref() {
            Effect::Seq(s) => s.iter().collect(),
            e => vec![e],
        };
        let (act, rest) = steps.split_first()?;
        let mut did = None;
        let mut does_followup = None;
        for e in rest {
            if let Some(x) = did_var(e) {
                did = Some(x);
                continue;
            }
            match e {
                Effect::If {
                    cond: Condition::PlayerMatches(PlayerRef::Iterated, f),
                    then,
                    otherwise,
                } if matches!(otherwise.as_ref(), Effect::Noop) && does_followup.is_none() => {
                    does_followup = Some((f, then.as_ref()));
                }
                _ => return None,
            }
        }
        if did.is_some() != pre_store.is_some() || (did.is_some() && did != pre_store) {
            return None;
        }
        let (past, base) = act_phrases(act).unwrap_or_default();
        let mut known = vec![(*accepted, past.clone(), base.clone())];
        if let Some(d) = did {
            known.push((d, past, base));
        }
        let saved = std::mem::replace(&mut self.opt_in_vars, known);
        let main = {
            let w = self.player(offered, Case::Subj);
            let a = self.as_each_player(act);
            format!("{w} may {a}")
        };
        let mut parts = vec![main];
        if let Some((f, body)) = does_followup {
            parts.push(self.opt_in_followup(f, body));
        }
        let mut n = k + 1 - i;
        if let Some(Effect::ForEachPlayer {
            who: PlayerRef::Each(PlayerFilter::And(fs)),
            effect: body,
        }) = v.get(k + 1)
        {
            let mentions = fs.iter().any(|x| {
                matches!(x, PlayerFilter::Not(r) if matches!(r.as_ref(), PlayerFilter::Ref(p)
                    if matches!(p.as_ref(), PlayerRef::Var(y) if self.opt_in_vars.iter().any(|(z, ..)| z == y))))
            });
            if mentions {
                let s = self.opt_in_followup(&PlayerFilter::And(fs.clone()), body);
                parts.push(s);
                n += 1;
            }
        }
        self.opt_in_vars = saved;
        Some((n, parts.join(". ")))
    }

    /// "Each player who discarded a card this way draws a card", "for each opponent who
    /// does, put a +1/+1 counter on each creature you control", "if a player does,
    /// creatures that player controls can't attack you ...".
    fn opt_in_followup(&mut self, f: &PlayerFilter, body: &Effect) -> String {
        let who = PlayerRef::Each(f.clone());
        if let Some(e) = about_players(body, &who) {
            return self.effect(&e);
        }
        // Performed by that player: "each player who discarded a card this way may search
        // their library ...".
        let performed = match body {
            Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect,
            } => Some((**effect).clone()),
            e => as_performer(e),
        };
        if let Some(e) = performed {
            let w = self.player(&who, Case::Subj);
            let vp = self.as_each_player(&Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(e),
            });
            return format!("{w} {}", super::effects::third_person(&vp));
        }
        let q = self.player_filter_noun(f, Num::One);
        // "If a player does, creatures that player controls ...": it's done for each
        // player who did, about that player.
        let about_them = format!("{body:?}").contains("Iterated");
        let b = self.effect(body);
        if about_them {
            format!("{{alt:for each {q},|if a player does,}} {b}")
        } else if !b.contains('.') {
            // "For each opponent who does, put ..." is "put ... for each opponent who
            // does" (one instruction, done that many times).
            format!("{b} for each {q}")
        } else {
            format!("for each {q}, {b}")
        }
    }

    /// "who does", "who discarded a card this way" (`did`), "who doesn't", "who didn't
    /// sacrifice a permanent or discard a card this way", for the players an opt-in choice
    /// recorded in `x`.
    pub(crate) fn opt_in_quality(&self, x: Var, did: bool) -> Option<String> {
        let (_, past, base) = self.opt_in_vars.iter().find(|(v, ..)| *v == x)?;
        Some(match (did, past.is_empty()) {
            (true, true) => "who {alt:does|did}".into(),
            (true, false) => format!("who {{alt:does|did|{past} this way}}"),
            (false, true) => "who {alt:doesn't|didn't}".into(),
            (false, false) => format!("who {{alt:doesn't|didn't|didn't {base} this way}}"),
        })
    }
}
