//! Wordings from roundtrip-pairs-6 (docs/agents/items/roundtrip-pairs-6.md), each put into
//! words from the AST it is compiled to.

use super::nouns::{cmp_phrase, Det};
use super::players::Case;
use super::*;

impl Renderer<'_> {
    /// "Artifacts and creatures target opponent controls don't untap during that player's
    /// next untap step", "Creatures don't untap during target player's next untap step": a
    /// restriction on the permanents a targeted player controls, lasting through that
    /// player's next untap step (`turn::untap_restricted_player`).
    pub(crate) fn doesnt_untap_target_players(
        &mut self,
        f: &Filter,
        duration: &Duration,
    ) -> Option<String> {
        if !matches!(duration, Duration::ThroughNextUntapStep) {
            return None;
        }
        let Filter::And(v) = f else {
            return None;
        };
        let i = v.iter().find_map(|x| match x {
            Filter::ControlledBy(PlayerRel::Target(i)) => Some(*i),
            _ => None,
        })?;
        if !self
            .targets
            .get(i as usize)
            .is_some_and(|t| matches!(t.what, TargetKind::Player(_)))
        {
            return None;
        }
        let rest = Filter::and(
            v.iter()
                .filter(|x| !matches!(x, Filter::ControlledBy(PlayerRel::Target(_))))
                .cloned()
                .collect(),
        );
        // The second wording names the player only in the duration (each wording
        // introduces the target, so it's rendered on a copy).
        let mut probe = self.clone();
        let plain = probe.restriction_subject(&rest);
        let p = probe.player(&PlayerRef::Target(i), Case::Poss);
        let s = self.restriction_subject(f);
        Some(format!(
            "{{alt:{s} doesn't untap during {{alt:its controller's|its|that player's}} next untap step|{plain} don't untap during {p} next untap step}}"
        ))
    }

    /// "Each player exiles the top card of their library" (`Exile { what:
    /// TopOfLibrary(each player, n) }`): each player exiles the cards of their own
    /// library, as cards say, or the cards are exiled (who exiles them makes no
    /// difference, CR 701.13a).
    pub(crate) fn each_player_exiles_top(&mut self, what: &Sel, face_down: bool) -> Option<String> {
        let Sel::TopOfLibrary(p @ (PlayerRef::EachPlayer | PlayerRef::EachOpponent), n) = what
        else {
            return None;
        };
        let w = self.sel(what, Case::Obj);
        let subj = self.player(p, Case::Subj);
        let top = self.top_cards(n, "their");
        let fd = if face_down { " face down" } else { "" };
        Some(format!("{{alt:exile {w}{fd}|{subj} exiles {top}{fd}}}"))
    }

    /// Conditions worded as cards do (see each arm).
    pub(crate) fn condition_pairs6(&mut self, c: &Condition) -> Option<String> {
        match c {
            // "if you had no cards in hand at the beginning of this turn" / "if you had a
            // card in hand" (Mindstorm Crown).
            Condition::Compare(
                Value::AsTurnBegan(TurnStartStat::HandSize, who),
                cmp,
                Value::Const(k),
            ) => {
                let p = self.player(who, Case::Poss);
                let subj = self.player(who, Case::Subj);
                match (cmp, k) {
                    (Cmp::Eq | Cmp::Le, 0) => Some(format!(
                        "{{alt:there are no cards in {p} hand at the beginning of this turn|{subj} had no cards in hand {{opt:at the beginning of this turn}}}}"
                    )),
                    (Cmp::Ge, 1) | (Cmp::Gt, 0) => Some(format!(
                        "{{alt:there is a card in {p} hand at the beginning of this turn|{subj} had a card in hand {{opt:at the beginning of this turn}}}}"
                    )),
                    _ => None,
                }
            }
            // "if it was kicked twice" (Archangel of Wrath): kicked two or more times.
            Condition::Compare(
                v @ (Value::Custom(_) | Value::TimesKicked),
                Cmp::Ge,
                Value::Const(2),
            ) => {
                // Looked at on a copy: a value that can't be worded here is a gap only
                // where it's rendered.
                let s = self.clone().value(v);
                let subj = s
                    .strip_prefix("the number of times ")
                    .and_then(|r| r.strip_suffix(" was kicked"))?;
                // ("{subj} is kicked twice": as the comparison reads "if it was ...", CR
                // 608.2h.)
                Some(format!(
                    "{{alt:there are two or more times {subj} was kicked|{subj} was kicked twice|{subj} is kicked twice}}"
                ))
            }
            // "if no mana was spent to cast it" (Freestrider Commando).
            Condition::Compare(
                v @ (Value::Custom(_) | Value::ManaSpent),
                Cmp::Eq | Cmp::Le,
                Value::Const(0),
            ) => {
                let s = self.clone().value(v);
                if s.starts_with("the amount of mana spent to cast ") {
                    let s = self.value(v);
                    Some(format!(
                        "{{alt:{s} is 0|no mana was spent to cast {{alt:it|~}}}}"
                    ))
                } else {
                    None
                }
            }
            // "if target player has fewer than nine poison counters" (Vraska).
            Condition::Compare(Value::PlayerCounters(p, k), cmp, Value::Const(n)) => {
                let subj = self.player(p, Case::Subj);
                let w = number_word(*n);
                let c = match cmp {
                    Cmp::Lt => format!("fewer than {w}"),
                    Cmp::Gt => format!("more than {w}"),
                    _ => cmp_phrase(*cmp, &w),
                };
                let k = plural(&counter_name(k.as_str()));
                let have = if subj == "you" { "have" } else { "has" };
                Some(format!(
                    "{{alt:there are {c} {k} {subj} {have}|{subj} {have} {c} {k}}}"
                ))
            }
            // "if it's an instant or sorcery spell" (Galvanoth): the top card of a library
            // looked at, which cards call a spell by the spell it would be.
            Condition::SelMatches(s @ Sel::TopOfLibrary(_, Value::Const(1)), f) => {
                let subj = self.sel(s, Case::Subj);
                let n = self.noun_det(f, Det::A);
                let card = n.strip_suffix(" card")?;
                Some(format!("{subj} is {{alt:{n}|{card} spell}}"))
            }
            _ => None,
        }
    }
}

/// "{opt:each player}" before each later sentence of a per-player instruction that starts
/// with a verb ("each player chooses a creature type. {opt:each player} returns ...").
pub(crate) fn subject_again(inner: &str, subj: &str) -> String {
    const VERBS: &[&str] = &[
        "put",
        "return",
        "exile",
        "sacrifice",
        "discard",
        "draw",
        "reveal",
        "search",
        "shuffle",
        "create",
        "gain",
        "lose",
        "tap",
        "untap",
        "destroy",
        "choose",
        "remove",
        "mill",
        "scry",
        "counter",
        "add",
    ];
    // "Each player" again, or "that player" for any other subject (as the comparison
    // reads a targeted player's clauses).
    let again = if subj.starts_with("each ") && !subj.contains('{') {
        subj
    } else {
        "that player"
    };
    let mut out = String::new();
    for (i, part) in inner.split(". ").enumerate() {
        if i > 0 {
            out.push_str(". ");
            let first = part.split(' ').next().unwrap_or("");
            if VERBS.contains(&first) {
                out.push_str(&format!("{{opt:{again}}} "));
            }
        }
        out.push_str(part);
    }
    out
}

/// Whether an instruction looks at the top card of your library without taking it
/// ("you may look at the top card of your library").
pub(crate) fn looks_at_top_card(e: &Effect) -> bool {
    match e {
        Effect::May { effect, .. } => looks_at_top_card(effect),
        Effect::Dig {
            who: PlayerRef::You,
            n: Value::Const(1),
            take: Value::Const(0),
            ..
        } => true,
        _ => false,
    }
}

/// Whether a list of subtypes is every land type there is (CR 205.3i), for "is every
/// land type".
pub(crate) fn is_every_land_type(s: &[Subtype]) -> bool {
    let all = crate::types::land_types();
    s.len() == all.len()
        && all
            .iter()
            .all(|t| s.iter().any(|x| x.as_str() == t.as_str()))
}

impl Renderer<'_> {
    /// The plural of [`Renderer::card_noun`] ("cards that share a color with it").
    pub(crate) fn card_noun_many(&mut self, f: &Filter) -> String {
        if matches!(f, Filter::Any) {
            return "cards".into();
        }
        let n = self.noun(&f.clone().in_zone(ZoneKind::Hand), Num::Many);
        n.replacen(" in hands", "", 1).replacen(" in a hand", "", 1)
    }

    /// Instructions worded as a whole (see each arm).
    pub(crate) fn effect_pairs6(&mut self, e: &Effect) -> Option<String> {
        match e {
            Effect::Move { what, to } => self
                .commanders_to_hand(what, to)
                .or_else(|| self.player_puts_of_their_choice(what, to)),
            // "Attach it to another permanent it can enchant" (Aura Graft): a permanent
            // other than the one it's attached to that it can legally enchant (CR 701.3b).
            Effect::Attach {
                what: what @ Sel::Target(i),
                to:
                    Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::And(v),
                        count: Value::Const(1),
                        up_to: false,
                        store: None,
                    },
            } => {
                let other = v.iter().any(|f| matches!(f, Filter::Not(x)
                    if matches!(x.as_ref(), Filter::In(s) if matches!(s.as_ref(), Sel::HostOf(h) if matches!(h.as_ref(), Sel::Target(j) if j == i)))));
                let can = v.iter().any(|f| matches!(f, Filter::CanBeAttachedBy(s) if matches!(s.as_ref(), Sel::Target(j) if j == i)));
                let rest = v.iter().all(|f| {
                    matches!(
                        f,
                        Filter::Permanent
                            | Filter::InZone(ZoneKind::Battlefield)
                            | Filter::Not(_)
                            | Filter::CanBeAttachedBy(_)
                    )
                });
                if !(other && can && rest) {
                    return None;
                }
                let w = self.sel(what, Case::Obj);
                let verb = if self
                    .targets
                    .get(*i as usize)
                    .is_some_and(|t| format!("{:?}", t.what).contains("\"Aura\""))
                {
                    "enchant"
                } else {
                    "equip"
                };
                Some(format!("attach {w} to another permanent it can {verb}"))
            }
            _ => None,
        }
    }

    /// "Put all commanders you own from the command zone and from your graveyard into
    /// your hand" (Campfire): the generic wording or the card's.
    pub(crate) fn commanders_to_hand(&mut self, what: &Sel, to: &Destination) -> Option<String> {
        let Sel::All(Filter::And(v)) = what else {
            return None;
        };
        let [Filter::Commander, Filter::OwnedBy(PlayerRel::You), Filter::Or(zones)] = v.as_slice()
        else {
            return None;
        };
        let command = zones
            .iter()
            .any(|z| matches!(z, Filter::InZone(ZoneKind::Command)));
        let graveyard = zones.iter().any(|z| {
            matches!(z, Filter::InZone(ZoneKind::Graveyard))
                || matches!(z, Filter::And(w) if w.iter().any(|x| matches!(x, Filter::InZone(ZoneKind::Graveyard))))
        });
        if !(command && graveyard && zones.len() == 2 && to.zone == ZoneKind::Hand) {
            return None;
        }
        let g = self.move_effect(what, to);
        Some(format!(
            "{{alt:{g}|put all commanders you own from the command zone and from your graveyard into your hand}}"
        ))
    }

    /// "Target opponent puts a creature card of their choice from their graveyard onto the
    /// battlefield under your control" (Visions of Dread): the player chooses the card.
    pub(crate) fn player_puts_of_their_choice(
        &mut self,
        what: &Sel,
        to: &Destination,
    ) -> Option<String> {
        let Sel::Choose {
            chooser: PlayerRef::Target(i),
            filter: Filter::And(v),
            count: Value::Const(1),
            up_to: false,
            store: None,
        } = what
        else {
            return None;
        };
        if to.zone != ZoneKind::Battlefield
            || !to
                .controller
                .as_ref()
                .is_none_or(|c| matches!(c, PlayerRef::You))
        {
            return None;
        }
        let zone = v.iter().find_map(|f| match f {
            Filter::InZone(z @ (ZoneKind::Graveyard | ZoneKind::Hand)) => Some(*z),
            _ => None,
        })?;
        if !v
            .iter()
            .any(|f| matches!(f, Filter::OwnedBy(PlayerRel::Target(j)) if j == i))
        {
            return None;
        }
        let rest = Filter::and(
            v.iter()
                .filter(|f| !matches!(f, Filter::InZone(_) | Filter::OwnedBy(_)))
                .cloned()
                .collect(),
        );
        let mut probe = self.clone();
        let g = probe.move_effect(what, to);
        let subj = self.player(&PlayerRef::Target(*i), Case::Subj);
        let card = with_article(&self.card_noun(&rest));
        let ctrl = if to.controller.is_some() {
            " under your control"
        } else {
            ""
        };
        let zone = zone_word(zone);
        Some(format!(
            "{{alt:{g}|{subj} puts {card} of their choice from their {zone} onto the battlefield{ctrl}}}"
        ))
    }

    /// Pairs of instructions said as one: (the wording, how many instructions).
    pub(crate) fn pairs6_seq(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        use crate::kw::choice_grammar as cg;
        match v {
            // "~ deals 10 damage to an opponent chosen at random" (Zaffai): the player is
            // chosen as the damage is dealt (CR 608.2d).
            [Effect::Custom(name), Effect::DealDamage {
                source,
                amount,
                to: Sel::Players(PlayerRef::Var(p)),
            }, ..]
                if name.starts_with(cg::RANDOM_PLAYER) =>
            {
                let j = name.strip_prefix(cg::RANDOM_PLAYER)?;
                let (var, from): (Var, PlayerRef) = serde_json::from_str(j).ok()?;
                if var != *p || !matches!(from, PlayerRef::EachOpponent) {
                    return None;
                }
                let src = self.sel(source, Case::Subj);
                let (a, w) = self.amount(amount);
                let w = w.unwrap_or_default();
                Some((
                    format!("{{alt:choose an opponent at random. {src} deals {a} damage to that player|{src} deals {a} damage to an opponent chosen at random}}{w}"),
                    2,
                ))
            }
            // "You may cast target instant or sorcery card ... When you cast that spell, ~
            // gets +X/+0" (Ogre Battlecaster): the reflexive trigger of casting it.
            [cast, rest @ ..]
                if (matches!(cast, Effect::CastCard { .. })
                    || matches!(cast, Effect::WithPlayTerms { effect, .. } if matches!(effect.as_ref(), Effect::CastCard { .. })))
                    && matches!(
                        rest,
                        [Effect::Reflexive { .. }, ..]
                            | [Effect::AddReplacement { .. }, Effect::Reflexive { .. }, ..]
                    ) =>
            {
                let n = if matches!(rest, [Effect::Reflexive { .. }, ..]) {
                    2
                } else {
                    3
                };
                let mut parts = vec![self.effect(cast)];
                if n == 3 {
                    parts.push(self.effect(&v[1]));
                }
                let r = self.effect(&v[n - 1]);
                let r = r.strip_prefix("when you do, ")?;
                parts.push(format!("{{alt:when you do|when you cast that spell}}, {r}"));
                Some((parts.join(". "), n))
            }
            // "You may cast ... from among them ... Then put all cards exiled this way that
            // weren't cast into your graveyard" (Epic Experiment): the rest of the group.
            [cast @ Effect::CastCard {
                what:
                    Sel::Choose {
                        filter: Filter::And(fs),
                        ..
                    },
                ..
            }, Effect::Move {
                what: Sel::All(Filter::And(parts)),
                to,
            }, ..]
                if to.zone == ZoneKind::Graveyard
                    && matches!(parts.as_slice(), [Filter::In(g), Filter::InZone(ZoneKind::Exile)]
                        if matches!(g.as_ref(), Sel::Var(x) if fs.iter().any(|f| matches!(f, Filter::In(s) if matches!(s.as_ref(), Sel::Var(y) if y == x))))) =>
            {
                if to.controller.is_some() {
                    return None;
                }
                let c = self.effect(cast);
                Some((
                    format!("{c}. {{alt:put the rest into their owners' graveyards|put all cards exiled this way that weren't cast into your graveyard}}"),
                    2,
                ))
            }
            _ => None,
        }
    }

    /// "Counter all abilities your opponents control" (Kadena's Silencer): activated and
    /// triggered abilities are the abilities on the stack (CR 113.3b, 113.3c).
    pub(crate) fn counter_all_abilities(&mut self, what: &Sel, w: &str) -> Option<String> {
        let Sel::All(Filter::And(v)) = what else {
            return None;
        };
        let kinds = v.iter().any(|f| {
            matches!(f, Filter::Or(k) if k.len() == 2 && k.iter().all(|x| matches!(x, Filter::Custom(n)
                if n.as_str() == crate::game_terms::ACTIVATED_ABILITY || n.as_str() == crate::game_terms::TRIGGERED_ABILITY)))
        });
        let ctrl = v.iter().find_map(|f| match f {
            Filter::ControlledBy(PlayerRel::Opponent) => Some("your opponents control"),
            Filter::ControlledBy(PlayerRel::You) => Some("you control"),
            _ => None,
        })?;
        if !kinds || v.len() > 3 {
            return None;
        }
        Some(format!("{{alt:counter {w}|counter all abilities {ctrl}}}"))
    }

    /// "Whenever you discard one or more cards, exile them from your graveyard" (Toluz):
    /// the triggering cards that are in that zone.
    pub(crate) fn exile_them_from_zone(&mut self, what: &Sel, face_down: bool) -> Option<String> {
        if face_down {
            return None;
        }
        let Sel::Matching(inner, Filter::And(v)) = what else {
            return None;
        };
        if !matches!(inner.as_ref(), Sel::TriggerObjects) {
            return None;
        }
        let [Filter::InZone(z), Filter::OwnedBy(PlayerRel::You)] = v.as_slice() else {
            return None;
        };
        let zone = zone_word(*z);
        let w = self.sel(what, Case::Obj);
        Some(format!("{{alt:exile {w}|exile them from your {zone}}}"))
    }

    /// "Exile all cards from target player's hand and graveyard" (Identity Crisis).
    pub(crate) fn exile_hand_and_graveyard(
        &mut self,
        what: &Sel,
        face_down: bool,
    ) -> Option<String> {
        if face_down {
            return None;
        }
        let Sel::All(Filter::And(v)) = what else {
            return None;
        };
        let [Filter::Card, Filter::Or(zones), Filter::OwnedBy(PlayerRel::Target(i))] = v.as_slice()
        else {
            return None;
        };
        if !matches!(
            zones.as_slice(),
            [
                Filter::InZone(ZoneKind::Hand),
                Filter::InZone(ZoneKind::Graveyard)
            ]
        ) {
            return None;
        }
        // Each wording introduces the target, so the first is rendered on a copy.
        let mut probe = self.clone();
        let w = probe.sel(what, Case::Obj);
        let p = self.player(&PlayerRef::Target(*i), Case::Poss);
        Some(format!(
            "{{alt:exile {w}|exile all cards from {p} hand and graveyard}}"
        ))
    }

    /// "Look at the top three cards of your library. Put one of them into your hand and
    /// the rest on the bottom of your library in any order. If ..., put each of those cards
    /// into your hand instead." (Accumulate Wisdom, See the Truth): the same look, all of
    /// the cards taken.
    pub(crate) fn dig_all_instead(
        &mut self,
        cond: &Condition,
        then: &Effect,
        otherwise: &Effect,
    ) -> Option<String> {
        let (
            Effect::Dig {
                who: w1,
                n: n1,
                reveal: r1,
                filter: Filter::Any,
                take: t1,
                take_up_to: false,
                take_to: d1,
                rest_to: e1,
            },
            Effect::Dig {
                who: w2,
                n: n2,
                reveal: r2,
                filter: Filter::Any,
                take: Value::Const(1),
                take_up_to: false,
                take_to: d2,
                rest_to: e2,
            },
        ) = (then, otherwise)
        else {
            return None;
        };
        if format!("{w1:?}") != format!("{w2:?}")
            || format!("{n1:?}") != format!("{n2:?}")
            || r1 != r2
            || format!("{t1:?}") != format!("{n1:?}")
            || format!("{d1:?}") != format!("{d2:?}")
            || format!("{e1:?}") != format!("{e2:?}")
            || d1.zone != ZoneKind::Hand
        {
            return None;
        }
        let o = self.effect(otherwise);
        let c = self.condition(cond);
        Some(format!(
            "{o}. If {c}, put {{alt:each of those cards|all of them|them}} into your hand instead"
        ))
    }
}
