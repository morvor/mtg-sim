//! Instructions written together because a later one uses what an earlier one chose:
//! "The next time a red source of your choice would deal damage to you this turn, prevent
//! that damage." (a chosen source, then a prevention shield for it, CR 615.7).

use super::*;

impl Renderer<'_> {
    /// At `v[i]` of a sequence: such instructions, how many they are and their text.
    pub(crate) fn tail_seq_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        self.chosen_source_prevention(v, i).map(|s| (2, s))
    }

    /// "Choose a [quality] source. Prevent the damage it would deal": "the next time a
    /// [quality] source of your choice would deal damage to you this turn, prevent that
    /// damage" (one use, CR 615.7), "prevent all damage a [quality] source of your choice
    /// would deal this turn".
    fn chosen_source_prevention(&mut self, v: &[Effect], i: usize) -> Option<String> {
        let (
            Effect::ChooseSource {
                who: PlayerRef::You,
                filter,
                var,
            },
            Some(Effect::AddReplacement {
                def,
                duration,
                uses,
            }),
        ) = (&v[i], v.get(i + 1))
        else {
            return None;
        };
        let ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only,
        } = &def.event
        else {
            return None;
        };
        if !matches!(def.action, ReplacementAction::Prevent) || def.optional {
            return None;
        }
        // The damage's source is the chosen one (and has the quality it was chosen with).
        let chosen = format!("{:?}", Filter::In(Box::new(Sel::Var(*var))));
        let parts: Vec<&Filter> = match source {
            Filter::And(v) => v.iter().collect(),
            f => vec![f],
        };
        if !parts.iter().any(|f| format!("{f:?}") == chosen) {
            return None;
        }
        let rest: Vec<Filter> = parts
            .into_iter()
            .filter(|f| format!("{f:?}") != chosen)
            .cloned()
            .collect();
        let quality = Filter::and(rest);
        if format!("{quality:?}") != format!("{filter:?}")
            && !(rest_is_any(&quality) && matches!(filter, Filter::Any))
        {
            return None;
        }
        let to = match (to_players, to_objects) {
            (Some(PlayerFilter::You), None) => " to you",
            (Some(PlayerFilter::Any), Some(Filter::Any)) => "",
            _ => return None,
        };
        let src = match filter {
            Filter::Any => "a source of your choice".to_string(),
            f => {
                let n = self.noun(f, Num::One);
                let adj = n.strip_suffix("permanent")?.trim_end();
                format!("{} source of your choice", with_article(adj))
            }
        };
        let when = match duration {
            Duration::EndOfTurn | Duration::ThisTurn => " this turn",
            _ => return None,
        };
        let damage = if *combat_only {
            "combat damage"
        } else {
            "damage"
        };
        Some(match uses {
            Some(1) => {
                format!("the next time {src} would deal {damage}{to}{when}, prevent that damage")
            }
            None => format!("prevent all {damage} {src} would deal{to}{when}"),
            Some(_) => return None,
        })
    }
}

fn rest_is_any(f: &Filter) -> bool {
    match f {
        Filter::Any => true,
        Filter::And(v) => v.is_empty(),
        _ => false,
    }
}

impl Renderer<'_> {
    /// What an earlier instruction did to the objects `s` holds, and whether they're
    /// cards (not permanents): "destroyed", "discarded".
    pub(crate) fn this_way_of(&self, s: &Sel) -> Option<(&'static str, bool, Option<CardType>)> {
        let Sel::Var(v) = s else {
            return None;
        };
        let (_, verb, t) = self.this_way.iter().rev().find(|(x, _, _)| x == v)?;
        let card = matches!(*verb, "discarded" | "milled" | "exiled" | "drawn");
        Some((verb, card, *t))
    }
}

/// The variables `prev` (earlier instructions of a sequence, in order) leave holding the
/// objects one of them acted on, and what it did: a destroy instruction leaves the
/// destroyed permanents in [`vars::IT`] (not those that were regenerated or
/// indestructible), a discard instruction the discarded cards in
/// [`crate::discard_rules::DISCARDED`], a draw instruction the drawn cards in
/// [`vars::REVEALED`].
pub(crate) fn this_way(prev: &[Effect]) -> Vec<(Var, &'static str, Option<CardType>)> {
    let mut out: Vec<(Var, &'static str, Option<CardType>)> = Vec::new();
    for e in prev {
        note_this_way(e, &mut out);
    }
    out
}

fn note_this_way(e: &Effect, out: &mut Vec<(Var, &'static str, Option<CardType>)>) {
    let set = |out: &mut Vec<(Var, &'static str, Option<CardType>)>,
               v: Var,
               verb: Option<(&'static str, Option<CardType>)>| {
        out.retain(|(x, _, _)| *x != v);
        if let Some((verb, t)) = verb {
            out.push((v, verb, t));
        }
    };
    // The one card type the objects acted on all have ("destroy all creatures": the
    // creatures destroyed this way).
    let ty = |s: &Sel| -> Option<CardType> {
        let Sel::All(f) = s else { return None };
        let parts: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            f => vec![f],
        };
        let types: Vec<CardType> = parts
            .iter()
            .filter_map(|f| match f {
                Filter::Type(t) => Some(*t),
                _ => None,
            })
            .collect();
        match types.as_slice() {
            [t] => Some(*t),
            _ => None,
        }
    };
    match e {
        Effect::Destroy { what, .. } => set(out, vars::IT, Some(("destroyed", ty(what)))),
        Effect::Exile { what, .. } => set(out, vars::IT, Some(("exiled", ty(what)))),
        Effect::Sacrifice { .. } | Effect::SacrificeObjects { .. } => {
            set(out, vars::IT, Some(("sacrificed", None)))
        }
        Effect::Move { to, what } if to.zone == ZoneKind::Hand => {
            set(out, vars::IT, Some(("returned", ty(what))))
        }
        Effect::Tap { what } => set(out, vars::TAPPED, Some(("tapped", ty(what)))),
        Effect::Discard { .. } | Effect::DiscardHand { .. } => set(
            out,
            crate::discard_rules::DISCARDED,
            Some(("discarded", None)),
        ),
        Effect::Draw { .. } => set(out, vars::REVEALED, Some(("drawn", None))),
        // "You may discard your hand. If you do, draw that many cards.": what the
        // optional instruction did, if it was done.
        Effect::May { effect, .. } => note_this_way(effect, out),
        Effect::Store {
            var,
            sel: Sel::Var(src),
        } => {
            let verb = out
                .iter()
                .find(|(x, _, _)| x == src)
                .map(|(_, v, t)| (*v, *t));
            set(out, *var, verb);
        }
        Effect::Store { var, .. } => set(out, *var, None),
        // Another instruction acting on objects leaves something else in "it".
        Effect::Move { .. } | Effect::Mill { .. } => set(out, vars::IT, None),
        _ => {}
    }
}

/// Instructions a player chooses among as the effect happens (CR 608.2d), written as one
/// instruction with the alternatives joined by "or": "add {B}{B} or {G}{G}", "~ gets
/// +1/-1 or -1/+1 until end of turn", "~ gains your choice of vigilance, lifelink, or haste
/// until end of turn". The words all of them start and end with are written once.
pub(crate) fn or_form(options: &[String]) -> Option<String> {
    if options.len() < 2
        || options.iter().any(|o| {
            o.is_empty() || o.contains(['\n', '|']) || o.trim_end_matches('.').contains(". ")
        })
    {
        return None;
    }
    let words: Vec<Vec<&str>> = options
        .iter()
        .map(|o| o.trim_end_matches('.').split(' ').collect())
        .collect();
    let same = |a: &str, b: &str| {
        let selfish = |w: &str| matches!(w, "~" | "~it" | "it");
        a == b || (selfish(a) && selfish(b))
    };
    let min = words.iter().map(Vec::len).min()?;
    let mut pre = 0;
    while pre < min && words.iter().all(|w| same(w[pre], words[0][pre])) {
        pre += 1;
    }
    let mut suf = 0;
    while suf < min - pre
        && words
            .iter()
            .all(|w| same(w[w.len() - 1 - suf], words[0][words[0].len() - 1 - suf]))
    {
        suf += 1;
    }
    let middles: Vec<String> = words
        .iter()
        .map(|w| w[pre..w.len() - suf].join(" "))
        .collect();
    if middles.iter().any(String::is_empty) {
        return None;
    }
    let head = words[0][..pre].join(" ");
    let tail = words[0][words[0].len() - suf..].join(" ");
    // "gains your choice of flying or vigilance": a choice among objects of a verb.
    let choice = if pre > 0 { "{opt:your choice of} " } else { "" };
    let list = super::join_list(&middles, "or");
    Some(
        [head, format!("{choice}{list}"), tail]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    )
}
