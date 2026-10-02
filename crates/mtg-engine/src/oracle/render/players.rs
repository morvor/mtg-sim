//! Selections ([`Sel`]), players ([`PlayerRef`], [`PlayerFilter`]) and targets.

use super::nouns::{possessive, Det};
use super::*;

/// A filter matching every card in a zone ("exile target player's graveyard"): the zone
/// and its owner.
fn whole_zone(f: &Filter) -> Option<(ZoneKind, Option<PlayerRel>)> {
    let atoms: Vec<&Filter> = match f {
        Filter::And(v) => v.iter().collect(),
        other => vec![other],
    };
    let mut zone = None;
    let mut owner = None;
    for a in atoms {
        match a {
            Filter::InZone(z)
                if matches!(z, ZoneKind::Graveyard | ZoneKind::Hand | ZoneKind::Library) =>
            {
                zone = Some(*z)
            }
            Filter::OwnedBy(r) => owner = Some(*r),
            Filter::Card | Filter::Any => {}
            _ => return None,
        }
    }
    zone.map(|z| (z, owner))
}

/// Grammatical role of a reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Subj,
    Obj,
    Poss,
}

fn decline(s: String, case: Case) -> String {
    match case {
        Case::Poss => possessive(&s),
        _ => s,
    }
}

impl Renderer<'_> {
    /// Whether target slot `i` is a player target.
    fn slot_is_player(&self, i: u8) -> bool {
        self.targets
            .get(i as usize)
            .is_some_and(|t| matches!(t.what, TargetKind::Player(_)))
    }

    fn slot_is_many(&self, i: u8) -> bool {
        self.targets
            .get(i as usize)
            .is_some_and(|t| t.max.as_const() != Some(1))
    }

    /// The first mention of target slot `i` ("target creature", "up to two target
    /// creatures"); later mentions are pronouns.
    pub(crate) fn target_phrase(&mut self, i: u8) -> String {
        let Some(t) = self.targets.get(i as usize).cloned() else {
            return self.gap(format!("target slot {i} out of range"));
        };
        let other = !t.distinct_from.is_empty();
        let count = match (t.min, t.max.as_const()) {
            (1, Some(1)) => None,
            (0, Some(m)) if m >= 99 => Some("any number of".into()),
            (n, Some(m)) if n as i32 == m => Some(number_word(m)),
            (0, Some(m)) if m >= 1 => Some(format!("up to {}", number_word(m))),
            (1, Some(2)) => Some("one or two".into()),
            (a, Some(m)) if a >= 1 && m > a as i32 && m - (a as i32) <= 3 => {
                let words: Vec<String> = (a as i32..=m).map(number_word).collect();
                Some(join_list(&words, "or"))
            }
            (1, Some(m)) if m > 1 && m < 100 => Some(format!("one or {}", number_word(m))),
            (0, Some(_)) | (0, None) if matches!(t.max, Value::Const(_)) => {
                Some("any number of".into())
            }
            (n, None) => {
                let v = self.value(&t.max);
                if n == 0 {
                    Some(format!("up to {v}"))
                } else {
                    Some(v)
                }
            }
            (_, Some(m)) => Some(m.to_string()),
        };
        let many = count.as_deref().is_some_and(|c| c != "up to one");
        let num = if many { Num::Many } else { Num::One };
        let core = match &t.what {
            TargetKind::Object(f) => self.noun(f, num),
            TargetKind::Spell(f) => {
                let n = self.noun(f, num);
                if n.contains("spell") || n.contains("~") {
                    n
                } else {
                    match num {
                        Num::One => format!("{n} spell"),
                        Num::Many => format!("{n} spells"),
                    }
                }
            }
            TargetKind::Player(pf) => self.player_filter_noun(pf, num),
            TargetKind::AnyTarget => {
                // "any number of targets" for divided damage up to X (CR 601.2d).
                let count = if t.divide.is_some() && t.max.as_const().is_none() {
                    Some("any number of".to_string())
                } else {
                    count
                };
                return match (count, other) {
                    (None, false) => "any target".into(),
                    (None, true) => "any other target".into(),
                    (Some(c), false) => format!("{c} targets"),
                    (Some(c), true) => format!("{c} other targets"),
                };
            }
            TargetKind::ObjectOrPlayer(f, pf) => {
                let o = self.noun(f, num);
                let p = self.player_filter_noun(pf, num);
                // "target player or planeswalker", "target creature or player".
                if o.starts_with("planeswalker") || o.starts_with("battle") {
                    format!("{p} or {o}")
                } else {
                    format!("{o} or {p}")
                }
            }
            TargetKind::Ability(f) => {
                let base = match num {
                    Num::One => "activated or triggered ability",
                    Num::Many => "activated and/or triggered abilities",
                };
                self.stack_object_noun(f, base)
            }
            TargetKind::SpellOrAbility(f) => {
                let base = match num {
                    Num::One => "spell or ability",
                    Num::Many => "spells and/or abilities",
                };
                self.stack_object_noun(f, base)
            }
        };
        // "another target creature" when the filter says "other" is in `core` already.
        let (core, other) = if let Some(rest) = core.strip_prefix("other ") {
            (rest.to_string(), true)
        } else {
            (core, other)
        };
        let mut s = match count {
            None if other => format!("another target {core}"),
            None => format!("target {core}"),
            Some(c) if other => format!("{c} other target {core}"),
            Some(c) => format!("{c} target {core}"),
        };
        if t.chosen_by_opponent {
            s.push_str(" of an opponent's choice");
        }
        s
    }

    fn stack_object_noun(&mut self, f: &Filter, base: &str) -> String {
        let flat: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            Filter::Any => vec![],
            other => vec![other],
        };
        let mut extra = Vec::new();
        for x in flat {
            match x {
                Filter::Spell | Filter::SpellOnStack | Filter::Any => {}
                Filter::ControlledBy(r) => {
                    let c = self.controls_phrase(*r, Num::One);
                    extra.push(c);
                }
                other => {
                    let s = self.noun(other, Num::One);
                    extra.push(s);
                }
            }
        }
        if extra.is_empty() {
            base.to_string()
        } else {
            format!("{base} {}", extra.join(" "))
        }
    }

    /// Mentions target slot `i`: the first time its full phrase, then a pronoun.
    pub(crate) fn target_mention(&mut self, i: u8, case: Case) -> String {
        let idx = i as usize;
        if idx >= self.targets.len() {
            return self.gap(format!("target slot {i} out of range"));
        }
        if !self.introduced[idx] {
            self.introduced[idx] = true;
            self.self_salient = false;
            let p = self.target_phrase(i);
            return decline(p, case);
        }
        if self.slot_is_player(i) {
            return match case {
                Case::Poss => "that player's".into(),
                _ => "that player".into(),
            };
        }
        if self.slot_is_many(i) {
            return match case {
                Case::Subj => "they".into(),
                Case::Obj => "them".into(),
                Case::Poss => "their".into(),
            };
        }
        match case {
            Case::Poss => "its".into(),
            _ => "it".into(),
        }
    }

    pub(crate) fn target_player_mention(&mut self, i: u8) -> String {
        self.target_mention(i, Case::Obj)
    }

    /// A selection of objects/players.
    pub(crate) fn sel(&mut self, s: &Sel, case: Case) -> String {
        let it = |case: Case| match case {
            Case::Poss => "its".to_string(),
            _ => "it".to_string(),
        };
        let them = |case: Case| match case {
            Case::Subj => "they".to_string(),
            Case::Obj => "them".to_string(),
            Case::Poss => "their".to_string(),
        };
        match s {
            Sel::None => self.gap("Sel::None"),
            Sel::This => {
                let m = self.me();
                decline(m, case)
            }
            Sel::Target(i) => self.target_mention(*i, case),
            Sel::AllTargets => {
                let any_new = self.introduced.iter().any(|b| !b);
                if any_new {
                    let n = self.targets.len() as u8;
                    let parts: Vec<String> =
                        (0..n).map(|i| self.target_mention(i, Case::Obj)).collect();
                    decline(join_list(&parts, "and"), case)
                } else {
                    them(case)
                }
            }
            Sel::Var(v) if self.var_defs.iter().any(|(x, _, used)| x == v && !used) => {
                let i = self
                    .var_defs
                    .iter()
                    .position(|(x, _, used)| x == v && !used)
                    .unwrap_or(0);
                self.var_defs[i].2 = true;
                let s = self.var_defs[i].1.clone();
                self.sel(&s, case)
            }
            Sel::Var(v) => match *v {
                vars::SACRIFICED => decline("the sacrificed creature".into(), case),
                vars::CREATED => it(case),
                _ => it(case),
            },
            Sel::TriggerObject | Sel::TriggerLki | Sel::TriggerSpell => {
                self.self_salient = false;
                it(case)
            }
            Sel::TriggerOtherObject => it(case),
            Sel::TriggerObjects => them(case),
            Sel::TriggerPlayer => decline("that player".into(), case),
            Sel::AttachedTo => {
                let n = self.attached_noun();
                decline(n, case)
            }
            Sel::AttachedToThis => decline("each permanent attached to ~".into(), case),
            Sel::All(f) => {
                if let Some(z) = whole_zone(f) {
                    let s = self.whole_zone_phrase(z.0, z.1);
                    return decline(s, case);
                }
                let s = self.noun_det(f, Det::Each);
                decline(s, case)
            }
            Sel::Players(p) => self.player(p, case),
            Sel::Choose {
                chooser,
                filter,
                count,
                up_to,
                ..
            } => {
                let det = if *up_to {
                    let n = match count {
                        Value::Const(n) => number_word(*n),
                        other => self.value(other),
                    };
                    Det::UpTo(n)
                } else {
                    self.det_for(count)
                };
                let mut s = self.noun_det(filter, det);
                if !matches!(chooser, PlayerRef::You) {
                    let c = self.player(chooser, Case::Poss);
                    s = format!("{s} of {c} choice");
                }
                decline(s, case)
            }
            Sel::Linked => decline("each card exiled with ~".into(), case),
            Sel::CreatorLinked => decline("the exiled card".into(), case),
            Sel::ExiledWithCardsNamed(n) => {
                decline(format!("a card you exiled with cards named {n}"), case)
            }
            Sel::Union(v) => {
                let mut parts: Vec<String> = v.iter().map(|x| self.sel(x, Case::Obj)).collect();
                // "each opponent and each creature and planeswalker they control".
                if parts.first().is_some_and(|p| p == "each opponent") {
                    for p in parts.iter_mut().skip(1) {
                        *p = p.replace("your opponents control", "they control");
                    }
                }
                decline(join_list(&parts, "and"), case)
            }
            Sel::TopOfGraveyard(p) => {
                let p = self.player(p, Case::Poss);
                decline(format!("the top card of {p} graveyard"), case)
            }
            Sel::TopOfLibrary(p, n) => {
                let p = self.player(p, Case::Poss);
                let s = match n {
                    Value::Const(1) => format!("the top card of {p} library"),
                    Value::Const(k) => format!("the top {} cards of {p} library", number_word(*k)),
                    other => {
                        let v = self.value(other);
                        format!("the top {v} cards of {p} library")
                    }
                };
                decline(s, case)
            }
        }
    }

    /// "target player's graveyard", "each opponent's graveyard", "all graveyards".
    fn whole_zone_phrase(&mut self, z: ZoneKind, owner: Option<PlayerRel>) -> String {
        let zw = zone_word(z);
        match owner {
            None | Some(PlayerRel::Any) => format!("all {}", plural(zw)),
            Some(PlayerRel::Opponent) => format!("each opponent's {zw}"),
            Some(r) => {
                let p = self.rel_possessive(r, Num::One);
                format!("{p} {zw}")
            }
        }
    }

    /// A player reference.
    pub(crate) fn player(&mut self, p: &PlayerRef, case: Case) -> String {
        let s = match p {
            PlayerRef::Player(_) => "that player".to_string(),
            PlayerRef::You => {
                return match case {
                    Case::Poss => "your".into(),
                    _ => "you".into(),
                }
            }
            PlayerRef::EachOpponent => "each opponent".into(),
            PlayerRef::EachPlayer => "each player".into(),
            PlayerRef::EachOtherPlayer => "each other player".into(),
            PlayerRef::Target(i) => return self.target_mention(*i, case),
            PlayerRef::ControllerOf(sel) => {
                let s = self.sel(sel, Case::Poss);
                format!("{s} controller")
            }
            PlayerRef::OwnerOf(sel) => {
                let s = self.sel(sel, Case::Poss);
                format!("{s} owner")
            }
            PlayerRef::TriggerPlayer if self.trigger_player.is_some() => {
                self.trigger_player.unwrap_or("that player").into()
            }
            PlayerRef::TriggerPlayer | PlayerRef::Iterated | PlayerRef::Var(_) => {
                "that player".into()
            }
            PlayerRef::ActivePlayer => "that player".into(),
            PlayerRef::DefendingPlayer => "defending player".into(),
            PlayerRef::ChosenPlayer(_) => "the chosen player".into(),
            PlayerRef::Each(pf) => {
                let n = self.player_filter_noun(pf, Num::One);
                format!("each {n}")
            }
            PlayerRef::Owner => "~'s owner".into(),
            PlayerRef::ChosenOpponent => "the chosen opponent".into(),
            PlayerRef::Monarch => "the monarch".into(),
        };
        decline(s, case)
    }

    /// "player", "opponent", "player with 10 or less life".
    pub(crate) fn player_filter_noun(&mut self, pf: &PlayerFilter, num: Num) -> String {
        let base = |n: &str| match num {
            Num::One => n.to_string(),
            Num::Many => plural(n),
        };
        match pf {
            PlayerFilter::Any => base("player"),
            PlayerFilter::Opponent => base("opponent"),
            PlayerFilter::NotYou => base("other player"),
            PlayerFilter::You | PlayerFilter::Controller => "you".into(),
            PlayerFilter::Is(_) => "that player".into(),
            PlayerFilter::Monarch => "the monarch".into(),
            PlayerFilter::Defending => "defending player".into(),
            PlayerFilter::Active => "the active player".into(),
            PlayerFilter::Ref(r) => self.player(r, Case::Obj),
            PlayerFilter::And(v) => {
                let mut head = base("player");
                let mut quals = Vec::new();
                for x in v {
                    match x {
                        PlayerFilter::Any => {}
                        PlayerFilter::Opponent => head = base("opponent"),
                        PlayerFilter::NotYou => head = base("other player"),
                        other => quals.push(self.player_quality(other)),
                    }
                }
                if quals.is_empty() {
                    head
                } else {
                    format!("{head} {}", join_list(&quals, "and"))
                }
            }
            PlayerFilter::Or(v) => {
                let parts: Vec<String> =
                    v.iter().map(|x| self.player_filter_noun(x, num)).collect();
                join_list(&parts, "or")
            }
            other => {
                let q = self.player_quality(other);
                format!("{} {q}", base("player"))
            }
        }
    }

    /// A player as an object ("you", "an opponent", "a player").
    pub(crate) fn player_filter_object(&mut self, pf: &PlayerFilter) -> String {
        match pf {
            PlayerFilter::You | PlayerFilter::Controller => "you".into(),
            PlayerFilter::Monarch => "the monarch".into(),
            PlayerFilter::Defending => "defending player".into(),
            PlayerFilter::Active => "the active player".into(),
            PlayerFilter::Ref(r) => self.player(r, Case::Obj),
            PlayerFilter::Is(_) => "that player".into(),
            other => {
                let n = self.player_filter_noun(other, Num::One);
                with_article(&n)
            }
        }
    }

    /// A quality of a player ("with 10 or less life", "who controls an Island").
    fn player_quality(&mut self, pf: &PlayerFilter) -> String {
        match pf {
            PlayerFilter::Any => String::new(),
            PlayerFilter::Is(_) => "that is that player".into(),
            PlayerFilter::You | PlayerFilter::Controller => "who is you".into(),
            PlayerFilter::Opponent => "who is an opponent".into(),
            PlayerFilter::NotYou => "other than you".into(),
            PlayerFilter::DealtDamageThisTurn => "who was dealt damage this turn".into(),
            PlayerFilter::Life(c, v) => {
                let v = self.value(v);
                format!("with {} life", nouns::cmp_phrase(*c, &v))
            }
            PlayerFilter::HandSize(c, v) => {
                let v = self.value(v);
                format!("with {} cards in hand", nouns::cmp_phrase(*c, &v))
            }
            PlayerFilter::GraveyardSize(c, v) => {
                let v = self.value(v);
                format!(
                    "with {} cards in their graveyard",
                    nouns::cmp_phrase(*c, &v)
                )
            }
            PlayerFilter::Monarch => "who is the monarch".into(),
            PlayerFilter::Controls(f, c, v) => {
                let n = self.count_phrase(f, *c, v);
                format!("who controls {n}")
            }
            PlayerFilter::Counters(k, c, v) => {
                let v = self.value(v);
                format!("with {} {} counters", nouns::cmp_phrase(*c, &v), k.as_str())
            }
            PlayerFilter::Defending => "who is the defending player".into(),
            PlayerFilter::Active => "whose turn it is".into(),
            PlayerFilter::Poisoned => "who is poisoned".into(),
            PlayerFilter::MaxSpeed => "with max speed".into(),
            PlayerFilter::Ref(r) => {
                let p = self.player(r, Case::Obj);
                format!("who is {p}")
            }
            PlayerFilter::And(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.player_quality(x)).collect();
                join_list(&parts, "and")
            }
            PlayerFilter::Or(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.player_quality(x)).collect();
                join_list(&parts, "or")
            }
            PlayerFilter::Not(x) => {
                let q = self.player_quality(x);
                format!("not {q}")
            }
        }
    }

    /// "three or more creatures", "no Islands", "fewer creatures than you".
    pub(crate) fn count_phrase(&mut self, f: &Filter, c: Cmp, v: &Value) -> String {
        match (c, v) {
            (Cmp::Ge, Value::Const(1)) | (Cmp::Gt, Value::Const(0)) => self.noun_det(f, Det::A),
            (Cmp::Eq, Value::Const(0))
            | (Cmp::Lt, Value::Const(1))
            | (Cmp::Le, Value::Const(0)) => {
                let n = self.noun(f, Num::Many);
                format!("no {n}")
            }
            (c, Value::Const(n)) => {
                let w = number_word(*n);
                let q = match c {
                    Cmp::Eq => format!("exactly {w}"),
                    Cmp::Ge => format!("{w} or more"),
                    Cmp::Le => format!("{w} or fewer"),
                    Cmp::Gt => format!("more than {w}"),
                    Cmp::Lt => format!("fewer than {w}"),
                    Cmp::Ne => format!("other than {w}"),
                };
                let num = if *n == 1 && matches!(c, Cmp::Eq | Cmp::Le) {
                    Num::One
                } else {
                    Num::Many
                };
                let n = self.noun(f, num);
                format!("{q} {n}")
            }
            (c, other) => {
                let v = self.value(other);
                let n = self.noun(f, Num::Many);
                match c {
                    Cmp::Gt => format!("more {n} than {v}"),
                    Cmp::Lt => format!("fewer {n} than {v}"),
                    Cmp::Ge => format!("at least as many {n} as {v}"),
                    Cmp::Le => format!("no more {n} than {v}"),
                    Cmp::Eq => format!("the same number of {n} as {v}"),
                    Cmp::Ne => format!("a different number of {n} than {v}"),
                }
            }
        }
    }
}
