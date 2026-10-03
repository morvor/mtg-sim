//! Object descriptions: [`Filter`] → noun phrases ("another target nontoken creature you
//! control", "creature cards with mana value 3 or less in your graveyard").

use super::players::Case;
use super::*;
use crate::keywords::KeywordKind;

/// How a noun phrase is quantified.
#[derive(Clone, Debug)]
pub enum Det {
    /// "a creature" / "another creature".
    A,
    /// "each creature" / "each other creature".
    Each,
    /// "creatures" / "other creatures" (also used for "all").
    Plural,
    /// "creature" with no determiner (inside other phrases: "Enchant creature").
    Bare,
    /// "two creatures".
    Count(String),
    /// "up to two creatures".
    UpTo(String),
    /// "one or more creatures".
    OneOrMore,
}

impl Det {
    fn num(&self) -> Num {
        match self {
            Det::A | Det::Each | Det::Bare => Num::One,
            Det::UpTo(n) | Det::Count(n) if n == "one" || n == "a" => Num::One,
            _ => Num::Many,
        }
    }
}

/// The parts of a noun phrase, before a determiner is chosen.
/// Where "cast from a [zone]" goes among a noun's relative phrases (see `np_text`).
const CAST_FROM: &str = "\u{0}cast-from";

#[derive(Clone, Debug, Default)]
pub(crate) struct Np {
    other: bool,
    /// "~" (the object itself).
    is_self: bool,
    /// A complete phrase that replaces the head ("enchanted creature").
    fixed: Option<String>,
    status: Vec<String>,
    supers: Vec<String>,
    colors: Vec<String>,
    quality: Vec<String>,
    nons: Vec<String>,
    subtypes: Vec<String>,
    types: Vec<CardType>,
    /// Alternative heads joined by "or"/"and" ("artifact or enchantment").
    alts: Vec<String>,
    /// "card", "token", "spell", "permanent".
    kind: Option<&'static str>,
    /// "permanent card" (CR 110.4b): "Rebel permanent card", "nonland permanent card".
    permanent_card: bool,
    controller: Option<PlayerRel>,
    controller_matches: Option<String>,
    owner: Option<PlayerRel>,
    zone: Option<ZoneKind>,
    with: Vec<String>,
    /// Qualities that need "it/them" ("with a +1/+1 counter on it").
    with_on: Vec<String>,
    without: Vec<String>,
    rel: Vec<String>,
    post: Vec<String>,
    /// "cast from a graveyard" (CR 601.2a: the zone the spell was cast from).
    cast_from: Option<ZoneKind>,
    /// A quality already says where the object is ("exiled with it").
    zone_said: bool,
}

/// CR 205.2a order in which card types are printed together ("artifact creature",
/// "enchantment creature", "artifact land", "kindred instant").
fn type_order(t: CardType) -> u8 {
    match t {
        CardType::Kindred => 0,
        CardType::Enchantment => 1,
        CardType::Artifact => 2,
        CardType::Land => 3,
        CardType::Creature => 4,
        CardType::Battle => 5,
        CardType::Planeswalker => 6,
        CardType::Instant => 7,
        CardType::Sorcery => 8,
        _ => 9,
    }
}

pub(crate) fn supertype_word(s: Supertype) -> &'static str {
    match s {
        Supertype::Basic => "basic",
        Supertype::Legendary => "legendary",
        Supertype::Ongoing => "ongoing",
        Supertype::Snow => "snow",
        Supertype::World => "world",
    }
}

/// A number written as such ("3", "X"), which "or less" can follow ("3 or less"); a
/// number described in words ("the number of lands you control") is compared with "less
/// than or equal to" instead. ("That much" and "that mana value" keep "or less".)
pub(crate) fn is_plain_number(v: &str) -> bool {
    v.parse::<i64>().is_ok() || v == "X" || v.starts_with("that ")
}

pub(crate) fn cmp_phrase(cmp: Cmp, v: &str) -> String {
    match cmp {
        // "with mana value less than or equal to that damage": the event's amount.
        Cmp::Le if v == "that much" || v == "{alt:that much|that many}" => {
            "{alt:that much or less|less than or equal to that damage|less than or equal to that much}"
                .into()
        }
        Cmp::Le if !is_plain_number(v) => format!("less than or equal to {v}"),
        Cmp::Ge if !is_plain_number(v) => format!("greater than or equal to {v}"),
        // "with mana value 3" / "with mana value equal to the number of ...".
        Cmp::Eq if v.parse::<i64>().is_ok() || v == "X" => v.to_string(),
        Cmp::Eq => format!("{{opt:equal to}} {v}"),
        Cmp::Ne => format!("other than {v}"),
        Cmp::Lt => format!("less than {v}"),
        // "with mana value less than or equal to ~'s power".
        Cmp::Le if v.parse::<i64>().is_err() && v != "X" => {
            format!("{{alt:{v} or less|less than or equal to {v}}}")
        }
        Cmp::Le => format!("{v} or less"),
        Cmp::Gt => format!("greater than {v}"),
        Cmp::Ge if v.parse::<i64>().is_err() && v != "X" => {
            format!("{{alt:{v} or greater|greater than or equal to {v}}}")
        }
        Cmp::Ge => format!("{v} or greater"),
    }
}

impl Renderer<'_> {
    /// `[A and Q, B and Q]` where Q is who controls or owns them: "A or B [Q]".
    fn shared_tail_union(&mut self, v: &[Filter], det: Det) -> Option<String> {
        if v.len() < 2 || !matches!(det.num(), Num::One) {
            return None;
        }
        let is_q = |f: &Filter| matches!(f, Filter::ControlledBy(_) | Filter::OwnedBy(_));
        let mut shared: Option<String> = None;
        let mut rests = Vec::new();
        for x in v {
            let Filter::And(parts) = x else { return None };
            let (q, rest): (Vec<&Filter>, Vec<&Filter>) = parts.iter().partition(|p| is_q(p));
            if q.is_empty() || rest.is_empty() {
                return None;
            }
            let key = format!("{q:?}");
            match &shared {
                None => shared = Some(key),
                Some(k) if *k == key => {}
                Some(_) => return None,
            }
            rests.push((Filter::and(rest.into_iter().cloned().collect()), x.clone()));
        }
        let (r0, full0) = rests.first()?.clone();
        let with = self.noun_det(&full0, det.clone());
        let without = self.noun_det(&r0, det.clone());
        let tail = with.strip_prefix(without.as_str())?.to_string();
        if tail.is_empty() || tail.contains('{') {
            return None;
        }
        let parts: Vec<String> = rests
            .iter()
            .map(|(r, _)| self.noun_det(r, det.clone()))
            .collect();
        let full: Vec<String> = rests
            .iter()
            .map(|(_, f)| self.noun_det(f, det.clone()))
            .collect();
        // Cards say it once or for each.
        Some(format!(
            "{{alt:{}{tail}|{}}}",
            join_list(&parts, "or"),
            join_list(&full, "or")
        ))
    }

    /// "with lesser mana value", "with greater power": compared with the same quality of
    /// the object the ability is about (itself, or the object or spell that triggered
    /// it), which the card leaves unsaid.
    fn lesser_greater(&mut self, c: Cmp, v: &Value, quality: &str) -> Option<String> {
        let s = match (quality, v) {
            ("power", Value::PowerOf(s))
            | ("toughness", Value::ToughnessOf(s))
            | ("mana value", Value::ManaValueOf(s)) => s,
            _ => return None,
        };
        if !matches!(
            s.as_ref(),
            Sel::This | Sel::TriggerObject | Sel::TriggerLki | Sel::TriggerSpell
        ) {
            return None;
        }
        let word = match c {
            Cmp::Lt => "lesser",
            Cmp::Gt => "greater",
            _ => return None,
        };
        let v = self.value(v);
        Some(format!(
            "{{alt:{quality} {}|{word} {quality}}}",
            cmp_phrase(c, &v)
        ))
    }

    /// What an object shares a quality with: any one of a group ("a creature you
    /// control", see `eval.rs`), or a selected object.
    fn shared_with(&mut self, s: &Sel) -> String {
        let r = self.sel(s, Case::Obj);
        if !matches!(s, Sel::All(_)) {
            return r;
        }
        // Any one of them: "each creature you control" is "a creature you control".
        let r = r.replace("{alt:each ", "{alt:a ").replace("|each ", "|a ");
        match r.strip_prefix("each ") {
            Some(rest) => with_article(rest),
            None => r,
        }
    }

    /// The noun used for "enchanted [thing]" / "equipped creature" on this face.
    pub(crate) fn attached_noun(&mut self) -> String {
        if self.equipment_holder_depth == Some(self.quote_depth) {
            return "equipped creature".into();
        }
        if self.info.has_subtype("Equipment") {
            return "equipped creature".into();
        }
        if self.info.has_subtype("Fortification") {
            return "fortified land".into();
        }
        match &self.info.enchant {
            // "Enchant creature card in a graveyard": "enchanted creature" (and "enchanted
            // creature card" for the card itself, see `move_effect`).
            Some(n) if n.contains(" card") => {
                let head = n.split(" card").next().unwrap_or(n);
                let head = head.rsplit(' ').next().unwrap_or(head);
                format!("enchanted {head}")
            }
            // "Enchant artifact or creature" Auras say "enchanted permanent"; "Enchant
            // nonland permanent" ones "enchanted permanent"; "Enchant opponent" ones
            // "enchanted player".
            // "Enchant green or white creature": the alternatives are adjectives.
            Some(n)
                if n.contains(" or ")
                    && n.rsplit_once(' ').is_some_and(|(adjs, _)| {
                        adjs.split_whitespace()
                            .map(|a| a.trim_end_matches(','))
                            .filter(|a| *a != "or")
                            .all(|a| CardType::from_word(a).is_none())
                    }) =>
            {
                let head = n.rsplit(' ').next().unwrap_or(n);
                format!("enchanted {head}")
            }
            Some(n) if n.contains(" or ") || n.contains(", ") => "enchanted permanent".into(),
            Some(n) if n == "opponent" => "enchanted player".into(),
            Some(n) => {
                let head = n.rsplit(' ').next().unwrap_or(n);
                format!("enchanted {head}")
            }
            // A creature with bestow is an Aura with enchant creature (CR 702.103b).
            None if self.info.card_types.contains(CardType::Creature) => {
                "enchanted creature".into()
            }
            None => "enchanted permanent".into(),
        }
    }

    /// A keyword as it appears inside a "with"/"has" phrase: lowercase.
    pub(crate) fn keyword_kind_word(&self, k: KeywordKind) -> String {
        k.name().to_lowercase()
    }

    fn is_type_like(f: &Filter) -> bool {
        match f {
            Filter::Type(_)
            | Filter::Subtype(_)
            | Filter::Permanent
            | Filter::Card
            | Filter::Token
            | Filter::Spell => true,
            Filter::And(v) => v.iter().all(Self::is_type_like),
            _ => false,
        }
    }

    fn kind_of(f: &Filter) -> Option<&'static str> {
        match f {
            Filter::Card => Some("card"),
            Filter::Token => Some("token"),
            Filter::Spell => Some("spell"),
            Filter::And(v) => v.iter().find_map(Self::kind_of),
            _ => None,
        }
    }

    fn type_like_head(&mut self, f: &Filter) -> String {
        let mut np = Np::default();
        self.collect(f, &mut np);
        self.head(&np)
    }

    /// Splits a filter into the parts of a noun phrase.
    pub(crate) fn collect(&mut self, f: &Filter, np: &mut Np) {
        match f {
            Filter::Any => {}
            Filter::And(v) => {
                // "a 1/1 creature": power and toughness both exactly given.
                let pt = v.iter().find_map(|x| match x {
                    Filter::Power(Cmp::Eq, p) => p.as_const(),
                    _ => None,
                });
                let tt = v.iter().find_map(|x| match x {
                    Filter::Toughness(Cmp::Eq, t) => t.as_const(),
                    _ => None,
                });
                let both = pt.is_some() && tt.is_some();
                if let (Some(p), Some(t)) = (pt, tt) {
                    np.status.push(format!("{p}/{t}"));
                }
                // The same quality twice says it once.
                let mut seen: Vec<String> = Vec::new();
                for x in v {
                    if both
                        && matches!(x, Filter::Power(Cmp::Eq, _) | Filter::Toughness(Cmp::Eq, _))
                    {
                        continue;
                    }
                    let k = format!("{x:?}");
                    if seen.contains(&k) {
                        continue;
                    }
                    seen.push(k);
                    // "with the greatest power among creatures they control".
                    if let Filter::ValueCmp(a, c, b) = x {
                        if let Some(q) = self.extreme_quality(a, *c, b, v) {
                            np.post.push(q);
                            continue;
                        }
                    }
                    self.collect(x, np);
                }
            }
            // CR 700.12: an outlaw is an object with the Assassin, Mercenary, Pirate,
            // Rogue, and/or Warlock creature types.
            Filter::Or(v) if is_outlaw(v) => np.subtypes.push("outlaw".into()),
            // "a land card with a basic land type" (CR 305.6: the five basic land types).
            Filter::Or(v) if is_basic_land_types(v) && np.types.contains(&CardType::Land) => {
                np.post.push("with a basic land type".into())
            }
            // "artifact, enchantment, or tapped creature": one list.
            Filter::Or(v) if v.iter().any(|x| matches!(x, Filter::Or(_))) => {
                let flat = flatten_or(f);
                self.collect(&flat, np)
            }
            // "a card named Festering Newt or Bubbling Cauldron".
            Filter::Or(v) if v.len() > 1 && v.iter().all(|x| matches!(x, Filter::Named(_))) => {
                let names: Vec<String> = v
                    .iter()
                    .filter_map(|x| match x {
                        Filter::Named(n) => Some(n.to_string()),
                        _ => None,
                    })
                    .collect();
                np.post.push(format!("named {}", join_list(&names, "or")));
            }
            // "artifact, enchantment, or legendary card": a supertype among card types,
            // the alternatives sharing the noun.
            Filter::Or(v)
                if v.iter().any(|x| matches!(x, Filter::Supertype(_)))
                    && v.iter().all(|x| {
                        matches!(
                            x,
                            Filter::Supertype(_) | Filter::Type(_) | Filter::Subtype(_)
                        )
                    }) =>
            {
                for x in v {
                    np.alts.push(match x {
                        Filter::Supertype(t) => supertype_word(*t).to_string(),
                        Filter::Type(t) => t.word().to_string(),
                        Filter::Subtype(t) => t.to_string(),
                        _ => String::new(),
                    });
                }
            }
            Filter::Or(v) => {
                let status_word = |x: &Filter| -> Option<&'static str> {
                    Some(match x {
                        Filter::Attacking => "attacking",
                        Filter::Blocking => "blocking",
                        Filter::Tapped => "tapped",
                        Filter::Untapped => "untapped",
                        Filter::Blocked => "blocked",
                        Filter::Unblocked => "unblocked",
                        _ => return None,
                    })
                };
                if v.iter().all(|x| status_word(x).is_some()) {
                    let w: Vec<String> = v
                        .iter()
                        .filter_map(|x| status_word(x).map(str::to_string))
                        .collect();
                    np.status.push(join_list(&w, "or"));
                } else if v.iter().all(|x| matches!(x, Filter::Color(_))) {
                    let cs: Vec<String> = v
                        .iter()
                        .map(|x| match x {
                            Filter::Color(c) => c.word().to_string(),
                            _ => String::new(),
                        })
                        .collect();
                    // "white or blue creature" / "white and/or blue creature".
                    np.colors.push(join_list(&cs, "{alt:or|and/or}"));
                } else if v.iter().all(Self::is_type_like) {
                    // A kind all the alternatives share is said once: "Merfolk and Druid
                    // cards", "instant or sorcery card".
                    let kinds: Vec<Option<&'static str>> = v.iter().map(Self::kind_of).collect();
                    let shared = kinds[0].filter(|k| kinds.iter().all(|x| *x == Some(*k)));
                    let alts: Vec<String> = v
                        .iter()
                        .map(|x| {
                            let h = self.type_like_head(x);
                            match shared {
                                Some(k) if h != k => h
                                    .strip_suffix(&format!(" {k}"))
                                    .map(|s| s.to_string())
                                    .unwrap_or(h),
                                _ => h,
                            }
                        })
                        .collect();
                    if let Some(k) = shared {
                        if np.kind.is_none() {
                            np.kind = Some(k);
                        }
                    }
                    np.alts.extend(alts);
                } else if self.plural_alts {
                    // "Red spells and white spells you cast cost {1} less to cast."
                    let alts: Vec<String> = v.iter().map(|x| self.noun(x, Num::Many)).collect();
                    np.fixed = Some(join_list(&alts, "and"));
                } else if let Some(s) = self.shared_tail_union(v, Det::Bare) {
                    np.fixed = Some(s);
                } else {
                    let alts: Vec<String> = v.iter().map(|x| self.noun(x, Num::One)).collect();
                    np.fixed = Some(join_list(&alts, "or"));
                }
            }
            Filter::Not(inner) => self.collect_not(inner, np),
            Filter::Type(t) => np.types.push(*t),
            Filter::Supertype(s) => np.supers.push(supertype_word(*s).into()),
            Filter::Subtype(s) => np.subtypes.push(s.to_string()),
            Filter::Color(c) => np.colors.push(c.word().into()),
            Filter::ExactColors(cs) => {
                let w: Vec<String> = cs.iter().map(|c| c.word().to_string()).collect();
                np.with.push(format!("exactly {}", join_list(&w, "and")));
            }
            Filter::Colorless => np.colors.push("colorless".into()),
            Filter::Multicolored => np.colors.push("multicolored".into()),
            Filter::Monocolored => np.colors.push("monocolored".into()),
            Filter::Permanent if np.kind == Some("commander") => {}
            Filter::Permanent => np.kind = Some("permanent"),
            Filter::PermanentCard => {
                np.permanent_card = true;
                np.kind = Some("card");
            }
            Filter::Spell | Filter::SpellOnStack => np.kind = Some("spell"),
            Filter::Token => np.kind = Some("token"),
            Filter::Card => {
                if np.kind.is_none() {
                    np.kind = Some("card")
                }
            }
            Filter::Copy => np.rel.push("that's a copy".into()),
            Filter::ControlledBy(r) => np.controller = Some(*r),
            Filter::OwnedBy(r) => np.owner = Some(*r),
            Filter::ControlledByPlayer(p) => {
                let w = self.player(p, Case::Subj);
                let v = if matches!(w.as_str(), "you" | "they") {
                    "control"
                } else {
                    "controls"
                };
                np.post.push(format!("{w} {v}"));
            }
            Filter::OwnedByPlayer(p) => {
                let w = self.player(p, Case::Subj);
                let v = if matches!(w.as_str(), "you" | "they") {
                    "own"
                } else {
                    "owns"
                };
                np.post.push(format!("{w} {v}"));
            }
            Filter::AttachedToAnyOf(s) => {
                let w = self.sel(s, Case::Obj);
                np.post.push(format!("attached to {w}"));
            }
            // "with toughness greater than its power": a value of the object itself
            // compared with another.
            Filter::ValueCmp(a, c, b) => {
                let own = |v: &Value| -> Option<&'static str> {
                    let tested = |s: &Sel| matches!(s, Sel::Var(crate::ability::vars::TESTED));
                    match v {
                        Value::PowerOf(s) if tested(s) => Some("power"),
                        Value::ToughnessOf(s) if tested(s) => Some("toughness"),
                        Value::ManaValueOf(s) if tested(s) => Some("mana value"),
                        _ => None,
                    }
                };
                // "total power and toughness": the sum of an object's power and toughness.
                let total = |v: &Value| -> Option<Option<Sel>> {
                    let Value::Sum(parts) = v else {
                        return None;
                    };
                    match parts.as_slice() {
                        [Value::PowerOf(p), Value::ToughnessOf(t)]
                            if format!("{p:?}") == format!("{t:?}") =>
                        {
                            let tested =
                                matches!(p.as_ref(), Sel::Var(crate::ability::vars::TESTED));
                            Some((!tested).then(|| (**p).clone()))
                        }
                        _ => None,
                    }
                };
                if let Some(None) = total(a) {
                    let w = match (total(b), c) {
                        // "with the same total power and toughness" (as the object named).
                        (Some(Some(other)), Cmp::Eq) => {
                            let o = self.sel(&other, Case::Obj);
                            format!("with the same total power and toughness {{opt:as {o}}}")
                        }
                        (None, _) => {
                            let v = self.value(b);
                            format!("with total power and toughness {}", cmp_phrase(*c, &v))
                        }
                        _ => self.gap("a comparison of values of the object"),
                    };
                    np.post.push(w);
                    return;
                }
                let w = match (own(a), own(b)) {
                    (Some(x), Some(y)) => {
                        format!("with {x} {}", cmp_phrase(*c, &format!("its {y}")))
                    }
                    (Some(x), None) => {
                        let v = self.value(b);
                        format!("with {x} {}", cmp_phrase(*c, &v))
                    }
                    _ => self.gap("a comparison of values of the object"),
                };
                np.post.push(w);
            }
            Filter::Together(g) => {
                let w = self.target_group(g);
                np.post.push(w);
            }
            Filter::ControllerMatches(pf) => {
                let p = self.player_filter_noun(pf, Num::One);
                np.controller_matches = Some(format!("whose controller is {}", with_article(&p)));
            }
            Filter::InZone(z) => np.zone = Some(*z),
            Filter::Tapped => np.status.push("tapped".into()),
            Filter::Untapped if np.status.iter().any(|x| x == "untapped") => {}
            Filter::Untapped => np.status.push("untapped".into()),
            Filter::Attacking => np.status.push("attacking".into()),
            Filter::Blocking => np.status.push("blocking".into()),
            Filter::Blocked => np.status.push("blocked".into()),
            Filter::Unblocked => np.status.push("unblocked".into()),
            Filter::AttackingPlayer(r) => {
                let who = self.rel_object(*r);
                np.post.push(format!("attacking {who}"));
            }
            Filter::BlockingSource if self.self_before_target => {
                np.post.push("blocking ~it".into())
            }
            Filter::BlockingSource => {
                let m = self.me();
                np.post.push(format!("blocking {m}"))
            }
            Filter::BlockedBySource if self.self_before_target => {
                np.post.push("blocked by ~it".into())
            }
            Filter::BlockedBySource => {
                let m = self.me();
                np.post.push(format!("blocked by {m}"))
            }
            Filter::BlockingAnyOf(s) => {
                let s = self.sel(s, Case::Obj);
                np.post.push(format!("blocking {s}"));
            }
            Filter::TargetOf(s) => {
                let s = self.sel(s, Case::Obj);
                np.rel.push(format!("targeted by {s}"));
            }
            Filter::AttackingAlone => np.post.push("attacking alone".into()),
            Filter::BlockingAlone => np.post.push("blocking alone".into()),
            Filter::AttackingPlayerAlone => np.post.push("attacking a player alone".into()),
            Filter::HadToAttack => np.rel.push("that had to attack".into()),
            Filter::Power(c, v) => {
                let q = match self.lesser_greater(*c, v, "power") {
                    Some(q) => q,
                    None => {
                        let v = self.value(v);
                        format!("power {}", cmp_phrase(*c, &v))
                    }
                };
                np.with.push(q);
            }
            Filter::Toughness(c, v) => {
                let q = match self.lesser_greater(*c, v, "toughness") {
                    Some(q) => q,
                    None => {
                        let v = self.value(v);
                        format!("toughness {}", cmp_phrase(*c, &v))
                    }
                };
                np.with.push(q);
            }
            Filter::PowerVsBase(c) => {
                np.with
                    .push(format!("power {}", cmp_phrase(*c, "its base power")));
            }
            Filter::ManaValue(c, v) => {
                let q = match self.lesser_greater(*c, v, "mana value") {
                    Some(q) => q,
                    None => {
                        let v = self.value(v);
                        format!("mana value {}", cmp_phrase(*c, &v))
                    }
                };
                np.with.push(q);
            }
            Filter::Loyalty(c, v) => {
                let v = self.value(v);
                np.with.push(format!("loyalty {}", cmp_phrase(*c, &v)));
            }
            Filter::Named(n) => np.post.push(format!("named {n}")),
            // "with the same name as a card exiled with ~": as any of them.
            Filter::SameNameAs(s) => {
                let s = self.sel(s, Case::Obj);
                let s = s.replace("|each card exiled with ~it}", "|a card exiled with ~it}");
                np.with.push(format!("the same name as {s}"));
            }
            Filter::DifferentNameFrom(s) => {
                let s = self.sel(s, Case::Obj);
                np.with.push(format!("a different name than {s}"));
            }
            Filter::NameOriginallyPrintedIn(set) => np
                .with
                .push(format!("a name originally printed in the {set} expansion")),
            // Sharing a quality with any of a group of objects: "that shares a color with a
            // creature you control".
            Filter::SharesCreatureType(s) => {
                let s = self.shared_with(s);
                np.rel.push(format!("that shares a creature type with {s}"));
            }
            Filter::SharesCardType(s) => {
                let s = self.shared_with(s);
                np.rel.push(format!("that shares a card type with {s}"));
            }
            Filter::SharesColor(s) => {
                let s = self.shared_with(s);
                np.rel.push(format!("that shares a color with {s}"));
            }
            Filter::HasKeyword(k) => np.with.push(self.keyword_kind_word(*k)),
            Filter::HasCounter(k) => np.with_on.push(match k {
                Some(k) => counter_name(k),
                None => "counter".into(),
            }),
            // "with three or more +1/+1 counters on it", "with exactly one tide counter on it".
            Filter::CounterCount(k, cmp, v) => {
                let noun = match k {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let n = self.value(v);
                let amount = match cmp {
                    Cmp::Ge => format!("{n} or more"),
                    Cmp::Le => format!("{n} or fewer"),
                    Cmp::Eq => format!("exactly {n}"),
                    Cmp::Gt => format!("more than {n}"),
                    Cmp::Lt => format!("fewer than {n}"),
                    Cmp::Ne => format!("other than {n}"),
                };
                np.with.push(format!("{amount} {} on it", plural(&noun)));
            }
            Filter::HasAbilities => np.with.push("an ability".into()),
            Filter::Source => np.is_self = true,
            Filter::Other => np.other = true,
            // "a creature dealt damage this way".
            Filter::In(s) if matches!(s.as_ref(), Sel::Var(crate::ability::vars::DAMAGED)) => {
                np.post.push("dealt damage this way".into())
            }
            // "a card exiled with ~": the cards an ability of this object exiled
            // (CR 607.2a).
            Filter::In(s) if matches!(s.as_ref(), Sel::Linked) => {
                np.zone_said = true;
                np.post.push("exiled with ~it".into());
            }
            // "for each creature card exiled this way": the cards the instruction before
            // exiled.
            Filter::In(s)
                if self.after_exile
                    && matches!(s.as_ref(), Sel::Var(v)
                        if *v == crate::oracle::patterns::hand_graveyard_grammar::AFFECTED) =>
            {
                np.post.push("exiled this way".into());
                np.kind.get_or_insert("card");
            }
            // "for each card revealed this way".
            Filter::In(s)
                if matches!(s.as_ref(), Sel::Var(crate::kw::reveal_from_hand::REVEALED)) =>
            {
                np.post.push("revealed this way".into());
                np.kind.get_or_insert("card");
            }
            // "for each permanent destroyed this way".
            Filter::In(s) if self.this_way_of(s).is_some() => {
                let (verb, card, ty) = self.this_way_of(s).unwrap_or(("", false, None));
                np.post.push(format!("{verb} this way"));
                if let (Some(t), true) = (ty, np.types.is_empty()) {
                    np.types.push(t);
                }
                if card {
                    np.kind.get_or_insert("card");
                }
            }
            // "a permanent card from among the milled cards".
            Filter::In(s)
                if self.milled
                    && matches!(s.as_ref(), Sel::Var(v) if *v == crate::ability::vars::IT) =>
            {
                np.post.push(
                    "{alt:among them|from among them|from among the milled cards|from among them milled this way}"
                        .into(),
                );
            }
            // "for each Aura attached to it".
            Filter::In(s) if matches!(s.as_ref(), Sel::AttachedToThis) => {
                np.post.push("attached to {alt:~|~it}".into());
            }
            Filter::In(s) => {
                let s = self.sel(s, Case::Obj);
                np.post.push(format!("among {s}"));
            }
            Filter::Objects(_) => np.fixed = Some(self.gap("Filter::Objects")),
            Filter::AttachedToSource => np.fixed = Some(self.attached_noun()),
            // "target Aura attached to a creature".
            Filter::AttachedTo(s) => {
                let s = self.sel(s, Case::Obj);
                np.post.push(format!("attached to {s}"));
            }
            // What an Aura is moved to must be something it can enchant: the rules say so
            // (CR 303.4j, 701.3b), and the card doesn't.
            Filter::CanBeAttachedBy(_) => {}
            Filter::Attached => np.status.push("attached".into()),
            Filter::Enchanted => np.status.push("enchanted".into()),
            Filter::Equipped => np.status.push("equipped".into()),
            Filter::EnteredThisTurn => np.rel.push("that entered this turn".into()),
            Filter::DealtDamageThisTurn => np.rel.push("that was dealt damage this turn".into()),
            Filter::Historic => np.quality.push("historic".into()),
            Filter::FaceDown => np.status.push("face-down".into()),
            Filter::HasX => np.with.push("{X} in its mana cost".into()),
            Filter::HasPhyrexianMana => np.with.push("{H} in its mana cost".into()),
            Filter::Commander => np.kind = Some("commander"),
            Filter::Modified => np.status.push("modified".into()),
            Filter::DiedThisTurn => np.rel.push("that died this turn".into()),
            Filter::AttackedThisTurn => np.rel.push("that attacked this turn".into()),
            // "Choose a color. ... each card of that color": the color just chosen.
            Filter::ChosenColor => np.post.push("of {alt:the chosen color|that color}".into()),
            Filter::LinkedChosenColor => np.post.push("of the chosen color".into()),
            Filter::ChosenType | Filter::LinkedChosenCreatureType => {
                np.post.push("of {alt:the chosen type|that type}".into())
            }
            Filter::ChosenName => np.with.push("{alt:the chosen name|that name}".into()),
            Filter::ChosenCardType => np.post.push("of the chosen card type".into()),
            Filter::Prepared => np.status.push("prepared".into()),
            Filter::Targets(f) => {
                let t = self.noun_det(f, Det::A);
                np.rel.push(format!("that targets {t}"));
            }
            // CR 123.4: an object with any kind of sticker on it is "stickered".
            Filter::HasSticker(None) => np.status.push("stickered".into()),
            Filter::HasSticker(k) => np.with_on.push(match k {
                None => "sticker".into(),
                Some(StickerType::Name) => "name sticker".into(),
                Some(StickerType::Ability) => "ability sticker".into(),
                Some(StickerType::PowerToughness) => "power and toughness sticker".into(),
                Some(StickerType::Art) => "art sticker".into(),
            }),
            Filter::StackTargets(tf) => {
                let s = self.targets_filter(tf);
                np.rel.push(s);
            }
            // Worded in `np_text`, where the owner is known.
            Filter::CastFrom(z) => {
                np.cast_from = Some(*z);
                np.rel.push(CAST_FROM.into());
            }
            Filter::CastWithCost(name) => np.status.push(name.to_string()),
            Filter::DealtDamageThisTurnBy(s) => {
                let s = self.sel(s, Case::Obj);
                np.rel.push(format!("dealt damage by {s} this turn"));
            }
            Filter::ManaValueOfChosenQuality => {
                np.with.push("mana value of the chosen quality".into())
            }
            // "activated ability from an artifact source".
            Filter::AbilityFrom(f) => {
                let saved = self.default_head.replace("source");
                let s = self.noun_det(f, Det::A);
                self.default_head = saved;
                np.post.push(format!("from {s}"));
            }
            Filter::Custom(name)
                if crate::kw::basic_effects::same_name_as_another_sel(name).is_some() =>
            {
                let sel =
                    crate::kw::basic_effects::same_name_as_another_sel(name).unwrap_or(Sel::None);
                let s = self.sel(&sel, Case::Obj);
                np.with.push(format!("the same name as {s}"));
            }
            Filter::Custom(name) => {
                if let Some(q) = super::custom_filters::custom_rel(name) {
                    use super::custom_filters::CustomQuality as Q;
                    match q {
                        Q::Rel(s) => np.rel.push(s),
                        Q::RelInExile(s) => {
                            np.rel.push(s);
                            np.zone_said = true;
                        }
                        Q::Implied => {}
                        Q::Kind(k) => np.kind = Some(k),
                        Q::Adj(a) => np.status.push(a.to_string()),
                    }
                    return;
                }
                if let Some(s) = self.custom_rel_with_data(name) {
                    np.rel.push(s);
                    return;
                }
                let (adj, s) = self.custom_filter_quality(name);
                if adj {
                    np.status.push(s);
                } else {
                    np.rel.push(s);
                }
            }
        }
    }

    fn collect_not(&mut self, inner: &Filter, np: &mut Np) {
        match inner {
            Filter::Type(t) => np.nons.push(format!("non{}", t.word())),
            Filter::Subtype(s) => np.nons.push(format!("non-{s}")),
            Filter::Supertype(s) => np.nons.push(format!("non{}", supertype_word(*s))),
            Filter::Color(c) => np.colors.push(format!("non{}", c.word())),
            Filter::Token => np.nons.push("nontoken".into()),
            Filter::Commander => np.nons.push("noncommander".into()),
            Filter::Historic => np.nons.push("nonhistoric".into()),
            Filter::Attacking => np.status.push("nonattacking".into()),
            Filter::Blocking => np.status.push("nonblocking".into()),
            Filter::Tapped => np.status.push("untapped".into()),
            Filter::Untapped => np.status.push("tapped".into()),
            Filter::Multicolored => np.colors.push("nonmulticolored".into()),
            Filter::Colorless => np.colors.push("colored".into()),
            Filter::FaceDown => np.status.push("face-up".into()),
            Filter::Source => np.other = true,
            // "each other permanent with the same name as that permanent": other than
            // the target just named.
            // "it gets +1/+1 for each other creature you control": other than "it".
            Filter::In(s)
                if matches!(
                    s.as_ref(),
                    Sel::Target(_) | Sel::TriggerObject | Sel::AttachedTo
                ) =>
            {
                np.other = true
            }
            Filter::Other => np.is_self = true,
            Filter::HasKeyword(k) => np.without.push(self.keyword_kind_word(*k)),
            Filter::HasAbilities => np.with.push("no abilities".into()),
            Filter::HasCounter(k) => np.without.push(match k {
                Some(k) => format!("{} on it", with_article(&counter_name(k))),
                None => "counters on it".into(),
            }),
            Filter::Named(n) => np.rel.push(format!("not named {n}")),
            Filter::ControlledBy(PlayerRel::You) => np.controller = Some(PlayerRel::NotYou),
            Filter::OwnedBy(PlayerRel::You) => np.owner = Some(PlayerRel::NotYou),
            Filter::Card => np.nons.push("noncard".into()),
            Filter::Permanent => np.nons.push("nonpermanent".into()),
            Filter::Spell => np.nons.push("nonspell".into()),
            Filter::Copy => np.rel.push("that isn't a copy".into()),
            Filter::Enchanted => np.status.push("unenchanted".into()),
            Filter::Equipped => np.status.push("unequipped".into()),
            Filter::Modified => np.status.push("unmodified".into()),
            Filter::EnteredThisTurn => np.rel.push("that didn't enter this turn".into()),
            Filter::AttackedThisTurn => np.rel.push("that didn't attack this turn".into()),
            // "a spell from anywhere other than your hand": a spell is cast from its
            // caster's own hand (CR 601.2a); one that isn't in a hand either (Drannith
            // Magistrate's "can't cast spells from anywhere other than their hands").
            Filter::CastFrom(z) => np.post.push(format!(
                "from anywhere other than {{alt:your|their}} {}",
                zone_word(*z)
            )),
            Filter::Or(v)
                if v.len() == 2
                    && v.iter().any(|x| matches!(x, Filter::CastFrom(_)))
                    && v.iter().all(|x| match x {
                        Filter::CastFrom(z) | Filter::InZone(z) => v.iter().all(
                            |y| matches!(y, Filter::CastFrom(w) | Filter::InZone(w) if w == z),
                        ),
                        _ => false,
                    }) =>
            {
                let z = v
                    .iter()
                    .find_map(|x| match x {
                        Filter::CastFrom(z) => Some(*z),
                        _ => None,
                    })
                    .unwrap_or(ZoneKind::Hand);
                np.post.push(format!(
                    "from anywhere other than {{alt:your|their}} {}",
                    zone_word(z)
                ))
            }
            // "target permanent not named ~".
            Filter::SameNameAs(s) if matches!(s.as_ref(), Sel::This) => {
                np.post.push(format!("not named {}", self.me()))
            }
            Filter::Or(v) if is_outlaw(v) => np.nons.push("non-outlaw".into()),
            Filter::Or(v) => {
                for x in v {
                    self.collect_not(x, np);
                }
            }
            Filter::Power(c, v) => {
                let neg = match c {
                    Cmp::Eq => Cmp::Ne,
                    Cmp::Ne => Cmp::Eq,
                    Cmp::Lt => Cmp::Ge,
                    Cmp::Le => Cmp::Gt,
                    Cmp::Gt => Cmp::Le,
                    Cmp::Ge => Cmp::Lt,
                };
                self.collect(&Filter::Power(neg, v.clone()), np);
            }
            // "each creature that isn't of the chosen type".
            Filter::ChosenType => np
                .rel
                .push("that {alt:isn't|aren't} of {alt:the chosen type|that type}".into()),
            // "a creature spell that doesn't share a color with a creature you control".
            Filter::SharesColor(x) | Filter::SharesCardType(x) | Filter::SharesCreatureType(x) => {
                let what = match other_kind(inner) {
                    Some(k) => k,
                    None => "a color",
                };
                let w = self.shared_with(x);
                np.rel
                    .push(format!("that {{alt:doesn't|don't}} share {what} with {w}"));
            }
            other => {
                let s = self.noun(other, Num::One);
                np.rel.push(format!("that isn't {}", with_article(&s)));
            }
        }
    }

    /// The head noun ("artifact creature", "Elf", "permanent", "creature card").
    fn head(&mut self, np: &Np) -> String {
        let mut words: Vec<String> = Vec::new();
        // "commander creatures you own".
        let commander_first =
            np.kind == Some("commander") && (!np.types.is_empty() || !np.subtypes.is_empty());
        if commander_first {
            words.push("commander".into());
        }
        words.extend(np.subtypes.iter().cloned());
        let mut types = np.types.clone();
        types.sort_by_key(|t| type_order(*t));
        // "each Frog, Rabbit, Raccoon, or Squirrel" / "Knights and Walls": the same kinds
        // either way. With a card type, the alternatives come first: "a Wolf or Werewolf
        // creature".
        let conj = if self.alt_and { "{alt:and|or}" } else { "or" };
        let alts_first = !np.alts.is_empty() && !types.is_empty() && np.subtypes.is_empty();
        if alts_first {
            words.push(join_list(&np.alts, conj));
        }
        words.extend(types.iter().map(|t| t.word().to_string()));
        if !np.alts.is_empty() && !alts_first {
            words.push(join_list(&np.alts, conj));
        }
        if np.permanent_card {
            words.push("permanent".into());
        }
        match np.kind {
            Some("permanent") if !words.is_empty() => {}
            Some("commander") if commander_first => {}
            Some(k) => words.push(k.to_string()),
            None => {}
        }
        words.join(" ")
    }

    /// Plural of a noun phrase whose head may be a list of alternatives
    /// ("artifacts and enchantments").
    fn plural_head(&mut self, np: &Np) -> String {
        if np.alts.len() > 1 && np.subtypes.is_empty() && np.types.is_empty() {
            let suffix = match np.kind {
                Some("permanent") | None => String::new(),
                Some(k) => format!(" {k}"),
            };
            let alts: Vec<String> = np
                .alts
                .iter()
                .map(|a| {
                    if suffix.is_empty() {
                        plural(a)
                    } else {
                        a.clone()
                    }
                })
                .collect();
            let list = join_list(&alts, "and");
            if suffix.is_empty() {
                // "Knights and Walls" / "Knights or Walls".
                return join_list(&alts, "{alt:and|or}");
            }
            // "Elemental spells and Warrior spells" / "instant and sorcery spells".
            if suffix == " spell" && alts.len() == 2 {
                let each: Vec<String> = alts.iter().map(|a| format!("{a} spells")).collect();
                return format!(
                    "{{alt:{}|{}}}",
                    plural(&format!("{list}{suffix}")),
                    join_list(&each, "and")
                );
            }
            return plural(&format!("{list}{suffix}"));
        }
        let h = self.head(np);
        plural(&h)
    }

    /// A noun phrase without determiner.
    pub(crate) fn noun(&mut self, f: &Filter, num: Num) -> String {
        if let Filter::Custom(n) = f {
            if let Some(s) = self.custom_noun(n) {
                return s;
            }
        }
        let mut np = Np::default();
        self.collect(f, &mut np);
        let s = self.np_text(&np, num, false);
        if np.other && !np.is_self {
            format!("other {s}")
        } else {
            s
        }
    }

    /// The default head when a filter names no type: depends on the zone.
    fn default_kind(np: &Np) -> &'static str {
        // Only creatures attack, block, and have power and toughness (CR 506.4, 208.1):
        // "attacking or blocking creature", "creatures with flying".
        let combat =
            np.status.iter().any(|s| {
                matches!(
                    s.as_str(),
                    "attacking"
                        | "blocking"
                        | "blocked"
                        | "unblocked"
                        | "attacking or blocking"
                        | "nonattacking"
                        | "nonblocking"
                )
            }) || np.with.iter().any(|w| {
                w.starts_with("power") || w.starts_with("toughness") || is_combat_keyword(w)
            }) || np.post.iter().any(|p| {
                p.starts_with("attacking") || p.starts_with("blocking") || p.starts_with("blocked")
            });
        if combat && matches!(np.zone, None | Some(ZoneKind::Battlefield)) {
            return "creature";
        }
        match np.zone {
            Some(ZoneKind::Battlefield) | None => "permanent",
            Some(ZoneKind::Stack) => "spell",
            _ => "card",
        }
    }

    pub(crate) fn np_text(&mut self, np: &Np, num: Num, _det_has_other: bool) -> String {
        if np.is_self {
            return self.me();
        }
        if let Some(f) = &np.fixed {
            let mut s = f.clone();
            // The qualities the alternatives share: "artifact, enchantment, or nonbasic land
            // an opponent controls", "artifact or non-Aura enchantment card in your
            // graveyard".
            let in_zone = np
                .zone
                .is_some_and(|z| !matches!(z, ZoneKind::Battlefield | ZoneKind::Stack));
            if in_zone && np.kind == Some("card") && !s.contains("card") && !s.contains('~') {
                s.push_str(match num {
                    Num::One => " card",
                    Num::Many => " cards",
                });
            }
            if let (Some(z), true) = (np.zone, in_zone) {
                if !s.contains(" in ") && !s.contains(" from ") {
                    s.push(' ');
                    s.push_str(&self.zone_phrase(z, np.owner, num));
                }
            }
            if let Some(c) = np.controller {
                s.push(' ');
                s.push_str(&self.controls_phrase(c, num));
            }
            for p in &np.post {
                s.push(' ');
                s.push_str(p);
            }
            for r in &np.rel {
                s.push(' ');
                s.push_str(r);
            }
            return s;
        }
        let mut np = np.clone();
        // Cards outside the battlefield and stack are "cards" (CR 108.1).
        if np.kind.is_none()
            && np.zone.is_some_and(|z| {
                !matches!(
                    z,
                    ZoneKind::Battlefield | ZoneKind::Stack | ZoneKind::Command
                )
            })
        {
            np.kind = Some("card");
        }
        let has_head = !np.subtypes.is_empty()
            || !np.types.is_empty()
            || !np.alts.is_empty()
            || np.kind.is_some();
        if !has_head {
            np.kind = Some(match self.default_head {
                Some(h) => h,
                None => Self::default_kind(&np),
            });
        }
        let mut words: Vec<String> = Vec::new();
        words.extend(np.status.iter().cloned());
        words.extend(np.supers.iter().cloned());
        // Oracle order: "a nonland historic permanent".
        let nons = np.nons.clone();
        words.extend(np.colors.iter().cloned());
        if !nons.is_empty() {
            let nons = nons.join(", ");
            words.push(nons);
        }
        // "nonland permanent card": the non- word goes before "permanent".
        words.extend(np.quality.iter().cloned());
        let head = match num {
            Num::One => self.head(&np),
            Num::Many => self.plural_head(&np),
        };
        words.push(head);
        let mut s = words.join(" ");
        if let Some(c) = np.controller {
            s.push(' ');
            s.push_str(&self.controls_phrase(c, num));
        }
        if let Some(c) = &np.controller_matches {
            s.push(' ');
            s.push_str(c);
        }
        for p in &np.post {
            s.push(' ');
            s.push_str(p);
        }
        let mut with: Vec<String> = np.with.clone();
        for w in &np.with_on {
            // Cards say "creatures with a +1/+1 counter on it" in the plural too.
            let _ = num;
            with.push(format!("{} on it", with_article(w)));
        }
        if !with.is_empty() {
            s.push_str(" with ");
            s.push_str(&join_list(&with, "and"));
        }
        if !np.without.is_empty() {
            s.push_str(" without ");
            s.push_str(&join_list(&np.without, "or"));
        }
        // "a spell from your graveyard": a card in your graveyard is yours (CR 400.3).
        if let Some(z) = np.cast_from {
            let z = zone_word(z);
            let w = if np.owner == Some(PlayerRel::You) {
                np.owner = None;
                format!("from your {z}")
            } else if z == "exile" {
                // "Whenever you cast a spell from exile", "spells cast from exile".
                "{opt:cast} from exile".to_string()
            } else if z == "hand" || z == "library" {
                // A spell cast from a hand or library is cast from its caster's (CR 601.2a:
                // a player casts the cards they may).
                format!("{{alt:{{opt:cast}} from your {z}|cast from a {z}}}")
            } else {
                format!("cast from a {z}")
            };
            for r in np.rel.iter_mut().filter(|r| *r == CAST_FROM) {
                *r = w.clone();
            }
        }
        match (np.zone, np.owner) {
            (Some(ZoneKind::Exile), None) if np.zone_said => {}
            // "a nonlegendary creature on the battlefield": a permanent is on the
            // battlefield anyway (CR 110.1), so cards may leave it out.
            (Some(ZoneKind::Battlefield), None) => s.push_str(" {opt:on the battlefield}"),
            (Some(z), owner) if !matches!(z, ZoneKind::Battlefield | ZoneKind::Stack) => {
                s.push(' ');
                s.push_str(&self.zone_phrase(z, owner, num));
            }
            (_, Some(o)) => {
                s.push(' ');
                s.push_str(&self.owns_phrase(o));
            }
            _ => {}
        }
        for r in &np.rel {
            s.push(' ');
            s.push_str(r);
        }
        s
    }

    /// "in your graveyard", "in an opponent's graveyard", "in exile".
    pub(crate) fn zone_phrase(
        &mut self,
        z: ZoneKind,
        owner: Option<PlayerRel>,
        num: Num,
    ) -> String {
        let zw = zone_word(z);
        match z {
            ZoneKind::Exile => {
                return match owner {
                    Some(o) => format!("{} in exile", self.owns_phrase(o)),
                    None => "in exile".into(),
                }
            }
            ZoneKind::Command | ZoneKind::Outside | ZoneKind::Stack | ZoneKind::Battlefield => {
                let z = match z {
                    ZoneKind::Command => "in the command zone",
                    ZoneKind::Outside => "from outside the game",
                    ZoneKind::Stack => "on the stack",
                    _ => "on the battlefield",
                };
                // "a sorcery card you own from outside the game", "a commander you own in
                // the command zone".
                return match owner {
                    Some(o) if o != PlayerRel::Any => format!("{} {z}", self.owns_phrase(o)),
                    _ => z.into(),
                };
            }
            _ => {}
        }
        match owner {
            None | Some(PlayerRel::Any) => match num {
                Num::One => format!("in a {zw}"),
                Num::Many => format!("in {}", plural(zw)),
            },
            Some(r) => {
                let p = self.rel_possessive(r, num);
                format!("in {p} {zw}")
            }
        }
    }

    /// "your", "an opponent's", "target player's".
    pub(crate) fn rel_possessive(&mut self, r: PlayerRel, num: Num) -> String {
        match r {
            PlayerRel::You => "your".into(),
            PlayerRel::Opponent => match num {
                Num::One => "an opponent's".into(),
                Num::Many => "your opponents'".into(),
            },
            PlayerRel::Any => "a player's".into(),
            PlayerRel::NotYou => "an opponent's".into(),
            // "target player's graveyard", then "that player's" / "their graveyard".
            PlayerRel::Target(i) => self.target_mention(i, Case::Poss),
            PlayerRel::TargetOrController(i) => {
                let p = self.target_player_mention(i);
                possessive(&p)
            }
            PlayerRel::TriggerPlayer | PlayerRel::Iterated => "{alt:that player's|their}".into(),
            PlayerRel::Defending => "defending player's".into(),
            PlayerRel::Active => "the active player's".into(),
            PlayerRel::Teammate => "a teammate's".into(),
            PlayerRel::Chosen => "the chosen player's".into(),
        }
    }

    /// "you", "an opponent", ... as an object ("attacking you").
    pub(crate) fn rel_object(&mut self, r: PlayerRel) -> String {
        match r {
            PlayerRel::You => "you".into(),
            PlayerRel::Opponent => "an opponent".into(),
            PlayerRel::Any => "a player".into(),
            PlayerRel::NotYou => "another player".into(),
            PlayerRel::Target(i) | PlayerRel::TargetOrController(i) => {
                self.target_player_mention(i)
            }
            // "the number of creatures attacking that player" / "... attacking them".
            PlayerRel::TriggerPlayer | PlayerRel::Iterated => "{alt:that player|them}".into(),
            PlayerRel::Defending => "defending player".into(),
            PlayerRel::Active => "the active player".into(),
            PlayerRel::Teammate => "a teammate".into(),
            PlayerRel::Chosen => "the chosen player".into(),
        }
    }

    /// "you", "an opponent", "each opponent" as a subject.
    pub(crate) fn rel_subject(&mut self, r: PlayerRel) -> String {
        match r {
            PlayerRel::Any => "a player".into(),
            PlayerRel::TriggerPlayer | PlayerRel::Iterated => "{alt:that player|they}".into(),
            other => self.rel_object(other),
        }
    }

    /// "you control", "an opponent controls", "your opponents control".
    pub(crate) fn controls_phrase(&mut self, r: PlayerRel, num: Num) -> String {
        let num = if self.each_mode { Num::Many } else { num };
        match r {
            PlayerRel::You => "you control".into(),
            // "a creature an opponent controls" / "each creature your opponents control":
            // the same objects, those any opponent controls.
            PlayerRel::Opponent => match num {
                Num::One => "{alt:an opponent controls|your opponents control}".into(),
                Num::Many => "your opponents control".into(),
            },
            PlayerRel::NotYou => "you don't control".into(),
            PlayerRel::Any => "a player controls".into(),
            PlayerRel::TargetOrController(i) => {
                let p = self.target_player_mention(i);
                // "each creature that player or that planeswalker's controller controls":
                // the target, mentioned again, is a player or a planeswalker.
                let p = if p == "it" {
                    "that player".to_string()
                } else {
                    p
                };
                format!("{p} or that planeswalker's controller controls")
            }
            // "among creatures they control": the player each player is.
            PlayerRel::Iterated => "{alt:that player controls|they control}".into(),
            other => {
                let p = self.rel_subject(other);
                format!("{p} controls")
            }
        }
    }

    pub(crate) fn owns_phrase(&mut self, r: PlayerRel) -> String {
        match r {
            PlayerRel::You => "you own".into(),
            PlayerRel::NotYou => "you don't own".into(),
            // The same cards: those any opponent owns.
            PlayerRel::Opponent => "{alt:an opponent owns|your opponents own}".into(),
            other => {
                let p = self.rel_object(other);
                format!("{p} owns")
            }
        }
    }

    /// A determiner + noun phrase.
    pub(crate) fn noun_det(&mut self, f: &Filter, det: Det) -> String {
        if let Filter::Custom(n) = f {
            if let Some(s) = self.custom_noun(n) {
                return s;
            }
        }
        // "If that creature would die this turn": the selection itself.
        if let Filter::In(sel) = f {
            if !matches!(
                sel.as_ref(),
                Sel::Var(crate::kw::reveal_from_hand::REVEALED)
            ) && self.this_way_of(sel).is_none()
            {
                return self.sel(sel, Case::Obj);
            }
        }
        // "the top card of target player's graveyard".
        if let Some(s) = self.custom_whole_noun(f) {
            return s;
        }
        // "target creature you control with the greatest power".
        if let Some(s) = self.extreme_noun(f, &det) {
            return s;
        }
        // A complex union inside a conjunction: "basic land card or Gate card in your
        // library" is "basic land card in your library or Gate card in your library".
        let f = &flatten_or(&distribute_or(f));
        // A complex union: each alternative gets the determiner ("~ or another creature");
        // a count applies to all of them ("up to two basic land cards and/or Gate cards").
        if let Filter::Or(v) = f {
            // "a card named Festering Newt or Bubbling Cauldron".
            let named = |x: &Filter| -> Option<String> {
                match x {
                    Filter::Named(n) => Some(n.to_string()),
                    Filter::And(p) => {
                        let names: Vec<String> = p
                            .iter()
                            .filter_map(|q| match q {
                                Filter::Named(n) => Some(n.to_string()),
                                _ => None,
                            })
                            .collect();
                        let others_plain = p.iter().all(|q| {
                            matches!(q, Filter::Named(_) | Filter::Card | Filter::InZone(_))
                        });
                        (names.len() == 1 && others_plain).then(|| names[0].clone())
                    }
                    _ => None,
                }
            };
            let names: Option<Vec<String>> = v.iter().map(named).collect();
            if let Some(names) = names.filter(|n| n.len() > 1) {
                let first = self.noun_det(&v[0], det.clone());
                if let Some(head) = first.strip_suffix(names[0].as_str()) {
                    return format!("{head}{}", join_list(&names, "or"));
                }
            }
            // "creatures with flying or reach": alternatives that differ only in a keyword.
            if let Some((shared, kws)) = keyword_alternatives(v) {
                let n = self.noun_det(&shared, det.clone());
                let k: Vec<String> = kws.iter().map(|k| self.keyword_kind_word(*k)).collect();
                return format!("{n} with {}", join_list(&k, "or"));
            }
            if !v.iter().all(Self::is_type_like) && !v.iter().all(|x| matches!(x, Filter::Color(_)))
            {
                // "enchanted creature or enchantment creature you control": a quality all
                // the alternatives share, said once after them.
                if let Some(s) = self.shared_tail_union(v, det.clone()) {
                    return s;
                }
                return match det.num() {
                    Num::One => {
                        let parts: Vec<String> =
                            v.iter().map(|x| self.noun_det(x, det.clone())).collect();
                        join_list(&parts, "or")
                    }
                    Num::Many => {
                        let parts: Vec<String> =
                            v.iter().map(|x| self.noun_det(x, Det::Plural)).collect();
                        let list = join_list(&parts, "and/or");
                        match det {
                            Det::Count(n) => format!("{n} {list}"),
                            Det::UpTo(n) => format!("up to {n} {list}"),
                            Det::OneOrMore => format!("one or more {list}"),
                            _ => list,
                        }
                    }
                };
            }
        }
        let mut np = Np::default();
        self.collect(f, &mut np);
        if np.is_self {
            return self.me();
        }
        // "enchanted creature" is definite: no determiner.
        if np.fixed.is_some() && !np.other {
            let n = det.num();
            return self.np_text(&np, n, false);
        }
        let num = det.num();
        let other = np.other;
        let saved = (self.each_mode, self.alt_and);
        self.each_mode = matches!(det, Det::Each);
        if matches!(det, Det::Each | Det::Plural) {
            self.alt_and = true;
        }
        let text = self.np_text(&np, num, other);
        (self.each_mode, self.alt_and) = saved;
        match det {
            Det::A => {
                if other {
                    format!("another {text}")
                } else {
                    with_article(&text)
                }
            }
            Det::Each => {
                if other {
                    format!("each other {text}")
                } else {
                    format!("each {text}")
                }
            }
            Det::Plural => {
                if other {
                    format!("other {text}")
                } else {
                    text
                }
            }
            Det::Bare => {
                if other {
                    format!("other {text}")
                } else {
                    text
                }
            }
            Det::Count(n) => {
                if n == "a" || n == "one" {
                    if other {
                        format!("another {text}")
                    } else {
                        with_article(&text)
                    }
                } else if other {
                    format!("{n} other {text}")
                } else {
                    format!("{n} {text}")
                }
            }
            Det::UpTo(n) => {
                if other {
                    format!("up to {n} other {text}")
                } else {
                    format!("up to {n} {text}")
                }
            }
            Det::OneOrMore => {
                if other {
                    format!("one or more other {text}")
                } else {
                    format!("one or more {text}")
                }
            }
        }
    }

    /// The noun after "for each": "for each instant and sorcery card in your graveyard".
    pub(crate) fn for_each_noun(&mut self, f: &Filter) -> String {
        let saved = self.alt_and;
        self.alt_and = true;
        let n = self.noun(f, Num::One);
        self.alt_and = saved;
        if Self::counts_all_permanents(f, &n) {
            format!("{n} {{opt:on the battlefield}}")
        } else {
            n
        }
    }

    /// A determiner for a count value ("a", "two", "X").
    pub(crate) fn det_for(&mut self, v: &Value) -> Det {
        match v {
            Value::Const(1) => Det::A,
            Value::Const(n) => Det::Count(number_word(*n)),
            other => {
                let s = self.value(other);
                Det::Count(s)
            }
        }
    }

    pub(crate) fn targets_filter(&mut self, tf: &TargetsFilter) -> String {
        match tf {
            TargetsFilter::Count(1) => "with a single target".into(),
            TargetsFilter::Count(n) => format!("with {} targets", number_word(*n as i32)),
            TargetsFilter::Targets { objects, players } => {
                let mut parts = Vec::new();
                if let Some(o) = objects {
                    parts.push(self.noun_det(o, Det::A));
                }
                if let Some(p) = players {
                    parts.push(self.player_filter_object(p));
                }
                format!("that targets {}", join_list(&parts, "or"))
            }
            TargetsFilter::Only { objects, players } => {
                let mut parts = Vec::new();
                if let Some(o) = objects {
                    parts.push(self.noun_det(o, Det::A));
                }
                if let Some(p) = players {
                    parts.push(self.player_filter_object(p));
                }
                format!("that targets only {}", join_list(&parts, "or"))
            }
        }
    }
}

/// `And([Or([a, b]), c])` → `Or([And([a, c]), And([b, c])])` when the union isn't a
/// simple list of types or colors.
/// "artifact, enchantment, or tapped creature": an alternative between alternatives is
/// one list.
fn flatten_or(f: &Filter) -> Filter {
    match f {
        Filter::Or(v) if v.iter().any(|x| matches!(x, Filter::Or(_))) => {
            let mut out = Vec::new();
            for x in v {
                match flatten_or(x) {
                    Filter::Or(w) => out.extend(w),
                    y => out.push(y),
                }
            }
            Filter::Or(out)
        }
        other => other.clone(),
    }
}

/// `[A and has K1, A and has K2, ...]`: (A, [K1, K2, ...]).
fn keyword_alternatives(v: &[Filter]) -> Option<(Filter, Vec<crate::keywords::KeywordKind>)> {
    if v.len() < 2 {
        return None;
    }
    let mut shared: Option<Vec<Filter>> = None;
    let mut kws = Vec::new();
    for x in v {
        let Filter::And(parts) = x else { return None };
        let (k, rest): (Vec<&Filter>, Vec<&Filter>) = parts
            .iter()
            .partition(|p| matches!(p, Filter::HasKeyword(_)));
        let [Filter::HasKeyword(k)] = k.as_slice() else {
            return None;
        };
        kws.push(*k);
        let rest: Vec<Filter> = rest.into_iter().cloned().collect();
        match &shared {
            None => shared = Some(rest),
            Some(s) if format!("{s:?}") == format!("{rest:?}") => {}
            Some(_) => return None,
        }
    }
    Some((Filter::and(shared?), kws))
}

/// The creature types an outlaw has one of (CR 700.12).
fn is_basic_land_types(v: &[Filter]) -> bool {
    let mut names: Vec<&str> = v
        .iter()
        .filter_map(|x| match x {
            Filter::Subtype(s) => Some(s.as_ref()),
            _ => None,
        })
        .collect();
    names.sort();
    v.len() == 5 && names == ["Forest", "Island", "Mountain", "Plains", "Swamp"]
}

fn is_outlaw(v: &[Filter]) -> bool {
    let mut names: Vec<&str> = v
        .iter()
        .filter_map(|x| match x {
            Filter::Subtype(s) => Some(s.as_ref()),
            _ => None,
        })
        .collect();
    names.sort();
    v.len() == 5 && names == ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
}

fn distribute_or(f: &Filter) -> Filter {
    let Filter::And(v) = f else {
        return f.clone();
    };
    let complex = |x: &Filter| {
        matches!(x, Filter::Or(alts) if !alts.iter().all(Renderer::is_type_like)
            && !alts.iter().all(|a| matches!(a, Filter::Color(_)))
            && !alts.iter().all(|a| matches!(a, Filter::Named(_)))
            && !alts.iter().all(|a| matches!(a, Filter::Attacking | Filter::Blocking | Filter::Tapped | Filter::Untapped | Filter::Blocked | Filter::Unblocked)))
    };
    let Some(pos) = v.iter().position(complex) else {
        return f.clone();
    };
    let Filter::Or(alts) = &v[pos] else {
        return f.clone();
    };
    let rest: Vec<Filter> = v
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != pos)
        .map(|(_, x)| x.clone())
        .collect();
    Filter::Or(
        alts.iter()
            .map(|a| {
                let mut parts = vec![a.clone()];
                parts.extend(rest.iter().cloned());
                Filter::and(parts)
            })
            .collect(),
    )
}

fn is_combat_keyword(w: &str) -> bool {
    matches!(
        w,
        "flying"
            | "reach"
            | "first strike"
            | "double strike"
            | "trample"
            | "deathtouch"
            | "lifelink"
            | "vigilance"
            | "menace"
            | "defender"
            | "haste"
            | "shadow"
            | "horsemanship"
            | "fear"
            | "intimidate"
            | "skulk"
            | "flanking"
            | "banding"
    )
}

/// "X's" / "X'" possessive.
pub fn possessive(s: &str) -> String {
    match s {
        "you" => "your".into(),
        "it" => "its".into(),
        "they" | "them" => "their".into(),
        _ if s.ends_with('s') && !s.ends_with("ss") && s != "~" => format!("{s}'"),
        _ => format!("{s}'s"),
    }
}

/// The quality a "shares a ... with" filter is about.
fn other_kind(f: &Filter) -> Option<&'static str> {
    match f {
        Filter::SharesColor(_) => Some("a color"),
        Filter::SharesCardType(_) => Some("a card type"),
        Filter::SharesCreatureType(_) => Some("a creature type"),
        _ => None,
    }
}
