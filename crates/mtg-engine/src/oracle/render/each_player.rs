//! Instructions each player performs in turn (CR 101.4):
//!
//! * "Each player returns all creature cards from their graveyard to the battlefield",
//!   "each player may put a land card from their hand onto the battlefield": each player's
//!   choice is collected (`kw::each_player_collect`), then the collected cards move at
//!   the same time.
//! * "Each player may discard their hand and draw seven cards": each player chooses
//!   whether to take part (`scry_rules::OPT_IN`, recorded in `scry_rules::OPTED`), then
//!   those who did act. A player who can't search libraries can't choose to search
//!   (`search_rules::ITERATED_CAN_SEARCH`, CR 701.23): the choice not offered isn't in the
//!   text.

use super::players::Case;
use super::*;
use crate::kw::each_player_collect::{COLLECT, COLLECTED, PICK};

/// Whether a per-player effect is the opt-in choice.
fn is_opt_in(e: &Effect) -> bool {
    match e {
        Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect,
        } => is_opt_in(effect),
        Effect::If {
            cond: Condition::Custom(c),
            then,
            otherwise,
        } if c == crate::search_rules::ITERATED_CAN_SEARCH
            && matches!(otherwise.as_ref(), Effect::Noop) =>
        {
            is_opt_in(then)
        }
        Effect::May {
            who: PlayerRef::Iterated,
            effect,
        } => matches!(effect.as_ref(), Effect::Custom(n) if n == crate::scry_rules::OPT_IN),
        _ => false,
    }
}

/// The base form of a verb phrase in the third person ("scries 1" → "scry 1").
fn base_form(vp: &str) -> String {
    let (verb, rest) = match vp.split_once(' ') {
        Some((v, r)) => (v, format!(" {r}")),
        None => (vp, String::new()),
    };
    let v = match verb {
        "has" => "have".to_string(),
        "does" => "do".to_string(),
        "may" | "can't" | "can" => verb.to_string(),
        v if v.ends_with("ies") => format!("{}y", &v[..v.len() - 3]),
        v if ["shes", "ches", "sses", "xes"]
            .iter()
            .any(|e| v.ends_with(e)) =>
        {
            v[..v.len() - 2].to_string()
        }
        v => v.strip_suffix('s').unwrap_or(v).to_string(),
    };
    format!("{v}{rest}")
}

impl Renderer<'_> {
    /// At `v[i]` of a sequence: instructions each player performs (see the module
    /// documentation), as (how many instructions, text).
    pub(crate) fn each_player_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        // The collected choices start empty.
        if let Effect::Store {
            var: COLLECTED,
            sel: Sel::None,
        } = &v[i]
        {
            return Some((1, String::new()));
        }
        if let Some((n, vp)) = self.collect_part(v, i, None) {
            return Some((n, vp));
        }
        let Effect::ForEachPlayer { who, effect } = &v[i] else {
            return None;
        };
        if !is_opt_in(effect) {
            return None;
        }
        // Those who chose to take part, maybe under another name.
        let mut opted = vec![crate::scry_rules::OPTED];
        let mut j = i + 1;
        while let Some(Effect::Store {
            var,
            sel: Sel::Var(x),
        }) = v.get(j)
        {
            if !opted.contains(x) {
                break;
            }
            opted.push(*var);
            j += 1;
        }
        let mut vps = Vec::new();
        while j < v.len() {
            if let Effect::Store {
                var: COLLECTED,
                sel: Sel::None,
            } = &v[j]
            {
                j += 1;
                continue;
            }
            if let Some((n, vp)) = self.collect_part(v, j, Some(&opted)) {
                vps.push(vp);
                j += n;
                continue;
            }
            match &v[j] {
                Effect::ForEachPlayer {
                    who: PlayerRef::Var(x),
                    effect,
                } if opted.contains(x) => {
                    vps.push(self.as_each_player(effect));
                    j += 1;
                }
                // "Each player may search their library for a basic land card, put it onto
                // the battlefield, then shuffle": each player who searched shuffles.
                Effect::Search {
                    who: PlayerRef::Var(x),
                    whose,
                    filter,
                    count,
                    to,
                    reveal,
                    shuffle,
                } if opted.contains(x) => {
                    let mine = |p: &PlayerRef| {
                        matches!(p, PlayerRef::Iterated)
                            || matches!(p, PlayerRef::Var(y) if opted.contains(y))
                    };
                    let mut to = to.clone();
                    if to.controller.as_ref().is_some_and(mine) {
                        to.controller = None;
                    }
                    let e = Effect::Search {
                        who: PlayerRef::You,
                        whose: if mine(whose) {
                            PlayerRef::You
                        } else {
                            whose.clone()
                        },
                        filter: filter.clone(),
                        count: count.clone(),
                        to,
                        reveal: *reveal,
                        shuffle: *shuffle,
                    };
                    let s = self.effect(&e);
                    let s = match s.strip_suffix(", then shuffle") {
                        Some(head) => format!(
                            "{head}, {{alt:then shuffle|each player who searched their library this way shuffles}}"
                        ),
                        None => s,
                    };
                    vps.push(
                        format!(" {s} ")
                            .replace(" your ", " their ")
                            .trim()
                            .to_string(),
                    );
                    j += 1;
                }
                // "Each player may scry 1": the instruction is about those players.
                e if opted
                    .iter()
                    .any(|x| format!("{e:?}").contains(&format!("{:?}", PlayerRef::Var(*x)))) =>
                {
                    let s = self.effect(e);
                    let s = s.strip_prefix("that player ").unwrap_or(&s).to_string();
                    vps.push(base_form(&s).replace(" your ", " their "));
                    j += 1;
                }
                _ => break,
            }
        }
        if vps.is_empty() {
            return None;
        }
        let w = self.player(who, Case::Subj);
        // "... and draw seven cards" / "Each player who does draws seven cards."
        Some((
            j - i,
            format!("{w} may {}", vps.join(" {alt:and|each player who does} ")),
        ))
    }

    /// What one player does, worded for "each player ...": "discard their hand and draw
    /// seven cards".
    fn as_each_player(&mut self, e: &Effect) -> String {
        let inner = match e {
            Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect,
            } => self.effect(effect),
            other => {
                let saved = self.trigger_player.take();
                let s = self.effect(other);
                self.trigger_player = saved;
                s
            }
        };
        let inner = inner.strip_prefix("you ").unwrap_or(&inner);
        format!(" {inner} ")
            .replace(". you ", ". ")
            .replace(" your ", " their ")
            .trim()
            .to_string()
    }

    /// `ForEachPlayer { AsPlayer { [store the player's choice], collect } }` followed by
    /// the move of the collected cards: "each player returns all creature cards from their
    /// graveyard to the battlefield". With `opted`, only for the players who chose to (the
    /// verb phrase alone).
    fn collect_part(
        &mut self,
        v: &[Effect],
        i: usize,
        opted: Option<&[Var]>,
    ) -> Option<(usize, String)> {
        let Effect::ForEachPlayer { who, effect } = &v[i] else {
            return None;
        };
        if let Some(o) = opted {
            if !matches!(who, PlayerRef::Var(x) if o.contains(x)) {
                return None;
            }
        }
        let Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect,
        } = effect.as_ref()
        else {
            return None;
        };
        let Effect::Seq(steps) = effect.as_ref() else {
            return None;
        };
        let [Effect::Store { var: PICK, sel }, Effect::Custom(c)] = steps.as_slice() else {
            return None;
        };
        if c != COLLECT {
            return None;
        }
        let Some(Effect::Move {
            what: Sel::Var(COLLECTED),
            to,
        }) = v.get(i + 1)
        else {
            return None;
        };
        // Each card enters under its owner's control: the player who put it there.
        let mut to = to.clone();
        if matches!(&to.controller, Some(PlayerRef::OwnerOf(s)) if matches!(s.as_ref(), Sel::Var(COLLECTED)))
        {
            to.controller = None;
        }
        let vp = self.move_effect(sel, &to);
        let vp = format!(" {vp} ")
            .replace(" your ", " their ")
            .trim()
            .to_string();
        if opted.is_some() {
            return Some((2, vp));
        }
        let w = self.player(who, Case::Subj);
        Some((2, format!("{w} {}", super::effects::third_person(&vp))))
    }
}
