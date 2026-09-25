//! Evaluation of the ability language: [`Filter`], [`PlayerFilter`], [`Value`],
//! [`Condition`], [`Sel`], and [`PlayerRef`] against the game state, relative to a
//! [`Ctx`] (the source, its controller, chosen targets, variables, trigger event).

use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Context for evaluating and resolving abilities.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ctx {
    /// The object whose ability this is (or the spell itself).
    pub source: Option<ObjectId>,
    pub controller: PlayerId,
    /// The resolving stack object, if any.
    pub stack_obj: Option<ObjectId>,
    /// Target slots of the current mode (legal targets only during resolution).
    pub targets: Vec<Vec<Entity>>,
    pub divided: Vec<Vec<u32>>,
    pub x: i32,
    /// X was defined by the resolving ability's text ([`Effect::SetX`], e.g. "ward {X},
    /// where X is ...", CR 702.21b), so no player chooses it when a cost with X is paid.
    #[serde(default)]
    pub x_defined: bool,
    pub vars: BTreeMap<Var, Vec<Entity>>,
    pub nums: BTreeMap<Var, i64>,
    pub event: Option<EventInfo>,
    /// Current player in a "for each player" loop.
    pub iter_player: Option<PlayerId>,
    pub prev_happened: bool,
    pub prev_value: i64,
    pub prev_affected: Vec<Entity>,
    pub cast: Option<CastInfo>,
    pub ability_uid: u64,
    pub link: u16,
    /// Source characteristics as last known (for abilities whose source left).
    pub source_lki: Option<Box<Characteristics>>,
    /// Chosen opponent ("choose an opponent").
    pub chosen_player: Option<PlayerId>,
    /// Set while an "as this enters" replacement effect is being applied: modifications
    /// to how the permanent enters made by [`Effect::EnterTapped`] and
    /// [`Effect::EnterWithCounters`] (CR 614.1c, 614.12).
    pub entering: Option<EntryMods>,
}

/// Modifications to how a permanent enters, collected while applying an "as this
/// enters" replacement effect (CR 614.1c).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntryMods {
    pub tapped: bool,
    pub counters: Vec<(CounterKind, u32)>,
    /// Enters prepared (CR 722.3a).
    pub prepared: bool,
    /// Exceptions to a copy effect it enters with (CR 707.9b).
    pub copy_exceptions: Vec<Modification>,
    /// Exceptions to a copy effect that are additional effects, conditional, or linked
    /// triggered abilities (CR 707.9e–707.9g).
    #[serde(default)]
    pub copy_extras: Vec<crate::copy_rules::CopyExtra>,
    /// Effects on the permanent performed as it's put onto the battlefield
    /// ([`Effect::OnEntry`]).
    pub on_entry: Vec<Effect>,
    /// Modifications to its copiable values ([`Effect::EnterAs`], CR 707.2).
    pub copiable: Vec<Modification>,
}

impl Ctx {
    pub fn new(source: Option<ObjectId>, controller: PlayerId) -> Ctx {
        Ctx {
            source,
            controller,
            ..Default::default()
        }
    }
    pub fn for_object(g: &Game, id: ObjectId) -> Ctx {
        Ctx::new(Some(id), g.obj(id).controller)
    }
    pub fn set_var(&mut self, v: Var, e: Vec<Entity>) {
        self.vars.insert(v, e);
    }
    pub fn var_objects(&self, v: Var) -> Vec<ObjectId> {
        self.vars
            .get(&v)
            .map(|e| e.iter().filter_map(|x| x.object()).collect())
            .unwrap_or_default()
    }
}

/// Read access to (possibly interim) characteristics — lets filters be evaluated both
/// against final characteristics and against partially-applied layer state.
pub trait View {
    fn chars<'a>(&'a self, g: &'a Game, id: ObjectId) -> &'a Characteristics;
    fn controller(&self, g: &Game, id: ObjectId) -> PlayerId;
    /// The controller an object is treated as having regardless of its zone (an object
    /// about to enter the battlefield, CR 614.12).
    fn controller_override(&self, _id: ObjectId) -> Option<PlayerId> {
        None
    }
}

/// The normal view: an object's computed characteristics.
pub struct Current;

impl View for Current {
    fn chars<'a>(&'a self, g: &'a Game, id: ObjectId) -> &'a Characteristics {
        &g.obj(id).chars
    }
    fn controller(&self, g: &Game, id: ObjectId) -> PlayerId {
        g.obj(id).controller
    }
}

impl Game {
    // ------------------------------------------------------------------
    // Players
    // ------------------------------------------------------------------

    pub fn player_rel_matches(&self, rel: PlayerRel, p: PlayerId, ctx: &Ctx) -> bool {
        match rel {
            PlayerRel::You => p == ctx.controller,
            PlayerRel::Opponent => self.are_opponents(ctx.controller, p),
            PlayerRel::Any => true,
            PlayerRel::NotYou => p != ctx.controller,
            PlayerRel::Target(slot) => ctx
                .targets
                .get(slot as usize)
                .is_some_and(|v| v.contains(&Entity::Player(p))),
            PlayerRel::TargetOrController(slot) => {
                ctx.targets.get(slot as usize).is_some_and(|v| {
                    v.iter().any(|e| match e {
                        Entity::Player(q) => *q == p,
                        Entity::Object(o) => self.obj(*o).controller == p,
                    })
                })
            }
            PlayerRel::TriggerPlayer => ctx.event.as_ref().and_then(|e| e.player) == Some(p),
            PlayerRel::Defending => self.defending_player_for(ctx) == Some(p),
            PlayerRel::Active => self.turn.active == p,
            PlayerRel::Teammate => {
                p != ctx.controller && self.player(p).team == self.player(ctx.controller).team
            }
            PlayerRel::Iterated => ctx.iter_player == Some(p),
            PlayerRel::Chosen => self.chosen_player_of_source(ctx) == Some(p),
        }
    }

    /// Choices made for the ability's source ("the chosen color", CR 607.2d): those made
    /// by the abilities linked to the one being evaluated. An ability with an explicit
    /// link (including abilities acquired from another object or through a copy effect)
    /// sees only its linked choices (CR 607.5a); an unlinked ability (link 0) whose
    /// linked abilities made no choice sees the choices made by any of the source's
    /// abilities.
    pub fn source_choices(&self, ctx: &Ctx) -> Option<&Choices> {
        match self.linked_choice(ctx) {
            Some(c) => Some(c),
            None if ctx.link == 0 => ctx.source.map(|s| &self.obj(s).choices),
            None => None,
        }
    }

    /// The player chosen for the source ("choose an opponent"), or the one chosen during
    /// the current resolution.
    pub fn chosen_player_of_source(&self, ctx: &Ctx) -> Option<PlayerId> {
        ctx.chosen_player
            .or_else(|| self.source_choices(ctx).and_then(|c| c.player))
    }

    /// Defending player relative to the source (CR 508.5).
    pub fn defending_player_for(&self, ctx: &Ctx) -> Option<PlayerId> {
        let combat = self.combat.as_ref()?;
        if let Some(src) = ctx.source {
            if let Some(p) = combat.defending_player_of(self, src) {
                return Some(p);
            }
        }
        combat.defending_players.first().copied()
    }

    pub fn player_filter_matches(&self, f: &PlayerFilter, p: PlayerId, ctx: &Ctx) -> bool {
        match f {
            PlayerFilter::Any => true,
            PlayerFilter::Is(q) => p == *q,
            PlayerFilter::You | PlayerFilter::Controller => p == ctx.controller,
            PlayerFilter::Opponent => self.are_opponents(ctx.controller, p),
            PlayerFilter::NotYou => p != ctx.controller,
            PlayerFilter::DealtDamageThisTurn => self
                .history
                .damage_dealt_to_players
                .get(&p)
                .is_some_and(|d| *d > 0),
            PlayerFilter::Life(cmp, v) => {
                cmp.eval(self.player(p).life as i64, self.eval_value(v, ctx))
            }
            PlayerFilter::HandSize(cmp, v) => {
                cmp.eval(self.player(p).hand.len() as i64, self.eval_value(v, ctx))
            }
            // Only cards count: a token in a graveyard isn't a card (CR 108.2b).
            PlayerFilter::GraveyardSize(cmp, v) => cmp.eval(
                self.player(p)
                    .graveyard
                    .iter()
                    .filter(|o| self.obj(**o).is_card())
                    .count() as i64,
                self.eval_value(v, ctx),
            ),
            PlayerFilter::Monarch => self.monarch == Some(p),
            PlayerFilter::Controls(f, cmp, v) => {
                let n = self
                    .permanents()
                    .filter(|o| o.controller == p && self.matches(o.id, f, ctx))
                    .count();
                cmp.eval(n as i64, self.eval_value(v, ctx))
            }
            PlayerFilter::Counters(k, cmp, v) => {
                cmp.eval(self.player(p).counter(k) as i64, self.eval_value(v, ctx))
            }
            PlayerFilter::Defending => self.defending_player_for(ctx) == Some(p),
            PlayerFilter::Active => self.turn.active == p,
            PlayerFilter::Poisoned => self.player(p).poison() > 0,
            PlayerFilter::Ref(r) => self.eval_players(r, ctx).contains(&p),
            PlayerFilter::And(v) => v.iter().all(|x| self.player_filter_matches(x, p, ctx)),
            PlayerFilter::Or(v) => v.iter().any(|x| self.player_filter_matches(x, p, ctx)),
            PlayerFilter::Not(x) => !self.player_filter_matches(x, p, ctx),
        }
    }

    /// Resolves a [`PlayerRef`] to players (in APNAP order where there are several).
    pub fn eval_players(&self, r: &PlayerRef, ctx: &Ctx) -> Vec<PlayerId> {
        let apnap = self.apnap();
        match r {
            PlayerRef::Player(p) => {
                if self.player(*p).in_game() {
                    vec![*p]
                } else {
                    vec![]
                }
            }
            PlayerRef::You => vec![ctx.controller],
            PlayerRef::EachOpponent => apnap
                .into_iter()
                .filter(|p| self.are_opponents(ctx.controller, *p))
                .collect(),
            PlayerRef::EachPlayer => apnap,
            PlayerRef::EachOtherPlayer => {
                apnap.into_iter().filter(|p| *p != ctx.controller).collect()
            }
            PlayerRef::Target(slot) => ctx
                .targets
                .get(*slot as usize)
                .map(|v| v.iter().filter_map(|e| e.player()).collect())
                .unwrap_or_default(),
            PlayerRef::ControllerOf(sel) => {
                let mut v: Vec<PlayerId> = self
                    .eval_sel(sel, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        Entity::Object(o) => Some(self.obj(o).controller),
                        Entity::Player(p) => Some(p),
                    })
                    .collect();
                v.dedup();
                v
            }
            PlayerRef::OwnerOf(sel) => {
                let mut v: Vec<PlayerId> = self
                    .eval_sel(sel, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        Entity::Object(o) => Some(self.obj(o).owner),
                        Entity::Player(p) => Some(p),
                    })
                    .collect();
                v.dedup();
                v
            }
            PlayerRef::TriggerPlayer => ctx
                .event
                .as_ref()
                .and_then(|e| e.player)
                .into_iter()
                .collect(),
            PlayerRef::ActivePlayer => vec![self.turn.active],
            PlayerRef::DefendingPlayer => self.defending_player_for(ctx).into_iter().collect(),
            PlayerRef::ChosenPlayer(v) | PlayerRef::Var(v) => ctx
                .vars
                .get(v)
                .map(|e| e.iter().filter_map(|x| x.player()).collect())
                .unwrap_or_default(),
            PlayerRef::Each(f) => apnap
                .into_iter()
                .filter(|p| self.player_filter_matches(f, *p, ctx))
                .collect(),
            PlayerRef::Owner => ctx.source.map(|s| self.obj(s).owner).into_iter().collect(),
            PlayerRef::Iterated => ctx.iter_player.into_iter().collect(),
            PlayerRef::ChosenOpponent => self
                .chosen_player_of_source(ctx)
                .filter(|p| self.player(*p).in_game())
                .into_iter()
                .collect(),
            PlayerRef::Monarch => self.monarch.into_iter().collect(),
        }
    }

    pub fn eval_player(&self, r: &PlayerRef, ctx: &Ctx) -> Option<PlayerId> {
        self.eval_players(r, ctx).into_iter().next()
    }

    // ------------------------------------------------------------------
    // Objects
    // ------------------------------------------------------------------

    /// Whether object `id` matches `f`, using current characteristics.
    pub fn matches(&self, id: ObjectId, f: &Filter, ctx: &Ctx) -> bool {
        self.matches_view(&Current, id, f, ctx)
    }

    /// Controller for filter purposes: objects outside the battlefield and stack have no
    /// controller (CR 108.4a); we treat their owner as controller for "you control" checks
    /// only where the filter is about a zone the player owns.
    fn filter_controller(&self, view: &dyn View, id: ObjectId) -> PlayerId {
        if let Some(p) = view.controller_override(id) {
            return p;
        }
        let o = self.obj(id);
        match o.zone {
            // Objects in the command zone have controllers too (CR 109.4c–g).
            Zone::Battlefield | Zone::Stack | Zone::Command => view.controller(self, id),
            _ => o.owner,
        }
    }

    pub fn matches_view(&self, view: &dyn View, id: ObjectId, f: &Filter, ctx: &Ctx) -> bool {
        let o = self.obj(id);
        let c = view.chars(self, id);
        match f {
            Filter::Any => true,
            Filter::And(v) => v.iter().all(|x| self.matches_view(view, id, x, ctx)),
            Filter::Or(v) => v.iter().any(|x| self.matches_view(view, id, x, ctx)),
            Filter::Not(x) => !self.matches_view(view, id, x, ctx),
            Filter::Type(t) => c.card_types.contains(*t),
            Filter::Supertype(s) => c.supertypes.contains(*s),
            Filter::Subtype(s) => {
                c.has_subtype(s)
                    || (is_creature_type(s)
                        && (c.card_types.contains(CardType::Creature)
                            || c.card_types.contains(CardType::Kindred))
                        && c.has_keyword(KeywordKind::Changeling))
            }
            Filter::Color(col) => c.colors.contains(*col),
            Filter::ExactColors(cs) => c.colors == *cs,
            Filter::Colorless => c.colors.is_colorless(),
            Filter::Multicolored => c.colors.is_multicolored(),
            Filter::Monocolored => c.colors.is_monocolored(),
            Filter::Permanent => o.zone == Zone::Battlefield,
            Filter::PermanentCard => c.card_types.has_permanent_type(),
            Filter::Spell => o.is_spell(),
            Filter::Token => o.kind == ObjKind::Token,
            Filter::Card => o.kind == ObjKind::Card,
            Filter::Copy => matches!(o.kind, ObjKind::CardCopy | ObjKind::SpellCopy),
            Filter::ControlledBy(rel) => {
                self.player_rel_matches(*rel, self.filter_controller(view, id), ctx)
            }
            Filter::OwnedBy(rel) => self.player_rel_matches(*rel, o.owner, ctx),
            Filter::InZone(z) => o.zone.kind() == Some(*z),
            // Only permanents have status (CR 110.5d).
            Filter::Tapped => o.zone == Zone::Battlefield && o.tapped,
            Filter::Untapped => o.zone == Zone::Battlefield && !o.tapped,
            Filter::Attacking => self.is_attacking(id),
            Filter::Blocking => self.is_blocking(id),
            Filter::Blocked => self.combat.as_ref().is_some_and(|cb| cb.is_blocked(id)),
            // CR 509.1h: attackers are neither blocked nor unblocked until blockers are declared.
            Filter::Unblocked => self.combat.as_ref().is_some_and(|cb| cb.is_unblocked(id)),
            Filter::AttackingAlone
            | Filter::BlockingAlone
            | Filter::AttackingPlayerAlone
            | Filter::HadToAttack => crate::combat::combat_filter(self, f, id),
            Filter::AttackingPlayer(rel) => self
                .combat
                .as_ref()
                .and_then(|cb| cb.attack_target(id))
                .is_some_and(|t| match t {
                    Entity::Player(p) => self.player_rel_matches(*rel, p, ctx),
                    Entity::Object(pw) => {
                        self.player_rel_matches(*rel, self.obj(pw).controller, ctx)
                    }
                }),
            Filter::BlockingSource => ctx.source.is_some_and(|s| {
                self.combat
                    .as_ref()
                    .is_some_and(|cb| cb.blockers_of(s).contains(&id))
            }),
            Filter::BlockedBySource => ctx.source.is_some_and(|s| {
                self.combat
                    .as_ref()
                    .is_some_and(|cb| cb.blockers_of(id).contains(&s))
            }),
            Filter::Power(cmp, v) => c
                .power
                .is_some_and(|p| cmp.eval(p as i64, self.eval_value(v, ctx))),
            Filter::Toughness(cmp, v) => c
                .toughness
                .is_some_and(|t| cmp.eval(t as i64, self.eval_value(v, ctx))),
            Filter::ManaValue(cmp, v) => {
                // Uses the viewed characteristics' mana cost (CR 601.3e), with X on the
                // stack (CR 107.3f) as in `mana_value_of`.
                let mv = match &c.mana_cost {
                    None => 0,
                    Some(mc) if o.zone == Zone::Stack => {
                        let x = o.stack.as_ref().and_then(|s| s.x).unwrap_or(0).max(0);
                        mc.mana_value_with_x(x as u32)
                    }
                    Some(mc) => mc.mana_value(),
                };
                cmp.eval(mv as i64, self.eval_value(v, ctx))
            }
            Filter::Loyalty(cmp, v) => cmp.eval(o.loyalty() as i64, self.eval_value(v, ctx)),
            Filter::Named(n) => c.has_name(n),
            Filter::SameNameAs(sel) => self
                .eval_sel(sel, ctx)
                .iter()
                .filter_map(|e| e.object())
                .any(|x| c.shares_name_with(&self.obj(x).chars)),
            Filter::SharesCreatureType(sel) => self
                .eval_sel(sel, ctx)
                .iter()
                .filter_map(|e| e.object())
                .any(|x| {
                    let other = &self.obj(x).chars;
                    c.subtypes
                        .iter()
                        .any(|s| is_creature_type(s) && other.has_subtype(s))
                        || (c.has_keyword(KeywordKind::Changeling)
                            && other.subtypes.iter().any(|s| is_creature_type(s)))
                        || (other.has_keyword(KeywordKind::Changeling)
                            && c.subtypes.iter().any(|s| is_creature_type(s)))
                }),
            Filter::SharesCardType(sel) => self
                .eval_sel(sel, ctx)
                .iter()
                .filter_map(|e| e.object())
                .any(|x| self.obj(x).chars.card_types.intersects(c.card_types)),
            Filter::SharesColor(sel) => self
                .eval_sel(sel, ctx)
                .iter()
                .filter_map(|e| e.object())
                .any(|x| self.obj(x).chars.colors.intersects(c.colors)),
            Filter::HasKeyword(k) => c.has_keyword(*k),
            Filter::HasCounter(k) => match k {
                Some(k) => o.counter(k) > 0,
                None => o.counters.values().any(|n| *n > 0),
            },
            Filter::HasAbilities => !c.has_no_abilities(),
            Filter::Source => ctx.source == Some(id),
            Filter::Other => ctx.source != Some(id),
            Filter::In(sel) => self.eval_sel(sel, ctx).contains(&Entity::Object(id)),
            // CR 609.7a: a chosen permanent spell is also the permanent it becomes.
            Filter::Objects(v) => v.iter().any(|x| {
                *x == id
                    || (self.obj(*x).zone == Zone::Stack
                        && o.zone == Zone::Battlefield
                        && self.current(*x) == id)
            }),
            Filter::AttachedToSource => {
                ctx.source.and_then(|s| self.obj(s).attached_to) == Some(Entity::Object(id))
            }
            Filter::Attached => o.attached_to.is_some(),
            Filter::Enchanted => self
                .attachments_of(Entity::Object(id))
                .iter()
                .any(|a| self.obj(*a).chars.has_subtype("Aura")),
            Filter::Equipped => self
                .attachments_of(Entity::Object(id))
                .iter()
                .any(|a| self.obj(*a).chars.has_subtype("Equipment")),
            Filter::EnteredThisTurn => {
                o.zone == Zone::Battlefield && o.entered_turn == self.turn.number
            }
            Filter::DealtDamageThisTurn => self.history.objects_dealt_damage.contains(&id),
            Filter::Historic => {
                c.card_types.contains(CardType::Artifact)
                    || c.is_legendary()
                    || c.has_subtype("Saga")
            }
            Filter::FaceDown => o.face_down,
            Filter::HasX => c.mana_cost.as_ref().is_some_and(|m| m.has_x()),
            Filter::HasPhyrexianMana => c
                .mana_cost
                .as_ref()
                .is_some_and(|m| m.symbols.iter().any(|s| s.is_phyrexian())),
            Filter::Commander => o.is_commander,
            Filter::Modified => {
                o.counters.values().any(|n| *n > 0)
                    || self.attachments_of(Entity::Object(id)).iter().any(|a| {
                        let ao = self.obj(*a);
                        ao.controller == o.controller
                            && (ao.chars.has_subtype("Equipment") || ao.chars.has_subtype("Aura"))
                    })
            }
            Filter::DiedThisTurn => o
                .prev
                .is_some_and(|p| self.history.creatures_died.contains(&p)),
            Filter::AttackedThisTurn => self.history.attackers.contains(&id),
            Filter::LinkedChosenColor => self
                .linked_choice(ctx)
                .and_then(|ch| ch.color)
                .is_some_and(|col| c.colors.contains(col)),
            Filter::ManaValueOfChosenQuality => {
                let mv = c.mana_cost.as_ref().map_or(0, |m| m.mana_value());
                match self
                    .linked_choice(ctx)
                    .and_then(|ch| ch.text.clone())
                    .as_deref()
                {
                    Some("odd") => mv % 2 == 1,
                    Some("even") => mv % 2 == 0,
                    _ => false,
                }
            }
            Filter::LinkedChosenCreatureType => self
                .linked_choice(ctx)
                .and_then(|ch| ch.creature_type.clone())
                .is_some_and(|t| c.has_subtype(&t)),
            // CR 607.2d: references to a choice made for the source; an undefined choice
            // matches nothing (CR 607.5a).
            Filter::ChosenColor => self
                .source_choices(ctx)
                .and_then(|ch| ch.color)
                .is_some_and(|col| c.colors.contains(col)),
            // "the chosen type" is whichever type the linked ability chose: a creature
            // type, a basic land type, or a card type.
            Filter::ChosenType => match self.source_choices(ctx) {
                Some(ch) => match ch.creature_type.clone().or(ch.basic_land_type.clone()) {
                    Some(t) => self.matches_view(view, id, &Filter::Subtype(t), ctx),
                    None => ch.card_type.is_some_and(|t| c.card_types.contains(t)),
                },
                None => false,
            },
            Filter::ChosenName => self
                .source_choices(ctx)
                .and_then(|ch| ch.card_name.as_ref())
                .is_some_and(|n| !n.is_empty() && c.name.eq_ignore_ascii_case(n)),
            Filter::Prepared => o.zone == Zone::Battlefield && o.prepared.is_some(),
            Filter::ChosenCardType => self
                .source_choices(ctx)
                .and_then(|ch| ch.card_type)
                .is_some_and(|t| c.card_types.contains(t)),
            Filter::StackTargets(tf) => crate::target_rules::stack_targets_match(self, id, tf, ctx),
            Filter::HasSticker(kind) => crate::stickers::has_sticker(self, id, *kind),
            Filter::Targets(inner) => {
                o.zone == Zone::Stack
                    && o.stack.as_ref().is_some_and(|si| {
                        si.chosen.iter().any(|cm| {
                            cm.targets.iter().flatten().any(|t| match t {
                                // CR 115.9b: a target that left its zone is ignored.
                                Entity::Object(x) => {
                                    self.is_live(*x) && self.matches(*x, inner, ctx)
                                }
                                Entity::Player(_) => false,
                            })
                        })
                    })
            }
            Filter::CastFrom(z) => {
                o.zone == Zone::Stack
                    && o.stack
                        .as_ref()
                        .is_some_and(|si| si.cast.was_cast && si.cast.from == Some(*z))
            }
            Filter::DealtDamageThisTurnBy(sel) => self
                .eval_sel_objects(sel, ctx)
                .into_iter()
                .any(|s| self.history.damage_by_source.contains(&(s, id))),
            Filter::CastWithCost(name) => {
                o.zone == Zone::Stack
                    && o.stack
                        .as_ref()
                        .is_some_and(|si| si.cast.paid.iter().any(|p| p == name))
            }
            Filter::Custom(name) => crate::custom::custom_filter(self, name, id, ctx),
        }
    }

    /// The choices made by the abilities linked to the ability being evaluated (its
    /// source's choices for `ctx.link`), if any (CR 607.2d, 607.5a).
    pub fn linked_choice(&self, ctx: &Ctx) -> Option<&crate::object::Choices> {
        let s = ctx.source?;
        self.obj(s).linked_choices.get(&ctx.link)
    }

    /// Mana value of an object (CR 202.3), accounting for X on the stack (CR 107.3f) and
    /// face-down status.
    pub fn mana_value_of(&self, id: ObjectId) -> u32 {
        let o = self.obj(id);
        let Some(mc) = &o.chars.mana_cost else {
            return 0;
        };
        if o.zone == Zone::Stack {
            let x = o.stack.as_ref().and_then(|s| s.x).unwrap_or(0).max(0) as u32;
            mc.mana_value_with_x(x)
        } else {
            mc.mana_value()
        }
    }

    /// Objects that are attached to the given entity.
    pub fn attachments_of(&self, e: Entity) -> Vec<ObjectId> {
        self.battlefield
            .iter()
            .copied()
            .filter(|a| self.obj(*a).attached_to == Some(e))
            .collect()
    }

    /// Enumerates objects matching a filter in the filter's zone (battlefield by default).
    /// Phased-out permanents are excluded (CR 702.26b).
    pub fn objects_matching(&self, f: &Filter, ctx: &Ctx) -> Vec<ObjectId> {
        let zone = f.zone().unwrap_or(ZoneKind::Battlefield);
        self.objects_in_zone_kind(zone)
            .into_iter()
            .filter(|id| self.matches(*id, f, ctx))
            .collect()
    }

    /// All object ids in every zone of the given kind.
    pub fn objects_in_zone_kind(&self, z: ZoneKind) -> Vec<ObjectId> {
        match z {
            ZoneKind::Battlefield => self.permanent_ids(),
            ZoneKind::Stack => self.stack.clone(),
            ZoneKind::Exile => self.exile.clone(),
            ZoneKind::Command => self.command.clone(),
            ZoneKind::Ante => self.ante.clone(),
            ZoneKind::Hand => self
                .players
                .iter()
                .flat_map(|p| p.hand.iter().copied())
                .collect(),
            ZoneKind::Graveyard => self
                .players
                .iter()
                .flat_map(|p| p.graveyard.iter().copied())
                .collect(),
            ZoneKind::Library => self
                .players
                .iter()
                .flat_map(|p| p.library.iter().copied())
                .collect(),
            ZoneKind::Outside => self
                .players
                .iter()
                .flat_map(|p| p.sideboard.iter().copied())
                .collect(),
        }
    }

    // ------------------------------------------------------------------
    // Selections
    // ------------------------------------------------------------------

    /// Evaluates a selection without making choices (Sel::Choose returns the stored var
    /// or nothing; use `resolve_sel` during resolution to make choices).
    pub fn eval_sel(&self, sel: &Sel, ctx: &Ctx) -> Vec<Entity> {
        match sel {
            Sel::None => vec![],
            Sel::This => ctx.source.map(Entity::Object).into_iter().collect(),
            Sel::Target(slot) => ctx.targets.get(*slot as usize).cloned().unwrap_or_default(),
            Sel::AllTargets => ctx.targets.iter().flatten().copied().collect(),
            Sel::Var(v) => ctx.vars.get(v).cloned().unwrap_or_default(),
            // CR 603.6: an ability can't find an object that went to a zone hidden from its
            // controller (a library, or another player's hand).
            Sel::TriggerObject => ctx
                .event
                .as_ref()
                .and_then(|e| e.object)
                .filter(|o| match self.obj(*o).zone {
                    Zone::Library(_) => false,
                    Zone::Hand(p) => p == ctx.controller,
                    _ => true,
                })
                .map(Entity::Object)
                .into_iter()
                .collect(),
            Sel::TriggerLki => ctx
                .event
                .as_ref()
                .and_then(|e| e.lki.or(e.object))
                .map(Entity::Object)
                .into_iter()
                .collect(),
            Sel::TriggerOtherObject => ctx
                .event
                .as_ref()
                .and_then(|e| e.other)
                .map(Entity::Object)
                .into_iter()
                .collect(),
            Sel::TriggerPlayer => ctx
                .event
                .as_ref()
                .and_then(|e| e.player)
                .map(Entity::Player)
                .into_iter()
                .collect(),
            Sel::TriggerSpell => ctx
                .event
                .as_ref()
                .and_then(|e| e.spell)
                .map(Entity::Object)
                .into_iter()
                .collect(),
            Sel::AttachedTo => ctx
                .source
                .and_then(|s| self.obj(s).attached_to)
                .into_iter()
                .collect(),
            Sel::AttachedToThis => ctx
                .source
                .map(|s| {
                    self.attachments_of(Entity::Object(s))
                        .into_iter()
                        .map(Entity::Object)
                        .collect()
                })
                .unwrap_or_default(),
            Sel::All(f) => self
                .objects_matching(f, ctx)
                .into_iter()
                .map(Entity::Object)
                .collect(),
            Sel::Players(r) => self
                .eval_players(r, ctx)
                .into_iter()
                .map(Entity::Player)
                .collect(),
            Sel::Choose { store, .. } => store
                .and_then(|v| ctx.vars.get(&v).cloned())
                .unwrap_or_default(),
            Sel::ExiledWithCardsNamed(name) => self
                .named_exiles
                .iter()
                .filter(|(p, n, _)| *p == ctx.controller && n == name)
                .map(|(_, _, o)| self.current(*o))
                .filter(|o| self.is_live(*o) && self.obj(*o).zone == Zone::Exile)
                .map(Entity::Object)
                .collect(),
            Sel::CreatorLinked => ctx
                .source
                .and_then(|s| self.obj(s).created_by)
                .map(|(c, link)| {
                    self.obj(c)
                        .linked
                        .get(&link)
                        .map(|v| v.iter().map(|o| Entity::Object(self.current(*o))).collect())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            Sel::Linked => ctx
                .source
                .map(|s| {
                    self.obj(s)
                        .linked
                        .get(&ctx.link)
                        .map(|v| v.iter().map(|o| Entity::Object(self.current(*o))).collect())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            Sel::TopOfGraveyard(r) => self
                .eval_player(r, ctx)
                .and_then(|p| self.player(p).graveyard.last().copied())
                .map(Entity::Object)
                .into_iter()
                .collect(),
            Sel::Union(v) => {
                let mut out: Vec<Entity> = Vec::new();
                for s in v {
                    for e in self.eval_sel(s, ctx) {
                        if !out.contains(&e) {
                            out.push(e);
                        }
                    }
                }
                out
            }
        }
    }

    pub fn eval_sel_objects(&self, sel: &Sel, ctx: &Ctx) -> Vec<ObjectId> {
        self.eval_sel(sel, ctx)
            .into_iter()
            .filter_map(|e| e.object())
            .collect()
    }

    // ------------------------------------------------------------------
    // Values
    // ------------------------------------------------------------------

    pub fn eval_value(&self, v: &Value, ctx: &Ctx) -> i64 {
        match v {
            Value::Const(n) => *n as i64,
            Value::X => ctx.x as i64,
            Value::Count(f) => self.objects_matching(f, ctx).len() as i64,
            Value::CountSel(s) => self.eval_sel(s, ctx).len() as i64,
            Value::CountPlayers(f) => self
                .players_in_game()
                .into_iter()
                .filter(|p| self.player_filter_matches(f, *p, ctx))
                .count() as i64,
            Value::PowerOf(s) => self
                .eval_sel_objects(s, ctx)
                .iter()
                .map(|o| self.obj(*o).power() as i64)
                .sum(),
            Value::ToughnessOf(s) => self
                .eval_sel_objects(s, ctx)
                .first()
                .map_or(0, |o| self.obj(*o).toughness() as i64),
            // CR 607.3: several objects give several answers, which are summed.
            Value::ManaValueOf(s) => self
                .eval_sel_objects(s, ctx)
                .iter()
                .map(|o| self.mana_value_of(*o) as i64)
                .sum(),
            Value::LoyaltyOf(s) => self
                .eval_sel_objects(s, ctx)
                .first()
                .map_or(0, |o| self.obj(*o).loyalty() as i64),
            Value::CountersOn(s, k) => self
                .eval_sel(s, ctx)
                .iter()
                .map(|e| match e {
                    Entity::Object(o) => match k {
                        Some(k) => self.obj(*o).counter(k) as i64,
                        None => self.obj(*o).counters.values().map(|n| *n as i64).sum(),
                    },
                    Entity::Player(p) => match k {
                        Some(k) => self.player(*p).counter(k) as i64,
                        None => self.player(*p).counters.values().map(|n| *n as i64).sum(),
                    },
                })
                .sum(),
            Value::PlayerCounters(r, k) => self
                .eval_player(r, ctx)
                .map_or(0, |p| self.player(p).counter(k) as i64),
            Value::LifeTotal(r) => self
                .eval_player(r, ctx)
                .map_or(0, |p| self.player(p).life as i64),
            Value::StartingLife => self.config.starting_life as i64,
            Value::HandSize(r) => self
                .eval_player(r, ctx)
                .map_or(0, |p| self.player(p).hand.len() as i64),
            Value::LibrarySize(r) => self
                .eval_player(r, ctx)
                .map_or(0, |p| self.player(p).library.len() as i64),
            Value::GraveyardSize(r) => self
                .eval_player(r, ctx)
                .map_or(0, |p| self.player(p).graveyard.len() as i64),
            Value::CardsInGraveyard(r, f) => self.eval_player(r, ctx).map_or(0, |p| {
                self.player(p)
                    .graveyard
                    .iter()
                    .filter(|o| self.matches(**o, f, ctx))
                    .count() as i64
            }),
            Value::EventAmount => ctx.event.as_ref().map_or(0, |e| e.amount as i64),
            Value::Prev => ctx.prev_value,
            Value::Var(v) => ctx.nums.get(v).copied().unwrap_or(0),
            Value::Devotion(colors) => {
                let mut n = 0i64;
                for o in self.permanents().filter(|o| o.controller == ctx.controller) {
                    if let Some(mc) = &o.chars.mana_cost {
                        n += mc
                            .symbols
                            .iter()
                            .filter(|s| s.colors().intersects(*colors))
                            .count() as i64;
                    }
                }
                n
            }
            Value::Domain => {
                let mut types = std::collections::BTreeSet::new();
                for o in self
                    .permanents()
                    .filter(|o| o.controller == ctx.controller && o.chars.is_land())
                {
                    for s in &o.chars.subtypes {
                        if is_basic_land_type(s) {
                            types.insert(s.clone());
                        }
                    }
                }
                types.len() as i64
            }
            Value::StormCount => {
                let this = ctx.stack_obj.or(ctx.source);
                let n = self.history.spells_cast.len() as i64;
                if this.is_some_and(|s| self.history.spells_cast.iter().any(|(_, x)| *x == s)) {
                    n - 1
                } else {
                    n
                }
            }
            Value::CardsDrawnThisTurn(r) => self.eval_player(r, ctx).map_or(0, |p| {
                self.history.cards_drawn.get(&p).copied().unwrap_or(0) as i64
            }),
            Value::LifeGainedThisTurn(r) => self.eval_player(r, ctx).map_or(0, |p| {
                self.history.life_gained.get(&p).copied().unwrap_or(0) as i64
            }),
            Value::LifeLostThisTurn(r) => self.eval_player(r, ctx).map_or(0, |p| {
                self.history.life_lost.get(&p).copied().unwrap_or(0) as i64
            }),
            Value::CreaturesDiedThisTurn => self.history.creatures_died.len() as i64,
            Value::TimesResolvedThisTurn => ctx
                .source
                .map(|s| {
                    self.obj(s)
                        .triggers_this_turn
                        .get(&(ctx.ability_uid | crate::triggers::turn_keys::RESOLVED))
                        .copied()
                        .unwrap_or(0) as i64
                })
                .unwrap_or(0),
            Value::CardTypesAmong(f) => {
                let mut set = CardTypeSet::NONE;
                for o in self.objects_matching(f, ctx) {
                    set = set.union(self.obj(o).chars.card_types);
                }
                set.count() as i64
            }
            Value::ClassLevel => ctx
                .source
                .map_or(1, |s| self.obj(s).class_level.max(1) as i64),
            Value::XOf(s) => self
                .eval_sel_objects(s, ctx)
                .first()
                .map_or(0, |o| crate::object::x_value_of(self.obj(*o)) as i64),
            Value::ColorPairsAmong(f) => {
                let mut pairs: Vec<ColorSet> = Vec::new();
                for o in self.objects_matching(f, ctx) {
                    let c = self.obj(o).chars.colors;
                    if c.is_color_pair() && !pairs.contains(&c) {
                        pairs.push(c);
                    }
                }
                pairs.len() as i64
            }
            Value::GreatestPower(f) => self
                .objects_matching(f, ctx)
                .iter()
                .map(|o| self.obj(*o).power() as i64)
                .max()
                .unwrap_or(0),
            Value::GreatestManaValue(f) => self
                .objects_matching(f, ctx)
                .iter()
                .map(|o| self.mana_value_of(*o) as i64)
                .max()
                .unwrap_or(0),
            Value::ColorsSpent => self.cast_info(ctx).map_or(0, |c| {
                let mut s = ColorSet::NONE;
                for m in &c.mana_spent {
                    if let Some(col) = m.color() {
                        s.insert(col);
                    }
                }
                s.count() as i64
            }),
            Value::ManaSpent => self.cast_info(ctx).map_or(0, |c| c.mana_spent.len() as i64),
            // The number chosen, paid or noted by the linked ability (CR 607.2e, 607.2g),
            // or else by any of the source's abilities.
            Value::Chosen => self
                .linked_choice(ctx)
                .and_then(|c| c.number)
                .or_else(|| ctx.source.and_then(|s| self.obj(s).choices.number))
                .unwrap_or(0) as i64,
            Value::TimesKicked => self.cast_info(ctx).map_or(0, |c| c.times_kicked as i64),
            Value::Speed(r) => self
                .eval_player(r, ctx)
                .and_then(|p| self.player(p).speed)
                .unwrap_or(0) as i64,
            Value::Sum(v) => v.iter().map(|x| self.eval_value(x, ctx)).sum(),
            Value::Diff(a, b) => self.eval_value(a, ctx) - self.eval_value(b, ctx),
            Value::Mul(a, b) => self.eval_value(a, ctx) * self.eval_value(b, ctx),
            Value::Div(a, d, up) => {
                let n = self.eval_value(a, ctx);
                let d = *d as i64;
                if *up {
                    (n + d - 1).div_euclid(d)
                } else {
                    n.div_euclid(d)
                }
            }
            Value::Min(a, b) => self.eval_value(a, ctx).min(self.eval_value(b, ctx)),
            Value::Max(a, b) => self.eval_value(a, ctx).max(self.eval_value(b, ctx)),
            Value::Custom(name) => crate::custom::custom_value(self, name, ctx),
        }
    }

    /// How the spell was cast, for "if it was kicked", "if you cast it", "mana spent to
    /// cast it" (CR 607.2i): the resolving spell's own information, or, for an ability of
    /// a permanent (including a triggered ability checking its intervening "if", CR 603.4),
    /// the spell that became that permanent (CR 400.7d).
    pub fn cast_info<'a>(&'a self, ctx: &'a Ctx) -> Option<&'a CastInfo> {
        let resolving_ability = ctx
            .stack_obj
            .is_some_and(|s| self.obj(s).kind == ObjKind::StackAbility);
        let own = if resolving_ability {
            None
        } else {
            ctx.cast.as_ref()
        };
        if let Some(c) = own.filter(|c| c.was_cast) {
            return Some(c);
        }
        ctx.source
            .and_then(|s| {
                let o = self.obj(s);
                // A spell on the stack ("when you cast ~, if it was kicked") keeps its
                // cast information in its stack info.
                o.cast
                    .as_deref()
                    .or_else(|| o.stack.as_ref().map(|si| &si.cast))
            })
            .or(own)
    }

    // ------------------------------------------------------------------
    // Conditions
    // ------------------------------------------------------------------

    pub fn eval_cond(&self, c: &Condition, ctx: &Ctx) -> bool {
        match c {
            Condition::Always => true,
            Condition::Never => false,
            Condition::Not(x) => !self.eval_cond(x, ctx),
            Condition::And(v) => v.iter().all(|x| self.eval_cond(x, ctx)),
            Condition::Or(v) => v.iter().any(|x| self.eval_cond(x, ctx)),
            Condition::Compare(a, cmp, b) => {
                cmp.eval(self.eval_value(a, ctx), self.eval_value(b, ctx))
            }
            Condition::Exists(f) => !self.objects_matching(f, ctx).is_empty(),
            Condition::SelNonEmpty(s) => !self.eval_sel(s, ctx).is_empty(),
            Condition::SelMatches(s, f) => {
                let objs = self.eval_sel_objects(s, ctx);
                !objs.is_empty() && objs.iter().all(|o| self.matches(*o, f, ctx))
            }
            Condition::PlayerMatches(r, f) => self
                .eval_players(r, ctx)
                .into_iter()
                .any(|p| self.player_filter_matches(f, p, ctx)),
            Condition::YourTurn => self.is_active_player(ctx.controller),
            Condition::NotYourTurn => !self.is_active_player(ctx.controller),
            // For a permanent's abilities, how the permanent was cast (CR 607.2i).
            Condition::CostPaid(name) => self
                .cast_info(ctx)
                .is_some_and(|c| c.paid.iter().any(|p| p == name)),
            Condition::WasCast => self.cast_info(ctx).is_some_and(|c| c.was_cast),
            Condition::PrevHappened => ctx.prev_happened,
            Condition::PrevAffectedAny => !ctx.prev_affected.is_empty(),
            Condition::CastFrom(z) => self
                .cast_info(ctx)
                .is_some_and(|c| c.was_cast && c.from == Some(*z)),
            Condition::Phase(p) => match p {
                PhaseCond::Combat => self.turn.step.is_combat(),
                PhaseCond::MainPhase => self.turn.step.is_main(),
                PhaseCond::Upkeep => self.turn.step == crate::turn::Step::Upkeep,
                PhaseCond::DeclareAttackers => {
                    self.turn.step == crate::turn::Step::DeclareAttackers
                }
                PhaseCond::EndStep => self.turn.step == crate::turn::Step::End,
            },
            Condition::ChosenWord(w) => self
                .linked_choice(ctx)
                .and_then(|c| c.text.as_deref())
                .is_some_and(|t| t == w.as_str()),
            Condition::AllTriggerConditionsThisTurn(conds) => conds.iter().all(|c| {
                self.turn_events
                    .iter()
                    .chain(self.events.iter())
                    .any(|ev| !self.trigger_matches_ctx(c, ctx, ev).is_empty())
            }),
            Condition::CitysBlessing => self.player(ctx.controller).has_citys_blessing,
            Condition::IsMonarch => self.monarch == Some(ctx.controller),
            Condition::HasInitiative => self.initiative == Some(ctx.controller),
            Condition::IsDay => self.day == Some(true),
            Condition::IsNight => self.day == Some(false),
            Condition::MaxSpeed => self.player(ctx.controller).speed.unwrap_or(0) >= 4,
            Condition::CombatTiming(t) => crate::combat::combat_timing_ok(self, *t),
            Condition::Chose(w) => self
                .source_choices(ctx)
                .and_then(|ch| ch.text.as_ref())
                .is_some_and(|t| t.eq_ignore_ascii_case(w)),
            Condition::Custom(name) => crate::custom::custom_condition(self, name, ctx),
        }
    }
}
