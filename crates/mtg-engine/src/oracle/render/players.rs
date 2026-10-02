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
        // Alternative wordings each take the possessive.
        Case::Poss if s.starts_with("{alt:") && s.ends_with('}') => {
            let inner = &s["{alt:".len()..s.len() - 1];
            let alts: Vec<String> = inner.split('|').map(possessive).collect();
            format!("{{alt:{}}}", alts.join("|"))
        }
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
        let count = match (t.fixed_min(), t.max.as_const()) {
            // "X target creatures": exactly that many (the minimum is the maximum).
            (None, _) if format!("{:?}", t.min) == format!("{:?}", t.max) => {
                Some(self.value(&t.max))
            }
            (None, _) => Some(self.gap("a target minimum other than a number or the maximum")),
            (Some(1), Some(1)) => None,
            (Some(0), Some(m)) if m >= 99 => Some("any number of".into()),
            (Some(n), Some(m)) if n == m => Some(number_word(m)),
            (Some(0), Some(m)) if m >= 1 => Some(format!("up to {}", number_word(m))),
            (Some(1), Some(2)) => Some("one or two".into()),
            (Some(a), Some(m)) if a >= 1 && m > a && m - a <= 3 => {
                let words: Vec<String> = (a..=m).map(number_word).collect();
                Some(join_list(&words, "or"))
            }
            (Some(1), Some(m)) if m > 1 && m < 100 => Some(format!("one or {}", number_word(m))),
            (Some(0), Some(_)) | (Some(0), None) if matches!(t.max, Value::Const(_)) => {
                Some("any number of".into())
            }
            (Some(n), None) => {
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
            // "any other target": other than the object dealing the damage.
            TargetKind::ObjectOrPlayer(Filter::And(v), PlayerFilter::Any)
                if count.is_none()
                    && matches!(v.as_slice(), [Filter::Or(kinds), other]
                        if kinds.len() == 3
                            && kinds.iter().all(|k| matches!(k,
                                Filter::Type(CardType::Creature | CardType::Planeswalker | CardType::Battle)))
                            && (matches!(other, Filter::Other)
                                || matches!(other, Filter::Not(x)
                                    if matches!(x.as_ref(), Filter::In(_) | Filter::AttachedToSource)))) =>
            {
                return "any other target".into();
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
        // After an earlier target object, "other" means other than that object; a filter
        // also excluding the source says something the text can't, and is left as is
        // ("other target other creature").
        let other_than_object = t.distinct_from.iter().any(|j| {
            self.targets.get(*j as usize).is_some_and(|e| {
                matches!(
                    e.what,
                    TargetKind::Object(_) | TargetKind::ObjectOrPlayer(..) | TargetKind::AnyTarget
                )
            })
        });
        let (core, other) = match core.strip_prefix("other ") {
            Some(rest) if !(other && other_than_object) => (rest.to_string(), true),
            _ => (core, other),
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
        // A requirement on the targets taken together (CR 115.3).
        let one = matches!(t.max, Value::Const(1));
        match &t.together {
            None => {}
            _ if one => {}
            Some(TargetGroup::SameOwner) => {
                let before = s.clone();
                for g in [" in a graveyard", " in graveyards"] {
                    s = s.replacen(g, " from a single graveyard", 1);
                }
                // "two target cards from an opponent's graveyard": one graveyard.
                if s == before && !s.contains("graveyard") {
                    s.push_str(" a single player owns");
                }
            }
            Some(TargetGroup::SameController) => {
                s.push_str(" {alt:a single player controls|controlled by the same player}")
            }
            Some(TargetGroup::ShareCreatureType) => s.push_str(" that share a creature type"),
            Some(TargetGroup::ShareCardType) => s.push_str(" that share a card type"),
            Some(TargetGroup::SharePermanentType) => s.push_str(" that share a permanent type"),
            Some(g) => {
                let w = self.target_group(g);
                s.push_str(&format!(" {w}"));
            }
        }
        // A relationship with the targets of an earlier instance of "target" ("another
        // target creature with the same controller", "that shares a card type with it").
        match &t.related_to {
            None => {}
            Some((_, TargetGroup::SameController)) => s.push_str(" with the same controller"),
            Some((_, TargetGroup::ShareCardType)) => s.push_str(" that shares a card type with it"),
            Some((_, TargetGroup::ShareCardTypeAmong(_))) => {
                s.push_str(" that shares one of those types with it")
            }
            Some((_, g)) => {
                let w = self.gap(format!("a relationship with another target: {g:?}"));
                s.push_str(&format!(" {w}"));
            }
        }
        s
    }

    /// A requirement on objects chosen together ("with different names", "with total
    /// mana value 6 or less").
    pub(crate) fn target_group(&mut self, g: &TargetGroup) -> String {
        match g {
            TargetGroup::SameOwner => "a single player owns".into(),
            TargetGroup::SameController => {
                "{alt:a single player controls|controlled by the same player}".into()
            }
            TargetGroup::ShareCreatureType => "that share a creature type".into(),
            TargetGroup::ShareCardType => "that share a card type".into(),
            TargetGroup::SharePermanentType => "that share a permanent type".into(),
            TargetGroup::ShareNoCreatureType => "that share no creature types".into(),
            TargetGroup::DifferentNames => "with different names".into(),
            TargetGroup::DifferentControllers => "with different controllers".into(),
            TargetGroup::DifferentManaValues => "with different mana values".into(),
            TargetGroup::DifferentPowers => "with different powers".into(),
            TargetGroup::EqualToughness => "with equal toughness".into(),
            TargetGroup::ShareCardTypeAmong(types) => {
                let words: Vec<String> = types
                    .iter()
                    .map(|t| format!("{t:?}").to_lowercase())
                    .collect();
                format!("that share {}", join_list(&words, "or"))
            }
            TargetGroup::OnePerCardType => "one of each card type".into(),
            TargetGroup::TotalAtMost(stat, v) => {
                let st = match stat {
                    TotalStat::ManaValue => "mana value",
                    TotalStat::Power => "power",
                    TotalStat::Toughness => "toughness",
                    TotalStat::PowerAndToughness => "power and toughness",
                };
                let v = self.value(v);
                format!("with total {st} {v} or less")
            }
        }
    }

    fn stack_object_noun(&mut self, f: &Filter, base: &str) -> String {
        let flat: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            Filter::Any => vec![],
            other => vec![other],
        };
        let mut extra = Vec::new();
        // "target activated ability", "target triggered ability you control".
        let mut base = base.to_string();
        for x in flat.iter() {
            if let Filter::Custom(n) = x {
                match n.as_str() {
                    "stack:activated ability" => base = "activated ability".into(),
                    "stack:triggered ability" => base = "triggered ability".into(),
                    _ => {}
                }
            }
        }
        let flat: Vec<&Filter> = flat
            .into_iter()
            .filter(|x| {
                !matches!(x, Filter::Custom(n)
                    if n == "stack:activated ability" || n == "stack:triggered ability")
            })
            .collect();
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
            base
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
                // "Target player shuffles their graveyard into their library."
                Case::Poss => "{alt:that player's|their}".into(),
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
            Sel::Var(v) if self.plural_vars.contains(v) => them(case),
            Sel::Var(v) => match *v {
                vars::SACRIFICED => {
                    let n = self.sacrificed.clone().unwrap_or_else(|| "creature".into());
                    decline(format!("the sacrificed {n}"), case)
                }
                vars::CREATED => it(case),
                _ => it(case),
            },
            Sel::TriggerObject
            | Sel::TriggerLki
            | Sel::TriggerSpell
            | Sel::TriggerOtherObject
            | Sel::TriggerObjects
            | Sel::TriggerPlayer
                if !self.event_scope =>
            {
                self.gap("the triggering object outside a triggered ability")
            }
            // Another object while "it" is the object itself (named again since the
            // trigger condition): cards say "that creature" (`that-object`, which matches
            // only that wording, see `compare::token_eq`).
            Sel::TriggerObject | Sel::TriggerLki | Sel::TriggerSpell
                if self.self_salient && !self.other_salient && !self.trigger_is_self =>
            {
                decline("that-object".into(), case)
            }
            Sel::TriggerObject | Sel::TriggerLki | Sel::TriggerSpell => {
                self.self_salient = false;
                self.other_salient = true;
                self.self_named_in_clause = false;
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
            // "the permanent target Aura is attached to".
            Sel::HostOf(s) => {
                let s = self.sel(s, Case::Subj);
                decline(format!("the permanent {s} is attached to"), case)
            }
            // "Reveal it and put it into your hand": the card, unless it has since become a
            // permanent (a new object, CR 400.7), which the instruction can't find.
            Sel::All(Filter::And(v))
                if matches!(v.as_slice(), [Filter::In(_), Filter::Not(z)]
                    if matches!(z.as_ref(), Filter::InZone(ZoneKind::Battlefield))) =>
            {
                let Filter::In(x) = &v[0] else {
                    return self.gap("Sel::All");
                };
                let x = (**x).clone();
                self.sel(&x, case)
            }
            Sel::All(f) => {
                if let Some(z) = whole_zone(f) {
                    let s = self.whole_zone_phrase(z.0, z.1);
                    return decline(s, case);
                }
                let s = self.noun_det(f, Det::Each);
                // "Untap all creatures you control. They gain hexproof ...": the group
                // just named is "them".
                let key = format!("{f:?}");
                let repeated = self.last_group.as_deref() == Some(key.as_str());
                self.last_group = Some(key);
                if repeated && !s.contains(['{', '|']) && !matches!(case, Case::Poss) {
                    let pron = if matches!(case, Case::Subj) {
                        "they"
                    } else {
                        "them"
                    };
                    return format!("{{alt:{pron}|{s}}}");
                }
                decline(s, case)
            }
            Sel::Players(p) => self.player(p, case),
            // "You choose one of them": one of the cards revealed.
            Sel::Choose {
                chooser,
                filter: Filter::In(from),
                count,
                ..
            } if matches!(
                from.as_ref(),
                Sel::Var(crate::kw::reveal_from_hand::REVEALED)
            ) =>
            {
                let n = match count {
                    Value::Const(n) => number_word(*n),
                    other => self.value(other),
                };
                let mut s = format!("{n} of them");
                if !matches!(chooser, PlayerRef::You) {
                    let c = self.player(chooser, Case::Poss);
                    s = format!("{s} of {c} choice");
                }
                decline(s, case)
            }
            // "Reveal any number of green cards in your hand": up to all of them.
            Sel::Choose {
                chooser: PlayerRef::You,
                filter,
                count: Value::CountSel(all),
                up_to: true,
                ..
            } if matches!(all.as_ref(), Sel::All(f) if format!("{f:?}") == format!("{filter:?}")) =>
            {
                let n = self.noun_det(filter, Det::Plural);
                decline(format!("any number of {n}"), case)
            }
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
                // "target creature and all other creatures with the same name as that
                // creature": the union already leaves out the one named first.
                if let [Sel::Target(_), Sel::All(_)] = v.as_slice() {
                    if let Some(rest) = parts[1].strip_prefix("all ") {
                        parts[1] = format!("all {{opt:other}} {rest}");
                    } else {
                        parts[1] = format!("{{opt:other}} {}", parts[1]);
                    }
                }
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
            // "Whenever a land enters under an opponent's control, that player loses 2
            // life": the opponent the trigger named.
            PlayerRef::ControllerOf(sel)
                if self.trigger_names_opponent
                    && matches!(sel.as_ref(), Sel::TriggerObject | Sel::TriggerLki) =>
            {
                "{alt:that player|its controller}".into()
            }
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
            // "Whenever an opponent draws a card, they lose 2 life" / "... that player
            // loses 2 life".
            PlayerRef::TriggerPlayer | PlayerRef::Iterated => {
                return match case {
                    Case::Subj => "{alt:that player|they}".into(),
                    Case::Obj => "{alt:that player|them}".into(),
                    Case::Poss => "{alt:that player's|their}".into(),
                }
            }
            PlayerRef::Var(_) => "that player".into(),
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
            PlayerFilter::FirstDrawInDrawStep => self.gap("PlayerFilter::FirstDrawInDrawStep"),
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
            PlayerFilter::LessThanHalfStartingLife => {
                "with less than half their starting life total".into()
            }
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
