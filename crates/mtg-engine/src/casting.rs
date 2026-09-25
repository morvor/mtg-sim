//! Casting spells (CR 601), activating abilities (CR 602, 605, 606), playing lands
//! (CR 305.1), special actions (CR 116), costs (CR 118), and legal-action enumeration.

use crate::ability::*;
use crate::decision::{Action, Answer, Decision, SpecialAction};
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::mana::*;
use crate::object::*;
use crate::replacement::*;
use crate::types::*;

/// Why an action was illegal (the game state is rolled back, CR 733).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Illegal(pub String);

/// One way of casting a card.
#[derive(Clone, Debug)]
pub struct CastOption {
    pub method: CastMethod,
    /// Replaces the mana cost (alternative cost, CR 118.9). `None` = pay the mana cost.
    pub alt_cost: Option<Cost>,
    /// Face/half to cast.
    pub face: FaceState,
    /// Additional costs that must be paid with this method.
    pub extra_cost: Option<Cost>,
    /// Can be cast at instant speed with this method.
    pub flash: bool,
    /// Ignore normal timing (e.g. cast during resolution).
    pub any_time: bool,
    /// Name recorded in `CastInfo::paid` (e.g. "flashback", "dash").
    pub tag: Option<&'static str>,
}

impl CastOption {
    pub fn normal(face: FaceState) -> CastOption {
        CastOption {
            method: CastMethod::Normal,
            alt_cost: None,
            face,
            extra_cost: None,
            flash: false,
            any_time: false,
            tag: None,
        }
    }
}

/// Grants for "you may play/cast that card" effects (from resolved effects).
#[derive(Clone, Debug)]
pub struct PlayGrant {
    pub player: PlayerId,
    pub object: ObjectId,
    pub duration: Duration,
    pub free: bool,
    pub source: Option<ObjectId>,
    pub turn: u32,
}

pub fn grant_play_permission(
    g: &mut Game,
    p: PlayerId,
    objs: Vec<ObjectId>,
    duration: Duration,
    free: bool,
    source: Option<ObjectId>,
) {
    let turn = g.turn.number;
    for o in objs {
        g.play_grants.push(PlayGrant {
            player: p,
            object: o,
            duration: duration.clone(),
            free,
            source,
            turn,
        });
    }
}

/// Casts a card during the resolution of a spell or ability (CR 608.2g). No player
/// receives priority afterward.
pub fn cast_during_resolution(
    g: &mut Game,
    p: PlayerId,
    card: ObjectId,
    method: CastMethod,
) -> Result<ObjectId, Illegal> {
    let face = FaceState::Front;
    let mut opt = CastOption::normal(face);
    opt.any_time = true;
    if method == CastMethod::Free {
        opt.method = CastMethod::Free;
        opt.alt_cost = Some(Cost::free());
    }
    g.cast_with_option(p, card, opt)
}

impl Game {
    // ------------------------------------------------------------------
    // Legal actions
    // ------------------------------------------------------------------

    /// Priority actions available to a player (a superset check: some may still fail
    /// when attempted, in which case the game state is rolled back).
    pub fn legal_actions(&mut self, p: PlayerId) -> Vec<Action> {
        if self.dirty {
            self.recompute();
        }
        let mut out = vec![Action::Pass];
        // Lands (CR 305.1, 505.6b).
        if self.can_play_land_now(p) {
            for c in self.playable_land_cards(p) {
                out.push(Action::PlayLand { card: c });
            }
        }
        // Spells.
        for c in self.castable_zone_cards(p) {
            for opt in self.cast_options(p, c) {
                if self.can_begin_cast(p, c, &opt) {
                    out.push(Action::Cast {
                        card: c,
                        method: opt.method.clone(),
                    });
                }
            }
        }
        // Activated abilities (non-mana).
        for (src, a) in self.activatable_abilities(p) {
            if !a.is_mana_ability() {
                out.push(Action::Activate {
                    source: src,
                    ability: a.uid,
                });
            }
        }
        // Special actions.
        out.extend(crate::keyword_impls::special_actions(self, p));
        out.dedup();
        out
    }

    pub fn can_play_land_now(&self, p: PlayerId) -> bool {
        // CR 305.2, 505.6b: active player, main phase, empty stack, land play available.
        self.turn.active == p
            && self.turn.step.is_main()
            && self.stack.is_empty()
            && self.turn.priority == Some(p)
            && self.player(p).lands_played_this_turn < self.player(p).land_plays
            && !self.player_restricted(p, |r| matches!(r, Restriction::CantPlayLands(_)))
    }

    fn playable_land_cards(&self, p: PlayerId) -> Vec<ObjectId> {
        let mut out: Vec<ObjectId> = self
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|c| self.card_has_land_face(*c))
            .collect();
        // Play permissions from other zones ("play lands from the top of your library").
        for c in self.permitted_cards(p) {
            if self.card_has_land_face(c) && !out.contains(&c) {
                out.push(c);
            }
        }
        out
    }

    fn card_has_land_face(&self, c: ObjectId) -> bool {
        let o = self.obj(c);
        o.chars.is_land()
            || o.card.as_ref().is_some_and(|d| {
                d.layout == crate::card::Layout::ModalDfc
                    && d.faces.get(1).is_some_and(|f| f.chars.is_land())
            })
    }

    /// Cards in zones other than hand that the player may play because of static
    /// permissions or grants.
    pub fn permitted_cards(&self, p: PlayerId) -> Vec<ObjectId> {
        let mut out = Vec::new();
        for (src, ctl, perm) in &self.statics.play_permissions {
            let ctx = Ctx::new(Some(*src), *ctl);
            if !self.player_rel_matches(perm.who, p, &ctx) {
                continue;
            }
            let cards: Vec<ObjectId> = match perm.zone {
                ZoneKind::Library => {
                    if perm.top_only {
                        self.library_top(p).into_iter().collect()
                    } else {
                        self.player(p).library.clone()
                    }
                }
                ZoneKind::Graveyard => self.player(p).graveyard.clone(),
                ZoneKind::Exile => self.exile.clone(),
                ZoneKind::Hand => self.player(p).hand.clone(),
                ZoneKind::Command => self.command.clone(),
                _ => vec![],
            };
            for c in cards {
                if self.matches(c, &perm.what, &ctx) && !out.contains(&c) {
                    out.push(c);
                }
            }
        }
        for gnt in &self.play_grants {
            if gnt.player == p && self.is_live(gnt.object) && !out.contains(&gnt.object) {
                out.push(gnt.object);
            }
        }
        out
    }

    fn castable_zone_cards(&self, p: PlayerId) -> Vec<ObjectId> {
        let mut out: Vec<ObjectId> = self.player(p).hand.clone();
        // Cards castable by keyword from other zones (flashback, escape, foretell, ...).
        for c in self
            .player(p)
            .graveyard
            .iter()
            .chain(self.exile.iter())
            .chain(self.command.iter())
        {
            if !out.contains(c) {
                out.push(*c);
            }
        }
        for c in self.permitted_cards(p) {
            if !out.contains(&c) {
                out.push(c);
            }
        }
        out
    }

    /// Ways a card can be cast by this player from where it is.
    pub fn cast_options(&self, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        let o = self.obj(card);
        let mut out = Vec::new();
        let in_hand = o.zone == Zone::Hand(p);
        let permitted = self.permitted_cards(p).contains(&card);
        let def = o.card.clone();
        let layout = def.as_ref().map(|d| d.layout);
        if in_hand || permitted {
            let grant_free = self
                .play_grants
                .iter()
                .any(|g| g.player == p && g.object == card && g.free);
            let push_face = |face: FaceState, out: &mut Vec<CastOption>| {
                let mut opt = CastOption::normal(face);
                if let FaceState::Half(i) = face {
                    opt.method = CastMethod::Half(i);
                }
                if grant_free {
                    opt.method = CastMethod::Free;
                    opt.alt_cost = Some(Cost::free());
                }
                out.push(opt);
            };
            match layout {
                Some(crate::card::Layout::Split) => {
                    push_face(FaceState::Half(0), &mut out);
                    push_face(FaceState::Half(1), &mut out);
                    if o.chars.has_keyword(KeywordKind::Fuse) && in_hand {
                        let mut f = CastOption::normal(FaceState::Fused);
                        f.method = CastMethod::Keyword(KeywordKind::Fuse);
                        out.push(f);
                    }
                }
                Some(crate::card::Layout::Adventure) | Some(crate::card::Layout::Prepare) => {
                    push_face(FaceState::Front, &mut out);
                    push_face(FaceState::Half(1), &mut out);
                }
                Some(crate::card::Layout::ModalDfc) => {
                    push_face(FaceState::Front, &mut out);
                    push_face(FaceState::Back, &mut out);
                }
                _ => push_face(FaceState::Front, &mut out),
            }
            // Alternative costs from the card's own abilities ("You may pay X rather than pay
            // this spell's mana cost", evoke, dash, ...).
            for a in &o.chars.abilities {
                if let AbilityKind::Static(s) = &a.kind {
                    if let StaticEffect::CostModifier(cm) = &s.effect {
                        if let (CostTarget::ThisSpell, CostChange::AlternativeCost(c)) =
                            (&cm.applies_to, &cm.change)
                        {
                            let mut opt = CastOption::normal(FaceState::Front);
                            opt.method = CastMethod::Alternative(a.uid);
                            opt.alt_cost = Some(c.clone());
                            out.push(opt);
                        }
                    }
                }
            }
        }
        out.extend(crate::keyword_impls::keyword_cast_options(self, p, card));
        // Lands can't be cast (CR 305.9).
        out.retain(|opt| {
            let chars = self.face_characteristics(card, opt.face);
            !chars.is_land()
        });
        out
    }

    /// Characteristics a card would have as a spell cast with the given face (CR 601.3e).
    pub fn face_characteristics(&self, card: ObjectId, face: FaceState) -> Characteristics {
        let o = self.obj(card);
        match (&o.card, face) {
            (Some(d), FaceState::Front) if d.layout == crate::card::Layout::Split => {
                d.characteristics(FaceState::Front)
            }
            (Some(d), f) if f != FaceState::Front => d.characteristics(f),
            _ => o.chars.clone(),
        }
    }

    /// Whether the player could begin casting the card this way right now (timing,
    /// permissions, targets, and an optimistic cost check).
    pub fn can_begin_cast(&self, p: PlayerId, card: ObjectId, opt: &CastOption) -> bool {
        let chars = self.face_characteristics(card, opt.face);
        if !opt.any_time && !self.timing_allows_cast(p, card, &chars, opt) {
            return false;
        }
        if self.cast_prohibited(p, card, &chars) {
            return false;
        }
        // Targets must be available.
        let ctx = Ctx::new(Some(card), p);
        let bodies: Vec<&Body> = chars
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Spell(s) => Some(&s.body),
                _ => None,
            })
            .collect();
        for b in bodies {
            if b.modal.is_none() && !self.targets_possible(&b.targets, &ctx, card) {
                return false;
            }
        }
        // Aura spells need a legal object to enchant (CR 303.4a).
        if chars.has_subtype("Aura") && chars.is(CardType::Enchantment) {
            if let Some(spec) = crate::attach::aura_target_spec(&chars) {
                if !self.targets_possible(&[spec], &ctx, card) {
                    return false;
                }
            }
        }
        // Optimistic cost check.
        let cost = self.base_total_cost(p, card, &chars, opt, 0);
        self.can_pay_cost_optimistic(p, &cost, Some(card), &chars)
    }

    fn timing_allows_cast(
        &self,
        p: PlayerId,
        card: ObjectId,
        chars: &Characteristics,
        opt: &CastOption,
    ) -> bool {
        // "Cast this spell only ..." (e.g. combat timing windows, CR 506.8).
        if !crate::combat::spell_cast_restrictions_ok(self, p, card, chars) {
            return false;
        }
        if self.turn.priority != Some(p) {
            return false;
        }
        let instant_speed = chars.is(CardType::Instant)
            || chars.has_keyword(KeywordKind::Flash)
            || opt.flash
            || self.flash_permitted(p, card);
        if instant_speed {
            return !self.player_restricted(p, |r| matches!(r, Restriction::SorcerySpeedOnly(_)))
                || self.is_sorcery_timing(p);
        }
        self.is_sorcery_timing(p)
    }

    fn flash_permitted(&self, p: PlayerId, card: ObjectId) -> bool {
        self.statics
            .flash_permissions
            .iter()
            .any(|(s, c, who, what)| {
                let ctx = Ctx::new(Some(*s), *c);
                self.player_rel_matches(*who, p, &ctx) && self.matches(card, what, &ctx)
            })
    }

    fn cast_prohibited(&self, p: PlayerId, card: ObjectId, _chars: &Characteristics) -> bool {
        let spells_cast = self
            .history
            .spells_cast
            .iter()
            .filter(|(q, _)| *q == p)
            .count() as u32;
        let check = |r: &Restriction, s: Option<ObjectId>, c: PlayerId| -> bool {
            let ctx = Ctx::new(s, c);
            match r {
                Restriction::CantCast { who, what } => {
                    self.player_filter_matches(who, p, &ctx) && self.matches(card, what, &ctx)
                }
                Restriction::MaxSpellsPerTurn(who, n) => {
                    self.player_filter_matches(who, p, &ctx) && spells_cast >= *n
                }
                _ => false,
            }
        };
        self.statics.restrictions.iter().any(|(s, c, r)| check(r, Some(*s), *c))
            || self.rule_effects.iter().any(|e| check(&e.restriction, e.source, e.controller))
            // CR 702.61a split second: no casting while a split-second spell is on the stack.
            || self.split_second_on_stack()
    }

    pub fn split_second_on_stack(&self) -> bool {
        self.stack
            .iter()
            .any(|s| self.obj(*s).is_spell() && self.obj(*s).has_keyword(KeywordKind::SplitSecond))
    }

    // ------------------------------------------------------------------
    // Performing actions
    // ------------------------------------------------------------------

    /// Performs a priority action. Illegal actions leave the game unchanged.
    pub fn perform_action(&mut self, p: PlayerId, action: Action) -> Result<(), Illegal> {
        let r = match action {
            Action::Pass => Ok(()),
            Action::Concede => {
                self.player_loses(p);
                Ok(())
            }
            Action::PlayLand { card } => self.play_land(p, card),
            Action::Cast { card, method } => self.cast_spell(p, card, method).map(|_| ()),
            Action::Activate { source, ability } => {
                self.activate_ability(p, source, ability).map(|_| ())
            }
            Action::Special(sa) => crate::keyword_impls::perform_special_action(self, p, sa),
        };
        self.flush_events();
        r
    }

    /// Plays a land (CR 305.1): a special action that doesn't use the stack.
    pub fn play_land(&mut self, p: PlayerId, card: ObjectId) -> Result<(), Illegal> {
        if !self.can_play_land_now(p) {
            return Err(Illegal("can't play a land now".into()));
        }
        if !self.playable_land_cards(p).contains(&card) {
            return Err(Illegal("not a playable land".into()));
        }
        let o = self.obj(card);
        let face = if o.chars.is_land() {
            FaceState::Front
        } else {
            FaceState::Back
        };
        self.players[p.idx()].lands_played_this_turn += 1;
        self.play_grants.retain(|g| g.object != card);
        let new = self.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::PlayLand,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                face: Some(face),
                ..Default::default()
            },
            source: None,
        });
        if let Some(n) = new {
            self.emit(Event::LandPlayed { player: p, land: n });
        }
        self.flush_events();
        Ok(())
    }

    /// Casts a spell with the given method (CR 601.2).
    pub fn cast_spell(
        &mut self,
        p: PlayerId,
        card: ObjectId,
        method: CastMethod,
    ) -> Result<ObjectId, Illegal> {
        let opts = self.cast_options(p, card);
        let opt = opts
            .into_iter()
            .find(|o| o.method == method)
            .ok_or_else(|| Illegal(format!("no such casting method {method:?}")))?;
        if !opt.any_time {
            let chars = self.face_characteristics(card, opt.face);
            if !self.timing_allows_cast(p, card, &chars, &opt) {
                return Err(Illegal("timing".into()));
            }
            if self.cast_prohibited(p, card, &chars) {
                return Err(Illegal("prohibited".into()));
            }
        }
        self.cast_with_option(p, card, opt)
    }

    pub(crate) fn cast_with_option(
        &mut self,
        p: PlayerId,
        card: ObjectId,
        opt: CastOption,
    ) -> Result<ObjectId, Illegal> {
        let snapshot = self.clone();
        match self.cast_inner(p, card, &opt) {
            Ok(id) => Ok(id),
            Err(e) => {
                // CR 733: return to the moment before casting was proposed.
                let agents = self.agents.clone();
                *self = snapshot;
                self.agents = agents;
                Err(e)
            }
        }
    }

    fn cast_inner(
        &mut self,
        p: PlayerId,
        card: ObjectId,
        opt: &CastOption,
    ) -> Result<ObjectId, Illegal> {
        let from = self.obj(card).zone;
        let from_kind = from.kind();
        // 601.2a: move the card to the stack.
        if let Some(list) = self.zone_list_mut(from) {
            list.retain(|x| *x != card);
        }
        let id = self.create_incarnation(card, Zone::Stack);
        {
            let o = &mut self.objects[id.0 as usize];
            o.zone = Zone::Stack;
            o.controller = p;
            o.base_controller = p;
            o.face = opt.face;
            if let CastMethod::FaceDown(_) = opt.method {
                o.face_down = true;
            }
            if let Some(d) = o.card.clone() {
                o.base = d.characteristics(opt.face);
            }
        }
        self.stack.push(id);
        self.play_grants.retain(|g| g.object != card);
        let mut cast_info = CastInfo {
            method: opt.method.clone(),
            from: from_kind,
            was_cast: true,
            turn: self.turn.number,
            ..Default::default()
        };
        if let Some(t) = opt.tag {
            cast_info.paid.push(t.into());
        }
        self.objects[id.0 as usize].stack = Some(Box::new(StackInfo {
            kind: StackKind::Spell,
            chosen: vec![],
            x: None,
            cast: cast_info.clone(),
            event: None,
            source_lki: None,
            chosen_values: Default::default(),
        }));
        self.emit(Event::ZoneChange {
            old: card,
            new: id,
            from,
            to: Zone::Stack,
            cause: MoveCause::Cast,
            by: Some(p),
            lookback: None,
        });
        self.recompute();
        let chars = self.obj(id).chars.clone();

        // 601.2b: optional additional costs (kicker etc.) and X.
        let mut extra = Cost::free();
        for (name, cost, repeatable) in crate::keyword_impls::optional_additional_costs(self, id) {
            if repeatable {
                let n = match self.ask(
                    p,
                    Decision::OptionalCost {
                        source: id,
                        name: name.to_string(),
                        repeatable: true,
                    },
                ) {
                    Answer::Number(n) if n >= 0 => n as u32,
                    Answer::Bool(true) => 1,
                    _ => 0,
                };
                for _ in 0..n {
                    add_cost(&mut extra, &cost);
                    cast_info.paid.push(name.clone());
                }
                if n > 0 {
                    cast_info.times_kicked += n;
                }
            } else if self.can_pay_cost_optimistic(p, &cost, Some(id), &chars)
                && matches!(
                    self.ask(
                        p,
                        Decision::OptionalCost {
                            source: id,
                            name: name.to_string(),
                            repeatable: false
                        }
                    ),
                    Answer::Bool(true)
                )
            {
                add_cost(&mut extra, &cost);
                cast_info.paid.push(name.clone());
                if name.as_str() == "kicker" || name.as_str() == "multikicker" {
                    cast_info.times_kicked += 1;
                }
            }
        }
        if let Some(c) = &opt.extra_cost {
            add_cost(&mut extra, c);
        }
        let base_cost_has_x = match &opt.alt_cost {
            Some(c) => c.mana.as_ref().is_some_and(|m| m.has_x()),
            None => chars.mana_cost.as_ref().is_some_and(|m| m.has_x()),
        } || extra.mana.as_ref().is_some_and(|m| m.has_x());
        let mut x: i64 = 0;
        if base_cost_has_x {
            let max = self.max_mana_available(p) as i64;
            x = match self.ask(p, Decision::ChooseX { source: id, max }) {
                Answer::Number(n) if n >= 0 => n,
                _ => max.max(0),
            };
        }
        cast_info.x = Some(x as i32);
        if let Some(si) = self.objects[id.0 as usize].stack.as_mut() {
            si.x = Some(x as i32);
            si.cast = cast_info.clone();
        }

        // 601.2c–d: modes and targets.
        let body = self.spell_body(id);
        let mut ctx = Ctx::new(Some(id), p);
        ctx.x = x as i32;
        ctx.cast = Some(cast_info.clone());
        let mut body = body;
        if chars.has_subtype("Aura")
            && chars.is(CardType::Enchantment)
            && !opt.method.eq(&CastMethod::FaceDown(KeywordKind::Morph))
        {
            // CR 303.4a / 115.1b: an Aura spell targets what it will enchant.
            if let Some(spec) = crate::attach::aura_target_spec(&chars) {
                if body.targets.is_empty() {
                    body.targets.push(spec);
                }
            }
        }
        if self.obj(id).face_down {
            body = Body::default();
        }
        body = crate::keyword_impls::adjust_spell_body(self, id, body);
        if !self.choose_modes_and_targets(id, &body, &mut ctx) {
            return Err(Illegal("no legal targets / modes".into()));
        }

        // 601.2f: total cost.
        let mut total = self.base_total_cost(p, id, &chars, opt, x as u32);
        add_cost(&mut total, &extra);
        // Mode costs (spree, etc.).
        if let (Some(modal), Some(si)) = (&body.modal, self.obj(id).stack.as_deref()) {
            for cm in &si.chosen {
                if let Some(m) = cm.mode {
                    if let Some(c) = &modal.modes[m].cost {
                        add_cost(&mut total, c);
                    }
                }
            }
        }
        if let Some(m) = total.mana.as_mut() {
            *m = m.with_x(x as u32);
        }
        // 601.2g–h: activate mana abilities and pay.
        let spend = SpendContext {
            is_spell: true,
            is_ability: false,
            card_types: chars.card_types,
            subtypes: chars.subtypes.to_vec(),
            has_x: base_cost_has_x,
            source: Some(id),
        };
        let paid = self.pay_total_cost(p, &total, Some(id), &spend, &ctx)?;
        if let Some(si) = self.objects[id.0 as usize].stack.as_mut() {
            si.cast.mana_spent = paid.mana_spent.clone();
            si.cast.cost_objects = paid.objects.clone();
        }
        if matches!(from, Zone::Command) && self.obj(id).is_commander {
            *self.players[p.idx()]
                .commander_casts
                .entry(chars.name.clone())
                .or_insert(0) += 1;
        }
        // 601.2i: the spell becomes cast.
        self.log(|g| format!("{p} casts {}", g.describe(id)));
        self.emit(Event::SpellCast {
            spell: id,
            player: p,
            from: from_kind,
        });
        self.flush_events();
        Ok(id)
    }

    /// Mana cost (or alternative cost) plus cost modifiers (CR 601.2f). Does not include
    /// optional additional costs chosen during casting.
    pub fn base_total_cost(
        &self,
        p: PlayerId,
        card: ObjectId,
        chars: &Characteristics,
        opt: &CastOption,
        x: u32,
    ) -> Cost {
        let mut cost = match &opt.alt_cost {
            Some(c) => c.clone(),
            None => Cost {
                mana: Some(chars.mana_cost.clone().unwrap_or_default()),
                parts: vec![],
            },
        };
        if let Some(e) = &opt.extra_cost {
            if opt.alt_cost.is_none() {
                add_cost(&mut cost, e);
            }
        }
        // Own additional costs ("As an additional cost to cast this spell, ...").
        for a in &chars.abilities {
            if let AbilityKind::Static(s) = &a.kind {
                if let StaticEffect::CostModifier(cm) = &s.effect {
                    if let CostTarget::ThisSpell = cm.applies_to {
                        let ctx = Ctx::new(Some(card), p);
                        match &cm.change {
                            CostChange::AdditionalCost(c) => add_cost(&mut cost, c),
                            CostChange::IncreaseGeneric(v) => {
                                let n = self.eval_value(v, &ctx).max(0) as u32;
                                add_cost(&mut cost, &Cost::mana(ManaCost::generic(n)));
                            }
                            CostChange::ReduceGeneric(v) => {
                                let n = self.eval_value(v, &ctx).max(0) as u32;
                                if let Some(m) = cost.mana.as_mut() {
                                    m.reduce_generic(n);
                                }
                            }
                            CostChange::ReduceColored(c, v) => {
                                let n = self.eval_value(v, &ctx).max(0);
                                if let Some(m) = cost.mana.as_mut() {
                                    for _ in 0..n {
                                        m.reduce_colored(*c);
                                    }
                                }
                            }
                            CostChange::IncreaseMana(m) => {
                                add_cost(&mut cost, &Cost::mana(m.clone()))
                            }
                            CostChange::AlternativeCost(_) => {}
                        }
                    }
                }
            }
        }
        // Static cost modifiers from other permanents: increases first, then reductions.
        let mut reductions: Vec<(u32, Option<Color>)> = Vec::new();
        for (src, ctl, cm) in &self.statics.cost_modifiers {
            let ctx = Ctx::new(Some(*src), *ctl);
            let applies = match &cm.applies_to {
                CostTarget::Spells(f) => {
                    self.player_rel_matches(cm.who, p, &ctx) && self.matches(card, f, &ctx)
                }
                _ => false,
            };
            if !applies {
                continue;
            }
            match &cm.change {
                CostChange::IncreaseGeneric(v) => {
                    let n = self.eval_value(v, &ctx).max(0) as u32;
                    add_cost(&mut cost, &Cost::mana(ManaCost::generic(n)));
                }
                CostChange::IncreaseMana(m) => add_cost(&mut cost, &Cost::mana(m.clone())),
                CostChange::ReduceGeneric(v) => {
                    reductions.push((self.eval_value(v, &ctx).max(0) as u32, None))
                }
                CostChange::ReduceColored(c, v) => {
                    reductions.push((self.eval_value(v, &ctx).max(0) as u32, Some(*c)))
                }
                CostChange::AdditionalCost(c) => add_cost(&mut cost, c),
                CostChange::AlternativeCost(_) => {}
            }
        }
        for (n, color) in reductions {
            if let Some(m) = cost.mana.as_mut() {
                match color {
                    None => m.reduce_generic(n),
                    Some(c) => {
                        for _ in 0..n {
                            if !m.reduce_colored(c) {
                                m.reduce_generic(1);
                            }
                        }
                    }
                }
            }
        }
        crate::keyword_impls::cost_reductions_from_keywords(self, p, card, chars, &mut cost, x);
        cost
    }

    // ------------------------------------------------------------------
    // Activated abilities (CR 602)
    // ------------------------------------------------------------------

    /// Activated abilities the player may activate now: (source, ability).
    pub fn activatable_abilities(&mut self, p: PlayerId) -> Vec<(ObjectId, Ability)> {
        let mut out = Vec::new();
        for id in self.live_objects() {
            let o = self.obj(id);
            for a in o.chars.abilities.clone() {
                let AbilityKind::Activated(act) = &a.kind else {
                    continue;
                };
                if self.can_activate(p, id, &a, act) {
                    out.push((id, a.clone()));
                }
            }
        }
        out
    }

    pub fn can_activate(
        &self,
        p: PlayerId,
        src: ObjectId,
        a: &Ability,
        act: &ActivatedAbility,
    ) -> bool {
        let o = self.obj(src);
        if !self.ability_functions(o, act.zone, false) {
            return false;
        }
        // Who can activate (CR 602.2): the controller (or owner for cards in hidden zones).
        let who = match o.zone {
            Zone::Battlefield | Zone::Stack => o.controller,
            _ => o.owner,
        };
        if who != p && !act.any_player {
            return false;
        }
        if self.turn.priority != Some(p) && !act.is_mana_ability {
            return false;
        }
        // Timing.
        let sorcery = act.timing == ActivationTiming::Sorcery || act.is_loyalty;
        if sorcery && !self.is_sorcery_timing(p) {
            return false;
        }
        match act.timing {
            ActivationTiming::YourUpkeep
                if !(self.turn.active == p && self.turn.step == crate::turn::Step::Upkeep) =>
            {
                return false
            }
            ActivationTiming::Combat if !self.turn.step.is_combat() => return false,
            ActivationTiming::YourTurn if self.turn.active != p => return false,
            ActivationTiming::OpponentsTurn if self.turn.active == p => return false,
            // CR 506.8, 506.8g: combat timing windows.
            ActivationTiming::CombatWindow(t) if !crate::combat::combat_timing_ok(self, t) => {
                return false
            }
            // CR 506.8b, 506.8d–e, 506.8g: "only before blockers are declared".
            ActivationTiming::BeforeBlockers
                if !crate::combat::combat_timing_ok(
                    self,
                    CombatTiming {
                        point: CombatPoint::BlockersDeclared,
                        after: false,
                        during_combat: false,
                    },
                ) =>
            {
                return false
            }
            _ => {}
        }
        // Loyalty abilities: once per turn per permanent (CR 606.3).
        if act.is_loyalty
            && o.activations_this_turn
                .iter()
                .any(|(uid, n)| *n > 0 && self.is_loyalty_uid(src, *uid))
        {
            return false;
        }
        if let Some(max) = act.max_per_turn {
            if o.activations_this_turn.get(&a.uid).copied().unwrap_or(0) >= max {
                return false;
            }
        }
        if let Some(c) = &act.condition {
            if !self.eval_cond(c, &Ctx::new(Some(src), p)) {
                return false;
            }
        }
        if self.activation_prohibited(p, src, act.is_mana_ability) {
            return false;
        }
        // Split second (CR 702.61b): only mana abilities.
        if self.split_second_on_stack() && !act.is_mana_ability {
            return false;
        }
        let ctx = Ctx::new(Some(src), p);
        if act.body.modal.is_none() && !self.targets_possible(&act.body.targets, &ctx, src) {
            return false;
        }
        let cost = self.ability_total_cost(p, src, a, act);
        self.can_pay_cost_optimistic(p, &cost, Some(src), &o.chars.clone())
    }

    fn is_loyalty_uid(&self, src: ObjectId, uid: u64) -> bool {
        self.obj(src)
            .chars
            .abilities
            .iter()
            .any(|a| a.uid == uid && matches!(&a.kind, AbilityKind::Activated(x) if x.is_loyalty))
    }

    fn activation_prohibited(&self, p: PlayerId, src: ObjectId, is_mana: bool) -> bool {
        let check = |r: &Restriction, s: Option<ObjectId>, c: PlayerId| -> bool {
            if let Restriction::CantActivate {
                who,
                sources,
                include_mana,
            } = r
            {
                let ctx = Ctx::new(s, c);
                (!is_mana || *include_mana)
                    && self.player_filter_matches(who, p, &ctx)
                    && self.matches(src, sources, &ctx)
            } else {
                false
            }
        };
        self.statics
            .restrictions
            .iter()
            .any(|(s, c, r)| check(r, Some(*s), *c))
            || self
                .rule_effects
                .iter()
                .any(|e| check(&e.restriction, e.source, e.controller))
    }

    /// Total cost of an activated ability including modifiers (CR 602.2b, 601.2f).
    pub fn ability_total_cost(
        &self,
        p: PlayerId,
        src: ObjectId,
        a: &Ability,
        act: &ActivatedAbility,
    ) -> Cost {
        let mut cost = act.cost.clone();
        for (s, ctl, cm) in &self.statics.cost_modifiers {
            let ctx = Ctx::new(Some(*s), *ctl);
            let applies = match &cm.applies_to {
                CostTarget::Abilities(f) => {
                    self.player_rel_matches(cm.who, p, &ctx) && self.matches(src, f, &ctx)
                }
                CostTarget::Keyword(k) => {
                    self.player_rel_matches(cm.who, p, &ctx)
                        && crate::keyword_impls::ability_from_keyword(a) == Some(*k)
                }
                _ => false,
            };
            if !applies {
                continue;
            }
            match &cm.change {
                CostChange::IncreaseGeneric(v) => {
                    let n = self.eval_value(v, &ctx).max(0) as u32;
                    add_cost(&mut cost, &Cost::mana(ManaCost::generic(n)));
                }
                CostChange::ReduceGeneric(v) => {
                    let n = self.eval_value(v, &ctx).max(0) as u32;
                    if let Some(m) = cost.mana.as_mut() {
                        m.reduce_generic(n);
                    }
                }
                CostChange::AdditionalCost(c) => add_cost(&mut cost, c),
                CostChange::IncreaseMana(m) => add_cost(&mut cost, &Cost::mana(m.clone())),
                _ => {}
            }
        }
        cost
    }

    /// Activates an ability (CR 602.2) or mana ability (CR 605.3).
    pub fn activate_ability(
        &mut self,
        p: PlayerId,
        src: ObjectId,
        uid: u64,
    ) -> Result<Option<ObjectId>, Illegal> {
        if self.dirty {
            self.recompute();
        }
        let a = self
            .obj(src)
            .chars
            .abilities
            .iter()
            .find(|a| a.uid == uid)
            .cloned()
            .ok_or_else(|| Illegal("no such ability".into()))?;
        let AbilityKind::Activated(act) = &a.kind else {
            return Err(Illegal("not an activated ability".into()));
        };
        let act = act.clone();
        if !self.can_activate(p, src, &a, &act) {
            return Err(Illegal("can't activate".into()));
        }
        let snapshot = self.clone();
        match self.activate_inner(p, src, &a, &act) {
            Ok(r) => Ok(r),
            Err(e) => {
                let agents = self.agents.clone();
                *self = snapshot;
                self.agents = agents;
                Err(e)
            }
        }
    }

    fn activate_inner(
        &mut self,
        p: PlayerId,
        src: ObjectId,
        a: &Ability,
        act: &ActivatedAbility,
    ) -> Result<Option<ObjectId>, Illegal> {
        let src_chars = self.obj(src).chars.clone();
        let mut ctx = Ctx::new(Some(src), p);
        ctx.ability_uid = a.uid;
        ctx.link = a.link;
        // X in the cost.
        let has_x = act.cost.mana.as_ref().is_some_and(|m| m.has_x())
            || act.cost.parts.iter().any(|c| cost_part_has_x(c));
        let mut x = 0i64;
        if has_x {
            let max = self.max_mana_available(p) as i64;
            x = match self.ask(p, Decision::ChooseX { source: src, max }) {
                Answer::Number(n) if n >= 0 => n,
                _ => 0,
            };
        }
        ctx.x = x as i32;
        if act.is_mana_ability {
            // CR 605.3: pay costs, then resolve immediately without using the stack.
            let mut cost = self.ability_total_cost(p, src, a, act);
            if let Some(m) = cost.mana.as_mut() {
                *m = m.with_x(x as u32);
            }
            let spend = SpendContext {
                is_ability: true,
                card_types: src_chars.card_types,
                source: Some(src),
                ..Default::default()
            };
            self.pay_total_cost(p, &cost, Some(src), &spend, &ctx)?;
            *self.objects[src.0 as usize]
                .activations_this_turn
                .entry(a.uid)
                .or_insert(0) += 1;
            self.emit(Event::AbilityActivated {
                ability: None,
                source: src,
                player: p,
                is_mana: true,
            });
            let tapped_for_mana = act.cost.has_tap();
            self.mana_ability_resolving = tapped_for_mana.then_some(src);
            let body = act.body.clone();
            self.exec(&body.effect, &mut ctx);
            self.mana_ability_resolving = None;
            self.flush_events();
            return Ok(None);
        }
        // 602.2a: create the ability on the stack.
        let id = crate::stack::create_stack_ability(
            self,
            src,
            p,
            a.clone(),
            StackKind::Activated {
                source: src,
                ability: a.clone(),
            },
            None,
            Some(Box::new(src_chars.clone())),
        );
        if let Some(si) = self.objects[id.0 as usize].stack.as_mut() {
            si.x = Some(x as i32);
        }
        // 602.2b: modes, targets.
        if !self.choose_modes_and_targets(id, &act.body, &mut ctx) {
            return Err(Illegal("no legal targets".into()));
        }
        // Costs.
        let mut cost = self.ability_total_cost(p, src, a, act);
        if let Some(m) = cost.mana.as_mut() {
            *m = m.with_x(x as u32);
        }
        let spend = SpendContext {
            is_ability: true,
            card_types: src_chars.card_types,
            source: Some(src),
            ..Default::default()
        };
        let paid = self.pay_total_cost(p, &cost, Some(src), &spend, &ctx)?;
        ctx.nums.insert(vars::USER + 90, paid.objects.len() as i64);
        if !paid.objects.is_empty() {
            ctx.vars.insert(
                vars::USER + 91,
                paid.objects.iter().map(|o| Entity::Object(*o)).collect(),
            );
        }
        self.saved_ctx.insert(id, ctx.clone());
        *self.objects[src.0 as usize]
            .activations_this_turn
            .entry(a.uid)
            .or_insert(0) += 1;
        // 602.2i: becomes activated.
        self.log(|g| format!("{p} activates {}", g.describe(src)));
        self.emit(Event::AbilityActivated {
            ability: Some(id),
            source: src,
            player: p,
            is_mana: false,
        });
        self.flush_events();
        Ok(Some(id))
    }

    // ------------------------------------------------------------------
    // Costs (CR 118)
    // ------------------------------------------------------------------

    /// Optimistic check that a cost could be paid (counts potential mana from untapped
    /// sources without solving colors exactly).
    pub fn can_pay_cost_optimistic(
        &self,
        p: PlayerId,
        cost: &Cost,
        src: Option<ObjectId>,
        _chars: &Characteristics,
    ) -> bool {
        let ctx = Ctx::new(src, p);
        for part in &cost.parts {
            if !self.cost_part_payable(p, part, src, &ctx) {
                return false;
            }
        }
        if let Some(m) = &cost.mana {
            let need = m.with_x(0);
            if need.mana_value() == 0
                && !need
                    .symbols
                    .iter()
                    .any(|s| matches!(s, ManaSymbol::Colorless | ManaSymbol::Colored(_)))
            {
                return true;
            }
            let plan =
                crate::mana_abilities::plan_payment(self, p, &need, &SpendContext::default(), src);
            return plan.is_some();
        }
        true
    }

    pub fn can_pay_cost(&self, p: PlayerId, cost: &Cost, src: Option<ObjectId>, ctx: &Ctx) -> bool {
        let chars = src.map(|s| self.obj(s).chars.clone()).unwrap_or_default();
        let _ = ctx;
        self.can_pay_cost_optimistic(p, cost, src, &chars)
    }

    /// Pays a cost during resolution ("you may pay ..."). Returns true if paid.
    pub fn pay_cost(&mut self, p: PlayerId, cost: &Cost, src: Option<ObjectId>, ctx: &Ctx) -> bool {
        let snapshot = self.clone();
        let spend = SpendContext {
            is_ability: true,
            source: src,
            ..Default::default()
        };
        match self.pay_total_cost(p, cost, src, &spend, ctx) {
            Ok(_) => true,
            Err(_) => {
                let agents = self.agents.clone();
                *self = snapshot;
                self.agents = agents;
                false
            }
        }
    }

    fn cost_part_payable(
        &self,
        p: PlayerId,
        part: &CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
    ) -> bool {
        let so = src.map(|s| self.obj(s));
        match part {
            CostPart::Tap | CostPart::Untap => {
                let Some(o) = so else { return false };
                if o.zone != Zone::Battlefield {
                    return false;
                }
                let want_tapped = matches!(part, CostPart::Untap);
                if o.tapped != want_tapped {
                    return false;
                }
                // CR 302.6: {T}/{Q} abilities of creatures need haste or no summoning sickness.
                !(o.is_creature() && o.summoning_sick && !o.has_keyword(KeywordKind::Haste))
            }
            CostPart::PayLife(v) => self.can_pay_life(p, self.eval_value(v, ctx).max(0) as u32),
            CostPart::Loyalty(n) => {
                let Some(o) = so else { return false };
                *n >= 0 || o.loyalty() >= -*n
            }
            CostPart::SacrificeSelf => {
                so.is_some_and(|o| o.zone == Zone::Battlefield && o.controller == p)
            }
            CostPart::Sacrifice { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| self.obj(*o).controller == p)
                    .count()
                    >= n
            }
            CostPart::DiscardSelf => so.is_some_and(|o| o.zone == Zone::Hand(p)),
            CostPart::Discard { filter, count, .. } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.player(p)
                    .hand
                    .iter()
                    .filter(|c| Some(**c) != src && self.matches(**c, filter, ctx))
                    .count()
                    >= n
            }
            CostPart::DiscardHand => true,
            CostPart::ExileSelf => so.is_some(),
            CostPart::Exile {
                filter,
                zone,
                count,
            } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.cost_zone_cards(p, *zone)
                    .into_iter()
                    .filter(|c| Some(*c) != src && self.matches(*c, filter, ctx))
                    .count()
                    >= n
            }
            CostPart::ReturnToHand { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| self.obj(*o).controller == p)
                    .count()
                    >= n
            }
            CostPart::ReturnSelfToHand => so.is_some_and(|o| o.zone == Zone::Battlefield),
            CostPart::RemoveCounters { kind, count } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                so.is_some_and(|o| o.counter(kind) >= n)
            }
            CostPart::RemoveCountersFromAmong {
                kind,
                filter,
                count,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let total: u32 = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .map(|o| match kind {
                        Some(k) => self.obj(o).counter(k),
                        None => self.obj(o).counters.values().sum(),
                    })
                    .sum();
                total >= n
            }
            CostPart::AddCounters { .. } => so.is_some(),
            CostPart::TapUntapped { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| {
                        let ob = self.obj(*o);
                        ob.controller == p
                            && !ob.tapped
                            && !(ob.is_creature()
                                && ob.summoning_sick
                                && !ob.has_keyword(KeywordKind::Haste)
                                && Some(*o) == src)
                    })
                    .count()
                    >= n
            }
            CostPart::UntapTapped { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| self.obj(*o).controller == p && self.obj(*o).tapped)
                    .count()
                    >= n
            }
            CostPart::PayEnergy(v) => {
                self.player(p).counter(counters::ENERGY) as i64 >= self.eval_value(v, ctx)
            }
            CostPart::PayPlayerCounters { kind, count } => {
                self.player(p).counter(kind) as i64 >= self.eval_value(count, ctx)
            }
            CostPart::Mill(v) => self.player(p).library.len() as i64 >= self.eval_value(v, ctx),
            CostPart::RevealFromHand { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.player(p)
                    .hand
                    .iter()
                    .filter(|c| Some(**c) != src && self.matches(**c, filter, ctx))
                    .count()
                    >= n
            }
            CostPart::ExertSelf => so.is_some(),
            CostPart::CollectEvidence(n) => {
                let total: u32 = self
                    .player(p)
                    .graveyard
                    .iter()
                    .map(|c| self.mana_value_of(*c))
                    .sum();
                total >= *n
            }
            CostPart::Forage => {
                self.player(p).graveyard.len() >= 3
                    || self
                        .permanents()
                        .any(|o| o.controller == p && o.chars.has_subtype("Food"))
            }
            CostPart::PutFromHandOnLibrary { filter, count, .. } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.player(p)
                    .hand
                    .iter()
                    .filter(|c| Some(**c) != src && self.matches(**c, filter, ctx))
                    .count()
                    >= n
            }
            CostPart::Effect(_) => true,
        }
    }

    fn cost_zone_cards(&self, p: PlayerId, zone: ZoneKind) -> Vec<ObjectId> {
        match zone {
            ZoneKind::Graveyard => self.player(p).graveyard.clone(),
            ZoneKind::Hand => self.player(p).hand.clone(),
            ZoneKind::Library => self.player(p).library.clone(),
            ZoneKind::Battlefield => self.permanents_controlled_by(p),
            ZoneKind::Exile => self
                .exile
                .iter()
                .copied()
                .filter(|c| self.obj(*c).owner == p)
                .collect(),
            _ => vec![],
        }
    }

    /// Pays a total cost: mana (activating mana abilities as needed) and all other parts
    /// (CR 601.2g–h). Errors leave partial state; callers roll back.
    pub fn pay_total_cost(
        &mut self,
        p: PlayerId,
        cost: &Cost,
        src: Option<ObjectId>,
        spend: &SpendContext,
        ctx: &Ctx,
    ) -> Result<PaidCost, Illegal> {
        let mut paid = PaidCost::default();
        // Check all parts are payable before paying anything.
        for part in &cost.parts {
            if !self.cost_part_payable(p, part, src, ctx) {
                return Err(Illegal(format!("can't pay {part:?}")));
            }
        }
        // Mana first (mana abilities must be activated before costs are paid, 601.2g),
        // but tapping the source for {T} must not be used for mana: reserve it.
        if let Some(m) = &cost.mana {
            let reserve = if cost.has_tap() { src } else { None };
            let spent = crate::mana_abilities::pay_mana(self, p, m, spend, reserve)
                .ok_or_else(|| Illegal("can't pay mana".into()))?;
            self.players[p.idx()].mana_spent_this_turn += spent.len() as u32;
            paid.mana_spent = spent;
        }
        for part in &cost.parts {
            self.pay_cost_part(p, part, src, ctx, &mut paid)?;
        }
        Ok(paid)
    }

    fn pay_cost_part(
        &mut self,
        p: PlayerId,
        part: &CostPart,
        src: Option<ObjectId>,
        ctx: &Ctx,
        paid: &mut PaidCost,
    ) -> Result<(), Illegal> {
        let bad = |s: &str| Err(Illegal(s.to_string()));
        match part {
            CostPart::Tap => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                if !self.tap(s) {
                    return bad("can't tap");
                }
            }
            CostPart::Untap => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                if !self.untap(s) {
                    return bad("can't untap");
                }
            }
            CostPart::PayLife(v) => {
                let n = self.eval_value(v, ctx).max(0) as u32;
                if !self.pay_life(p, n) {
                    return bad("can't pay life");
                }
            }
            CostPart::Loyalty(n) => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                if *n > 0 {
                    self.add_counters(Entity::Object(s), counters::LOYALTY, *n as u32, Some(s));
                } else if *n < 0
                    && self.remove_counters(Entity::Object(s), counters::LOYALTY, (-*n) as u32)
                        < (-*n) as u32
                {
                    return bad("not enough loyalty");
                }
            }
            CostPart::SacrificeSelf => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                paid.objects.push(s);
                if self.sacrifice(s, p).is_none() && self.is_live(s) {
                    return bad("can't sacrifice");
                }
            }
            CostPart::Sacrifice { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let cands: Vec<ObjectId> = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| self.obj(*o).controller == p && !self.cant_be_sacrificed(*o))
                    .collect();
                if (cands.len() as u32) < n {
                    return bad("not enough to sacrifice");
                }
                let pick =
                    self.ask_objects(p, src, "Choose permanents to sacrifice (cost)", cands, n, n);
                for o in pick {
                    paid.objects.push(o);
                    self.sacrifice(o, p);
                }
            }
            CostPart::DiscardSelf => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                paid.objects.push(s);
                if self.discard(p, s, src).is_none() {
                    return bad("can't discard");
                }
            }
            CostPart::Discard {
                filter,
                count,
                random,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let cands: Vec<ObjectId> = self
                    .player(p)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| Some(*c) != src && self.matches(*c, filter, ctx))
                    .collect();
                if (cands.len() as u32) < n {
                    return bad("not enough cards");
                }
                let pick = if *random {
                    use rand::seq::SliceRandom;
                    let mut c = cands.clone();
                    c.shuffle(&mut self.rng);
                    c.into_iter().take(n as usize).collect()
                } else {
                    self.ask_objects(p, src, "Choose cards to discard (cost)", cands, n, n)
                };
                for c in pick {
                    paid.objects.push(c);
                    self.discard(p, c, src);
                }
            }
            CostPart::DiscardHand => {
                for c in self.player(p).hand.clone() {
                    paid.objects.push(c);
                    self.discard(p, c, src);
                }
            }
            CostPart::ExileSelf => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                paid.objects.push(s);
                self.exile_object(s, src);
            }
            CostPart::Exile {
                filter,
                zone,
                count,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let cands: Vec<ObjectId> = self
                    .cost_zone_cards(p, *zone)
                    .into_iter()
                    .filter(|c| Some(*c) != src && self.matches(*c, filter, ctx))
                    .collect();
                if (cands.len() as u32) < n {
                    return bad("not enough cards to exile");
                }
                let pick = self.ask_objects(p, src, "Choose cards to exile (cost)", cands, n, n);
                for c in pick {
                    paid.objects.push(c);
                    self.exile_object(c, src);
                }
            }
            CostPart::ReturnToHand { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let cands: Vec<ObjectId> = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| self.obj(*o).controller == p)
                    .collect();
                let pick =
                    self.ask_objects(p, src, "Choose permanents to return (cost)", cands, n, n);
                for o in pick {
                    let owner = self.obj(o).owner;
                    paid.objects.push(o);
                    self.move_object(o, Zone::Hand(owner), MoveCause::Cost, Some(p));
                }
            }
            CostPart::ReturnSelfToHand => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                let owner = self.obj(s).owner;
                self.move_object(s, Zone::Hand(owner), MoveCause::Cost, Some(p));
            }
            CostPart::RemoveCounters { kind, count } => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                let n = self.eval_value(count, ctx).max(0) as u32;
                if self.remove_counters(Entity::Object(s), kind, n) < n {
                    return bad("not enough counters");
                }
            }
            CostPart::RemoveCountersFromAmong {
                kind,
                filter,
                count,
            } => {
                let mut n = self.eval_value(count, ctx).max(0) as u32;
                for o in self.objects_matching(filter, ctx) {
                    if n == 0 {
                        break;
                    }
                    let kinds: Vec<CounterKind> = match kind {
                        Some(k) => vec![k.clone()],
                        None => self.obj(o).counters.keys().cloned().collect(),
                    };
                    for k in kinds {
                        let r = self.remove_counters(Entity::Object(o), &k, n);
                        n -= r;
                    }
                }
                if n > 0 {
                    return bad("not enough counters");
                }
            }
            CostPart::AddCounters { kind, count } => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                let n = self.eval_value(count, ctx).max(0) as u32;
                self.add_counters(Entity::Object(s), kind, n, src);
            }
            CostPart::TapUntapped { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let cands: Vec<ObjectId> = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| self.obj(*o).controller == p && !self.obj(*o).tapped)
                    .collect();
                if (cands.len() as u32) < n {
                    return bad("not enough untapped permanents");
                }
                let pick = self.ask_objects(p, src, "Choose permanents to tap (cost)", cands, n, n);
                for o in pick {
                    paid.objects.push(o);
                    self.tap(o);
                }
            }
            CostPart::UntapTapped { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let cands: Vec<ObjectId> = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| self.obj(*o).controller == p && self.obj(*o).tapped)
                    .collect();
                let pick =
                    self.ask_objects(p, src, "Choose permanents to untap (cost)", cands, n, n);
                for o in pick {
                    self.untap(o);
                }
            }
            CostPart::PayEnergy(v) => {
                let n = self.eval_value(v, ctx).max(0) as u32;
                if self.remove_counters(Entity::Player(p), counters::ENERGY, n) < n {
                    return bad("not enough energy");
                }
            }
            CostPart::PayPlayerCounters { kind, count } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                if self.remove_counters(Entity::Player(p), kind, n) < n {
                    return bad("not enough counters");
                }
            }
            CostPart::Mill(v) => {
                let n = self.eval_value(v, ctx).max(0) as u32;
                self.mill(p, n);
            }
            CostPart::RevealFromHand { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let cands: Vec<ObjectId> = self
                    .player(p)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| Some(*c) != src && self.matches(*c, filter, ctx))
                    .collect();
                let pick = self.ask_objects(p, src, "Choose cards to reveal (cost)", cands, n, n);
                paid.objects.extend(pick);
            }
            CostPart::ExertSelf => {
                if let Some(s) = src {
                    self.objects[s.0 as usize].exerted = true;
                }
            }
            CostPart::CollectEvidence(n) => {
                if !crate::keyword_actions::collect_evidence(self, p, *n, src) {
                    return bad("can't collect evidence");
                }
            }
            CostPart::Forage => {
                if !crate::keyword_actions::forage(self, p, src) {
                    return bad("can't forage");
                }
            }
            CostPart::PutFromHandOnLibrary { filter, count, top } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let cands: Vec<ObjectId> = self
                    .player(p)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| Some(*c) != src && self.matches(*c, filter, ctx))
                    .collect();
                let pick = self.ask_objects(
                    p,
                    src,
                    "Choose cards to put on your library (cost)",
                    cands,
                    n,
                    n,
                );
                for c in pick {
                    self.move_object_ev(MoveEv {
                        obj: c,
                        to: Zone::Library(p),
                        pos: if *top {
                            LibraryPosition::Top
                        } else {
                            LibraryPosition::Bottom
                        },
                        cause: MoveCause::Cost,
                        by: Some(p),
                        etb: EtbInfo::default(),
                        source: src,
                    });
                }
            }
            CostPart::Effect(e) => {
                let mut c = ctx.clone();
                self.exec(e, &mut c);
            }
        }
        Ok(())
    }

    /// Rough upper bound of mana the player could produce now (for choosing X).
    pub fn max_mana_available(&self, p: PlayerId) -> u32 {
        self.player(p).mana_pool.total() as u32
            + crate::mana_abilities::potential_mana_count(self, p, None)
    }
}

/// What was paid for a cost.
#[derive(Clone, Debug, Default)]
pub struct PaidCost {
    pub mana_spent: Vec<ManaType>,
    pub objects: Vec<ObjectId>,
}

/// Adds one cost to another.
pub fn add_cost(total: &mut Cost, c: &Cost) {
    if let Some(m) = &c.mana {
        match total.mana.as_mut() {
            Some(t) => t.add(m),
            None => total.mana = Some(m.clone()),
        }
    }
    total.parts.extend(c.parts.iter().cloned());
}

fn cost_part_has_x(c: &CostPart) -> bool {
    let is_x = |v: &Value| matches!(v, Value::X);
    match c {
        CostPart::PayLife(v) | CostPart::PayEnergy(v) | CostPart::Mill(v) => is_x(v),
        CostPart::Sacrifice { count, .. }
        | CostPart::Discard { count, .. }
        | CostPart::Exile { count, .. }
        | CostPart::RemoveCounters { count, .. }
        | CostPart::RemoveCountersFromAmong { count, .. }
        | CostPart::TapUntapped { count, .. } => is_x(count),
        CostPart::Loyalty(_) => false,
        _ => false,
    }
}

#[allow(dead_code)]
fn _unused(_: SpecialAction) {}
