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

pub(crate) fn cmp_phrase(cmp: Cmp, v: &str) -> String {
    match cmp {
        // "with mana value 3" / "with mana value equal to the number of ...".
        Cmp::Eq if v.parse::<i64>().is_ok() || v == "X" => v.to_string(),
        Cmp::Eq => format!("{{opt:equal to}} {v}"),
        Cmp::Ne => format!("other than {v}"),
        Cmp::Lt => format!("less than {v}"),
        Cmp::Le => format!("{v} or less"),
        Cmp::Gt => format!("greater than {v}"),
        Cmp::Ge => format!("{v} or greater"),
    }
}

impl Renderer<'_> {
    /// The noun used for "enchanted [thing]" / "equipped creature" on this face.
    pub(crate) fn attached_noun(&mut self) -> String {
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
                for x in v {
                    self.collect(x, np);
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
                let v = self.value(v);
                np.with.push(format!("power {}", cmp_phrase(*c, &v)));
            }
            Filter::Toughness(c, v) => {
                let v = self.value(v);
                np.with.push(format!("toughness {}", cmp_phrase(*c, &v)));
            }
            Filter::PowerVsBase(c) => {
                np.with
                    .push(format!("power {}", cmp_phrase(*c, "its base power")));
            }
            Filter::ManaValue(c, v) => {
                let v = self.value(v);
                np.with.push(format!("mana value {}", cmp_phrase(*c, &v)));
            }
            Filter::Loyalty(c, v) => {
                let v = self.value(v);
                np.with.push(format!("loyalty {}", cmp_phrase(*c, &v)));
            }
            Filter::Named(n) => np.post.push(format!("named {n}")),
            // "with the same name as a card exiled with ~": as any of them.
            Filter::SameNameAs(s) => {
                let s = self.sel(s, Case::Obj);
                let s = s.replace("|each card exiled with ~}", "|a card exiled with ~}");
                np.with.push(format!("the same name as {s}"));
            }
            Filter::DifferentNameFrom(s) => {
                let s = self.sel(s, Case::Obj);
                np.with.push(format!("a different name than {s}"));
            }
            Filter::NameOriginallyPrintedIn(set) => np
                .with
                .push(format!("a name originally printed in the {set} expansion")),
            Filter::SharesCreatureType(s) => {
                let s = self.sel(s, Case::Obj);
                np.rel.push(format!("that shares a creature type with {s}"));
            }
            Filter::SharesCardType(s) => {
                let s = self.sel(s, Case::Obj);
                np.rel.push(format!("that shares a card type with {s}"));
            }
            Filter::SharesColor(s) => {
                let s = self.sel(s, Case::Obj);
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
            Filter::ChosenColor | Filter::LinkedChosenColor => {
                np.post.push("of the chosen color".into())
            }
            Filter::ChosenType | Filter::LinkedChosenCreatureType => {
                np.post.push("of {alt:the chosen type|that type}".into())
            }
            Filter::ChosenName => np.with.push("the chosen name".into()),
            Filter::ChosenCardType => np.post.push("of the chosen card type".into()),
            Filter::Prepared => np.status.push("prepared".into()),
            Filter::Targets(f) => {
                let t = self.noun_det(f, Det::A);
                np.rel.push(format!("that targets {t}"));
            }
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
            Filter::CastFrom(z) => {
                let z = zone_word(*z);
                np.rel.push(format!("cast from a {z}"));
            }
            Filter::CastWithCost(name) => np.status.push(name.to_string()),
            Filter::DealtDamageThisTurnBy(s) => {
                let s = self.sel(s, Case::Obj);
                np.rel.push(format!("dealt damage by {s} this turn"));
            }
            Filter::ManaValueOfChosenQuality => {
                np.with.push("mana value of the chosen quality".into())
            }
            Filter::AbilityFrom(f) => {
                let s = self.noun(f, Num::One);
                np.post.push(format!("from {} source", article(&s)));
                np.post.push(s);
            }
            Filter::Custom(name) => {
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
        words.extend(types.iter().map(|t| t.word().to_string()));
        if !np.alts.is_empty() {
            let conj = if self.alt_and { "and" } else { "or" };
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
                return list;
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
            for p in &np.post {
                s.push(' ');
                s.push_str(p);
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
        words.extend(np.colors.iter().cloned());
        words.extend(np.quality.iter().cloned());
        if !np.nons.is_empty() {
            let nons = np.nons.join(", ");
            // "nonland permanent card": the non- word goes before "permanent".
            match words.iter().position(|w| w == "permanent") {
                Some(i) => words.insert(i, nons),
                None => words.push(nons),
            }
        }
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
        match (np.zone, np.owner) {
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
            ZoneKind::Command => return "in the command zone".into(),
            ZoneKind::Outside => return "from outside the game".into(),
            ZoneKind::Stack => return "on the stack".into(),
            ZoneKind::Battlefield => return "on the battlefield".into(),
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
            PlayerRel::Opponent => match num {
                Num::One => "an opponent controls".into(),
                Num::Many => "your opponents control".into(),
            },
            PlayerRel::NotYou => "you don't control".into(),
            PlayerRel::Any => "a player controls".into(),
            PlayerRel::TargetOrController(i) => {
                let p = self.target_player_mention(i);
                format!("{p} or that planeswalker's controller controls")
            }
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
            PlayerRel::Opponent => "an opponent owns".into(),
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
            return self.sel(sel, Case::Obj);
        }
        // A complex union inside a conjunction: "basic land card or Gate card in your
        // library" is "basic land card in your library or Gate card in your library".
        let f = &distribute_or(f);
        // A complex union: each alternative gets the determiner ("~ or another creature");
        // a count applies to all of them ("up to two basic land cards and/or Gate cards").
        if let Filter::Or(v) = f {
            if !v.iter().all(Self::is_type_like) && !v.iter().all(|x| matches!(x, Filter::Color(_)))
            {
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
fn distribute_or(f: &Filter) -> Filter {
    let Filter::And(v) = f else {
        return f.clone();
    };
    let complex = |x: &Filter| {
        matches!(x, Filter::Or(alts) if !alts.iter().all(Renderer::is_type_like)
            && !alts.iter().all(|a| matches!(a, Filter::Color(_)))
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
