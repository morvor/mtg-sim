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
        // With shared team turns each player on the active team may (CR 805.4c).
        self.is_active_player(p)
            && self.turn.step.is_main()
            && self.stack.is_empty()
            && self.has_priority(p)
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
            if self.card_has_land_face(c)
                && !out.contains(&c)
                && self.permission_allows(p, c, &self.land_face_characteristics(c), true)
            {
                out.push(c);
            }
        }
        out
    }

    /// Characteristics of the face a card would be played with as a land.
    fn land_face_characteristics(&self, c: ObjectId) -> Characteristics {
        let o = self.obj(c);
        if o.chars.is_land() {
            o.chars.clone()
        } else {
            self.face_characteristics(c, FaceState::Back)
        }
    }

    /// Whether a rule or effect allows player `p` to play `card` from where it is, as a
    /// land (`land`) or as a spell with the characteristics `chars` it would have
    /// (CR 601.3, 601.3e, 305.1). Cards in the player's hand are always allowed.
    pub fn permission_allows(
        &self,
        p: PlayerId,
        card: ObjectId,
        chars: &Characteristics,
        land: bool,
    ) -> bool {
        let o = self.obj(card);
        if o.zone == Zone::Hand(p) {
            return true;
        }
        // Grants from resolved effects name specific cards ("you may play that card").
        if self
            .play_grants
            .iter()
            .any(|g| g.player == p && g.object == card)
        {
            return true;
        }
        // CR 722.3c: a prepared permanent's controller may cast its prepare-spell copy.
        if !land && crate::designations::castable_prepared_copies(self, p).contains(&card) {
            return true;
        }
        // CR 601.3f: a face-down card in exile can be cast only by a player who may look
        // at it; permissions to cast spells "with certain qualities" don't reveal it.
        if o.zone == Zone::Exile && o.face_down {
            return false;
        }
        for (src, ctl, perm) in &self.statics.play_permissions {
            if (land && !perm.lands) || (!land && !perm.spells) {
                continue;
            }
            let ctx = Ctx::new(Some(*src), *ctl);
            if !self.player_rel_matches(perm.who, p, &ctx) {
                continue;
            }
            let in_zone = match perm.zone {
                ZoneKind::Library => {
                    if perm.top_only {
                        self.library_top(p) == Some(card)
                    } else {
                        o.zone == Zone::Library(p)
                    }
                }
                ZoneKind::Graveyard => o.zone == Zone::Graveyard(p),
                ZoneKind::Exile => o.zone == Zone::Exile,
                ZoneKind::Hand => o.zone == Zone::Hand(p),
                ZoneKind::Command => o.zone == Zone::Command,
                _ => false,
            };
            if !in_zone {
                continue;
            }
            // CR 601.3e: the alternative characteristics the card would have as it's
            // played are what the permission checks.
            let f = if land {
                perm.what.clone()
            } else {
                as_spell_filter(&perm.what)
            };
            let view = WithChars { id: card, chars };
            if self.matches_view(&view, card, &f, &ctx) {
                return true;
            }
        }
        false
    }

    fn card_has_land_face(&self, c: ObjectId) -> bool {
        let o = self.obj(c);
        o.chars.is_land()
            || o.card.as_ref().is_some_and(|d| {
                d.layout == crate::card::Layout::ModalDfc
                    && d.faces.get(1).is_some_and(|f| f.chars.is_land())
            })
    }

    /// Cards in zones other than hand that the player may be able to play because of
    /// static permissions or grants. Whether a particular way of playing the card is
    /// permitted is decided by [`Game::permission_allows`] (CR 601.3e: the permission
    /// looks at the characteristics the card would have as it's played).
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
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        }
        for gnt in &self.play_grants {
            if gnt.player == p && self.is_live(gnt.object) && !out.contains(&gnt.object) {
                out.push(gnt.object);
            }
        }
        // CR 722.3c: a prepared permanent's controller may cast its prepare-spell copy.
        for c in crate::designations::castable_prepared_copies(self, p) {
            if !out.contains(&c) {
                out.push(c);
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
        // CR 609.4: "as though those cards were in your graveyard".
        for c in crate::as_though::other_graveyard_cards(self, p) {
            if !out.contains(&c) {
                out.push(c);
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
                Some(crate::card::Layout::Adventure) => {
                    push_face(FaceState::Front, &mut out);
                    push_face(FaceState::Half(1), &mut out);
                }
                // CR 722.3: a preparation card can't be cast as its prepare spell; only
                // the copy created as it becomes prepared can.
                Some(crate::card::Layout::Prepare) => push_face(FaceState::Front, &mut out),
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
                        match (&cm.applies_to, &cm.change) {
                            (CostTarget::ThisSpell, CostChange::AlternativeCost(c)) => {
                                let mut opt = CastOption::normal(FaceState::Front);
                                opt.method = CastMethod::Alternative(a.uid);
                                opt.alt_cost = Some(c.clone());
                                out.push(opt);
                            }
                            // CR 601.3c: "You may cast this spell as though it had flash
                            // if you pay [cost] more to cast it."
                            (CostTarget::ThisSpell, CostChange::FlashForAdditionalCost(c)) => {
                                let mut opt = CastOption::normal(FaceState::Front);
                                opt.method = CastMethod::Alternative(a.uid);
                                opt.extra_cost = Some(c.clone());
                                opt.flash = true;
                                out.push(opt);
                            }
                            _ => {}
                        }
                    }
                }
            }
            // CR 601.3, 601.3e: outside the hand, each way of casting the card must be
            // permitted by a rule or effect given the characteristics it would have.
            if !in_hand {
                out.retain(|opt| {
                    let chars = self.option_characteristics(card, opt);
                    self.permission_allows(p, card, &chars, false)
                });
            }
        }
        out.extend(crate::keyword_impls::keyword_cast_options(self, p, card));
        // Lands can't be cast (CR 305.9).
        out.retain(|opt| {
            let chars = self.option_characteristics(card, opt);
            !chars.is_land()
        });
        out
    }

    /// CR 400.7g: if an effect granted `card` the keyword ability it's being cast with
    /// ("target card gains flashback", "each instant and sorcery card in your graveyard has
    /// flashback"), that ability continues to apply to the spell `spell` it became, even
    /// though the effect no longer applies to that new object.
    fn keep_granted_casting_keyword(&mut self, card: ObjectId, spell: ObjectId, m: &CastMethod) {
        let CastMethod::Keyword(k) = *m else {
            return;
        };
        let o = self.obj(spell);
        if o.chars.has_keyword(k) || o.base.keywords().any(|x| x.kind == k) {
            return;
        }
        let controller = o.controller;
        let Some(kw) = self
            .obj(card)
            .chars
            .keywords()
            .find(|x| x.kind == k)
            .cloned()
        else {
            return;
        };
        let eid = self.new_effect_id();
        let timestamp = self.new_timestamp();
        self.effects.push(ContinuousEffect {
            id: eid,
            source: Some(spell),
            controller,
            timestamp,
            duration: Duration::Permanent,
            affected: Affected::Objects(vec![spell]),
            mods: vec![Modification::AddKeyword(kw)],
            layer1: None,
            created_turn: self.turn.number,
        });
        self.recompute();
    }

    /// Characteristics a card would have as a spell cast this way (CR 601.3e): those of
    /// the chosen face, or a face-down spell's (CR 702.37c, 708.2a).
    pub fn option_characteristics(&self, card: ObjectId, opt: &CastOption) -> Characteristics {
        if let CastMethod::FaceDown(k) = opt.method {
            // The characteristics the ability it's cast with lists (e.g. disguise's ward
            // {2}, CR 708.4).
            return crate::facedown::face_down_spell_characteristics(k);
        }
        self.face_characteristics(card, opt.face)
    }

    /// Characteristics a card would have as a spell cast with the given face (CR 601.3e).
    pub fn face_characteristics(&self, card: ObjectId, face: FaceState) -> Characteristics {
        let o = self.obj(card);
        match (&o.card, face) {
            (Some(d), FaceState::Front) if d.layout == crate::card::Layout::Split => {
                d.characteristics(FaceState::Front)
            }
            (Some(d), f) if f != FaceState::Front => d.characteristics(f),
            // A face-down card outside the battlefield (e.g. foretold) is cast face up
            // (CR 702.143a).
            (Some(d), f) if o.face_down && o.zone != Zone::Battlefield => d.characteristics(f),
            _ => o.chars.clone(),
        }
    }

    /// Whether the player could begin casting the card this way right now (timing,
    /// permissions, targets, and an optimistic cost check).
    pub fn can_begin_cast(&self, p: PlayerId, card: ObjectId, opt: &CastOption) -> bool {
        let chars = self.option_characteristics(card, opt);
        if !opt.any_time && !self.timing_allows_cast(p, card, &chars, opt) {
            return false;
        }
        if self.cast_prohibited(p, card, &chars) && !proposal_may_change_qualities(&chars) {
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
        // Optimistic cost check, with the keywords the spell would be given as it's cast.
        let chars = crate::kw::with_granted_spell_keywords(self, p, card, &chars);
        let mut cost = self.base_total_cost(p, card, &chars, opt, 0);
        crate::cost_rules::spend_any_type(self, p, card, &mut cost);
        crate::kw::payable_otherwise(self, p, card, &chars, &opt.method, &mut cost);
        self.can_pay_cost_optimistic(p, &cost, Some(card), &chars)
    }

    pub(crate) fn timing_allows_cast(
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
        if !self.has_priority(p) {
            return false;
        }
        // CR 601.3d: a spell that has flash only while a condition is met can be cast as
        // though it had flash while the condition is met (its characteristics include flash).
        let instant_speed = chars.is(CardType::Instant)
            || chars.has_keyword(KeywordKind::Flash)
            || opt.flash
            || self.flash_permitted(p, card, chars);
        if instant_speed {
            return !self.player_restricted(p, |r| matches!(r, Restriction::SorcerySpeedOnly(_)))
                || self.is_sorcery_timing(p);
        }
        self.is_sorcery_timing(p)
    }

    /// Whether an effect lets `p` cast `card` as though it had flash, judged by the
    /// characteristics the spell would have given the choices made in its proposal
    /// (CR 601.3b, 601.3e).
    fn flash_permitted(&self, p: PlayerId, card: ObjectId, chars: &Characteristics) -> bool {
        let view = WithChars { id: card, chars };
        self.statics
            .flash_permissions
            .iter()
            .any(|(s, c, who, what)| {
                let ctx = Ctx::new(Some(*s), *c);
                self.player_rel_matches(*who, p, &ctx)
                    && self.matches_view(&view, card, &as_spell_filter(what), &ctx)
            })
    }

    pub(crate) fn cast_prohibited(
        &self,
        p: PlayerId,
        card: ObjectId,
        chars: &Characteristics,
    ) -> bool {
        self.cast_prohibited_by_effects(p, card, chars)
            // CR 702.61a split second: no casting while a split-second spell is on the stack.
            || self.split_second_on_stack()
    }

    /// Rules and effects that prohibit casting a spell with the given characteristics
    /// ("can't cast", "can't cast more than one spell each turn"), CR 601.3.
    fn cast_prohibited_by_effects(
        &self,
        p: PlayerId,
        card: ObjectId,
        chars: &Characteristics,
    ) -> bool {
        let spells_cast = self
            .history
            .spells_cast
            .iter()
            .filter(|(q, _)| *q == p)
            .count() as u32;
        let view = WithChars { id: card, chars };
        let check = |r: &Restriction, s: Option<ObjectId>, c: PlayerId| -> bool {
            let ctx = Ctx::new(s, c);
            match r {
                Restriction::CantCast { who, what } => {
                    self.player_filter_matches(who, p, &ctx)
                        && self.matches_view(&view, card, &as_spell_filter(what), &ctx)
                }
                Restriction::MaxSpellsPerTurn(who, n) => {
                    self.player_filter_matches(who, p, &ctx) && spells_cast >= *n
                }
                _ => false,
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
        let chars = self.option_characteristics(card, &opt);
        if !opt.any_time && !self.timing_allows_cast(p, card, &chars, &opt) {
            return Err(Illegal("timing".into()));
        }
        // CR 601.3a: if choices made while proposing the spell (such as the value of X)
        // could change the qualities a prohibition looks at, the player may begin to cast
        // it; the prohibition is checked again once the proposal is complete (CR 601.2e).
        if self.cast_prohibited(p, card, &chars) && !proposal_may_change_qualities(&chars) {
            return Err(Illegal("prohibited".into()));
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
        self.special.casting += 1;
        match self.cast_inner(p, card, &opt) {
            Ok(id) => {
                // CR 601.2i: the spell became cast.
                crate::draw_rules::finish_casting(self);
                Ok(id)
            }
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
        // Abilities that trigger when a card leaves a graveyard look back (CR 603.10a).
        let lookback = matches!(from, Zone::Graveyard(_))
            .then(|| std::sync::Arc::new(self.lookback_snapshot()));
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
            lookback,
        });
        // CR 601.2a: effects that apply to the spell as it's cast begin now (CR 610.5,
        // 611.2f).
        crate::next_spell::spell_put_on_stack(self, id, p);
        self.recompute();
        self.keep_granted_casting_keyword(card, id, &opt.method);
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
                    // CR 702.33c–d: a multikicker cost is a kicker cost; paying it kicks
                    // the spell.
                    if name.as_str() == "multikicker" {
                        cast_info.paid.push("kicker".into());
                    }
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
                    // CR 607.2i: abilities linked to a specific kicker cost refer to it.
                    if let Some(m) = &cost.mana {
                        cast_info.paid.push(format!("kicker {m}").into());
                    }
                }
            }
        }
        // CR 702.33d: a spell whose controller declared the intention to pay any of its
        // kicker costs (sticker kicker included, CR 702.33h) has been kicked.
        crate::kw::kicker::record_kicked(&mut cast_info.paid);
        // (Additional costs required by the casting method itself are part of
        // `base_total_cost`.)
        // CR 702.47a: splice (the spell gains text, CR 612.10). Splice affordability
        // accounts for every additional cost chosen so far, including the casting
        // method's own.
        let mut committed = extra.clone();
        if let Some(c) = &opt.extra_cost {
            add_cost(&mut committed, c);
        }
        for c in crate::splice::offer_splices(self, p, id, &committed) {
            add_cost(&mut extra, &c);
        }
        let base_cost_has_x = match &opt.alt_cost {
            Some(c) => c.mana.as_ref().is_some_and(|m| m.has_x()),
            None => chars.mana_cost.as_ref().is_some_and(|m| m.has_x()),
        } || extra.mana.as_ref().is_some_and(|m| m.has_x())
            || extra.parts.iter().any(cost_part_has_x)
            // A variable additional cost ("As an additional cost to cast this spell, pay X
            // life", CR 601.2b, 607.2j).
            || chars.abilities.iter().any(|a| match &a.kind {
                AbilityKind::Static(s) => match &s.effect {
                    StaticEffect::CostModifier(CostModifier {
                        applies_to: CostTarget::ThisSpell,
                        change: CostChange::AdditionalCost(c),
                        ..
                    }) => {
                        c.mana.as_ref().is_some_and(|m| m.has_x())
                            || c.parts.iter().any(cost_part_has_x)
                    }
                    _ => false,
                },
                _ => false,
            })
            || opt
                .extra_cost
                .as_ref()
                .is_some_and(|c| c.mana.as_ref().is_some_and(|m| m.has_x()));
        let mut x: i64 = 0;
        if base_cost_has_x {
            let max = self.max_mana_available(p) as i64;
            x = match self.ask(p, Decision::ChooseX { source: id, max }) {
                Answer::Number(n) if n >= 0 => n,
                _ => max.max(0),
            };
        }
        // The spell's cast info records X only if a value was chosen for one of its costs:
        // that's the X its permanent's enters abilities use (CR 107.3m).
        cast_info.x = base_cost_has_x.then_some(x as i32);
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

        // 601.2e: the game checks whether the proposed spell can legally be cast. A
        // prohibition that applies to the spell as proposed (e.g. to the mana value it has
        // with the chosen X) makes the casting illegal (CR 601.3a, 601.6).
        let proposed = self.obj(id).chars.clone();
        if self.cast_prohibited_by_effects(p, id, &proposed) {
            return Err(Illegal("the proposed spell can't be cast".into()));
        }

        // 601.2f: total cost. The player chooses halves of hybrid symbols by which the
        // cost is reduced (CR 118.7e).
        crate::cost_rules::choose_reduction_halves(self, p, id);
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
        // CR 118.14: mana of any type may be spent to cast it.
        crate::cost_rules::spend_any_type(self, p, id, &mut total);
        // CR 118.13a: how symbols that can be paid in more than one way will be paid.
        crate::cost_rules::choose_payment_ways(self, p, Some(id), &mut total);
        // CR 702.51a–b: once the total cost is determined, keywords such as convoke may
        // pay part of it other than with mana.
        crate::kw::pay_mana_otherwise(self, p, id, &mut total)?;
        // 601.2g–h: activate mana abilities and pay.
        let spend = SpendContext {
            is_spell: true,
            is_ability: false,
            card_types: chars.card_types,
            subtypes: chars.subtypes.to_vec(),
            has_x: base_cost_has_x,
            source: Some(id),
            any_color: self.any_color_mana(p, id, false),
        };
        let paid = self.pay_total_cost(p, &total, Some(id), &spend, &ctx)?;
        if let Some(si) = self.objects[id.0 as usize].stack.as_mut() {
            si.cast.mana_spent = paid.mana_spent.clone();
            si.cast.cost_objects = paid.objects.clone();
        }
        // "The sacrificed creature" (resolution reads the spell's saved context).
        if !paid.sacrificed.is_empty() {
            self.saved_ctx.entry(id).or_default().vars.insert(
                vars::SACRIFICED,
                paid.sacrificed.iter().map(|o| Entity::Object(*o)).collect(),
            );
        }
        // CR 700.14: the player expends N for each N reached by this payment.
        let spent = paid.mana_spent.len() as u32;
        if spent > 0 {
            let total = self.history.spell_mana_spent.entry(p).or_insert(0);
            let before = *total;
            *total += spent;
            for n in before + 1..=before + spent {
                self.emit(Event::Custom {
                    name: "expend".into(),
                    player: Some(p),
                    obj: None,
                    amount: n as i32,
                });
            }
        }
        if matches!(from, Zone::Command) && self.obj(id).is_commander {
            *self.players[p.idx()]
                .commander_casts
                .entry(chars.name.clone())
                .or_insert(0) += 1;
        }
        // 601.2i: the spell becomes cast. A prepared permanent whose prepare-spell copy
        // this is loses the designation now (CR 722.3c).
        if from == Zone::Exile && self.obj(card).kind == ObjKind::CardCopy {
            crate::designations::prepared_copy_left_exile(self, card);
        }
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
                // CR 118.6: an object with no mana cost has an unpayable cost.
                mana: Some(
                    chars
                        .mana_cost
                        .clone()
                        .unwrap_or_else(crate::cost_rules::unpayable),
                ),
                parts: vec![],
            },
        };
        // Reductions by mana symbols (CR 118.7a–g), applied after the other changes.
        let mut mana_reductions: Vec<(ManaCost, bool)> = Vec::new();
        // Additional costs required by the casting method (e.g. CR 601.3c).
        if let Some(e) = &opt.extra_cost {
            add_cost(&mut cost, e);
        }
        // X has its announced value before cost reductions apply (CR 601.2f, 107.3b).
        if let Some(m) = cost.mana.as_mut() {
            *m = m.with_x(x);
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
                            CostChange::ReduceMana { mana, colored_only } => {
                                mana_reductions.push((mana.clone(), *colored_only))
                            }
                            CostChange::AlternativeCost(_)
                            | CostChange::FlashForAdditionalCost(_) => {}
                        }
                    }
                }
            }
        }
        // Static cost modifiers from other permanents: increases first, then reductions.
        let mut reductions: Vec<(u32, Option<Color>)> = Vec::new();
        for (src, ctl, cm) in &self.statics.cost_modifiers {
            let ctx = Ctx::new(Some(*src), *ctl);
            // (A card being considered for casting is judged as the spell it would be.)
            let applies = match &cm.applies_to {
                CostTarget::Spells(f) => {
                    self.player_rel_matches(cm.who, p, &ctx)
                        && self.matches(card, &as_spell_filter(f), &ctx)
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
                CostChange::ReduceMana { mana, colored_only } => {
                    mana_reductions.push((mana.clone(), *colored_only))
                }
                CostChange::AdditionalCost(c) => add_cost(&mut cost, c),
                CostChange::AlternativeCost(_) | CostChange::FlashForAdditionalCost(_) => {}
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
        let mut hybrid = 0;
        for (by, colored_only) in mana_reductions {
            if let Some(m) = cost.mana.as_mut() {
                crate::cost_rules::reduce_by(m, &by, colored_only, |_, cur, s| {
                    let h = crate::cost_rules::chosen_half(self, card, hybrid, cur, s);
                    hybrid += 1;
                    h
                });
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
        // CR 602.2, 605.3a: abilities are activated by a player with priority; mana
        // abilities also while a mana payment is being made (casting, activating, or an
        // effect asking for a payment).
        if !self.has_priority(p) && !(act.is_mana_ability && self.mana_hint.is_some()) {
            return false;
        }
        // Timing.
        let sorcery = act.timing == ActivationTiming::Sorcery || act.is_loyalty;
        if sorcery && !self.is_sorcery_timing(p) {
            return false;
        }
        match act.timing {
            ActivationTiming::YourUpkeep
                if !(self.is_active_player(p) && self.turn.step == crate::turn::Step::Upkeep) =>
            {
                return false
            }
            ActivationTiming::Combat if !self.turn.step.is_combat() => return false,
            ActivationTiming::YourTurn if !self.is_active_player(p) => return false,
            ActivationTiming::OpponentsTurn if self.is_active_player(p) => return false,
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
        if self.activation_prohibited(p, src, act.is_mana_ability)
            || !crate::kw::activation_allowed(self, p, src, a)
        {
            return false;
        }
        // Split second (CR 702.61b): only mana abilities.
        if self.split_second_on_stack() && !act.is_mana_ability {
            return false;
        }
        let ctx = Ctx::new(Some(src), p);
        if act.body.modal.is_none() && !self.targets_possible(&act.body.targets, &ctx, src) {
            // CR 602.2b, 601.2b: the value of X is announced before targets are chosen,
            // so targets that depend on X ("target creature card with mana value X") are
            // possible if they are for some value the player could choose.
            let has_x = act.cost.mana.as_ref().is_some_and(|m| m.has_x())
                || act.cost.parts.iter().any(cost_part_has_x);
            let max_x = self.max_mana_available(p) + o.counter(counters::LOYALTY);
            let for_some_x = has_x
                && (1..=max_x as i32).any(|x| {
                    let mut c = ctx.clone();
                    c.x = x;
                    c.x_defined = true;
                    self.targets_possible(&act.body.targets, &c, src)
                });
            if !for_some_x {
                return false;
            }
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

    pub(crate) fn activation_prohibited(&self, p: PlayerId, src: ObjectId, is_mana: bool) -> bool {
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
                // CR 606.4: the cost of a loyalty ability may be modified by other effects.
                CostTarget::LoyaltyAbilities(f) => {
                    act.is_loyalty
                        && self.player_rel_matches(cm.who, p, &ctx)
                        && self.matches(src, f, &ctx)
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
                CostChange::ReduceMana { mana, colored_only } => {
                    if let Some(m) = cost.mana.as_mut() {
                        crate::cost_rules::reduce_by(m, mana, *colored_only, |_, cur, s| {
                            crate::cost_rules::default_half(cur, s)
                        });
                    }
                }
                _ => {}
            }
        }
        // CR 606.5: multiple costs to add or remove loyalty counters combine into one.
        let loyalty: Vec<i32> = cost
            .parts
            .iter()
            .filter_map(|c| match c {
                CostPart::Loyalty(n) => Some(*n),
                _ => None,
            })
            .collect();
        if loyalty.len() > 1 {
            cost.parts.retain(|c| !matches!(c, CostPart::Loyalty(_)));
            cost.parts
                .insert(0, CostPart::Loyalty(loyalty.iter().sum()));
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
        self.special.casting += 1;
        match self.activate_inner(p, src, &a, &act) {
            Ok(r) => {
                // CR 602.2e: the ability became activated.
                crate::draw_rules::finish_casting(self);
                Ok(r)
            }
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
        // CR 602.2a: an ability activated from a hidden zone reveals the card.
        if matches!(self.obj(src).zone, Zone::Hand(_) | Zone::Library(_)) {
            self.emit(Event::Custom {
                name: "revealed".into(),
                player: Some(p),
                obj: Some(src),
                amount: 0,
            });
        }
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
                any_color: self.any_color_mana(p, src, true),
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
            let pools_before: Vec<usize> = self
                .players
                .iter()
                .map(|pl| pl.mana_pool.mana.len())
                .collect();
            let body = act.body.clone();
            self.exec(&body.effect, &mut ctx);
            self.mana_ability_resolving = None;
            // CR 106.12a: "tapped for mana" triggers when such an ability resolves and
            // produces mana.
            if tapped_for_mana {
                let mana: Vec<crate::mana::ManaType> = self
                    .players
                    .iter()
                    .zip(pools_before)
                    .flat_map(|(pl, n)| pl.mana_pool.mana.iter().skip(n).map(|m| m.ty))
                    .collect();
                if !mana.is_empty() {
                    self.emit(Event::TappedForMana {
                        obj: src,
                        player: p,
                        mana,
                    });
                }
            }
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
        // CR 118.13a: how symbols that can be paid in more than one way will be paid.
        crate::cost_rules::choose_payment_ways(self, p, Some(src), &mut cost);
        // CR 602.1e: a modification of how the activation cost may be paid applies to the
        // total cost.
        let spend = SpendContext {
            is_ability: true,
            card_types: src_chars.card_types,
            source: Some(src),
            any_color: self.any_color_mana(p, src, true),
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
        if !paid.sacrificed.is_empty() {
            ctx.vars.insert(
                vars::SACRIFICED,
                paid.sacrificed.iter().map(|o| Entity::Object(*o)).collect(),
            );
        }
        self.saved_ctx.insert(id, ctx.clone());
        *self.objects[src.0 as usize]
            .activations_this_turn
            .entry(a.uid)
            .or_insert(0) += 1;
        // CR 702.29c: discarding a card to pay a cycling ability's cost is cycling it.
        if a.text == "Cycling"
            && act
                .cost
                .parts
                .iter()
                .any(|c| matches!(c, CostPart::DiscardSelf))
        {
            let card = self.current(src);
            self.emit(Event::Cycled { player: p, card });
        }
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

    /// Mana types `p` may spend as though they were mana of any color to pay for the
    /// spell `src` (`ability == false`) or the activated abilities of `src` (CR 602.1e).
    pub fn any_color_mana(&self, p: PlayerId, src: ObjectId, ability: bool) -> Vec<ManaType> {
        let mut out = Vec::new();
        for (s, c, e) in &self.statics.other {
            let StaticEffect::SpendAsAnyColor { applies_to, types } = e else {
                continue;
            };
            if *c != p {
                continue;
            }
            let ctx = Ctx::new(Some(*s), *c);
            let applies = match applies_to {
                CostTarget::Abilities(f) => ability && self.matches(src, f, &ctx),
                CostTarget::Spells(f) => !ability && self.matches(src, f, &ctx),
                CostTarget::ThisSpell => !ability && src == *s,
                CostTarget::Keyword(_) | CostTarget::LoyaltyAbilities(_) => false,
            };
            if applies {
                if types.is_empty() {
                    out.extend(ManaType::ALL);
                } else {
                    out.extend(types.iter().copied());
                }
            }
        }
        out
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
        let cost = &crate::kw::cumulative_upkeep::expand_repeated(self, cost, &ctx);
        for part in &cost.parts {
            if !self.cost_part_payable(p, part, src, &ctx) {
                return false;
            }
        }
        if let Some(m) = &cost.mana {
            let need = crate::as_though::payment_cost(self, p, &m.with_x(0));
            if need.mana_value() == 0
                && !need
                    .symbols
                    .iter()
                    .any(|s| matches!(s, ManaSymbol::Colorless | ManaSymbol::Colored(_)))
            {
                return true;
            }
            let spend = SpendContext {
                any_color: src
                    .map(|s| {
                        let mut v = self.any_color_mana(p, s, true);
                        v.extend(self.any_color_mana(p, s, false));
                        v
                    })
                    .unwrap_or_default(),
                ..Default::default()
            };
            let plan = crate::mana_abilities::plan_payment(self, p, &need, &spend, src);
            return plan.is_some();
        }
        true
    }

    pub fn can_pay_cost(&self, p: PlayerId, cost: &Cost, src: Option<ObjectId>, ctx: &Ctx) -> bool {
        let chars = src.map(|s| self.obj(s).chars.clone()).unwrap_or_default();
        let cost = &crate::kw::cumulative_upkeep::expand_repeated(self, cost, ctx);
        self.can_pay_cost_optimistic(p, cost, src, &chars)
    }

    /// Pays a cost during resolution ("you may pay ..."). Returns true if paid.
    pub fn pay_cost(&mut self, p: PlayerId, cost: &Cost, src: Option<ObjectId>, ctx: &Ctx) -> bool {
        let snapshot = self.clone();
        // CR 118.13b: the choice of how to pay hybrid and Phyrexian symbols is made
        // immediately before paying.
        let mut cost = cost.clone();
        crate::cost_rules::choose_payment_ways(self, p, src, &mut cost);
        let cost = &cost;
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
            // CR 614.17b: a cost that includes an event that can't happen can't be paid.
            CostPart::SacrificeSelf => so.is_some_and(|o| {
                o.zone == Zone::Battlefield && o.controller == p && !self.cant_be_sacrificed(o.id)
            }),
            CostPart::Sacrifice { filter, count } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| self.obj(*o).controller == p && !self.cant_be_sacrificed(*o))
                    .count()
                    >= n
            }
            CostPart::DiscardSelf => so.is_some_and(|o| o.zone == Zone::Hand(p)),
            CostPart::Discard { filter, count, .. } => {
                let n = self.eval_value(count, ctx).max(0) as usize;
                self.player(p)
                    .hand
                    .iter()
                    .filter(|c| {
                        Some(**c) != src
                            && crate::draw_rules::usable_for_cost(self, **c, filter, ctx)
                    })
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
                    .filter(|c| {
                        Some(*c) != src && crate::draw_rules::usable_for_cost(self, *c, filter, ctx)
                    })
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
                    .filter(|c| {
                        Some(**c) != src
                            && crate::draw_rules::usable_for_cost(self, **c, filter, ctx)
                    })
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
                    .filter(|c| {
                        Some(**c) != src
                            && crate::draw_rules::usable_for_cost(self, **c, filter, ctx)
                    })
                    .count()
                    >= n
            }
            // CR 121.2b: a cost that includes drawing more cards than the player may draw
            // can't be paid.
            CostPart::Effect(e) => {
                // The player paying the cost performs the action ("you" is that player).
                let mut c = ctx.clone();
                c.controller = p;
                crate::draw_rules::can_choose(self, e, &c)
            }
            CostPart::PayManaCostOf(s) => {
                crate::mana_abilities::can_pay_mana_cost_of(self, p, s, src, ctx)
            }
            CostPart::Repeated { .. } => {
                let one = Cost {
                    mana: None,
                    parts: vec![part.clone()],
                };
                let flat = crate::kw::cumulative_upkeep::expand_repeated(self, &one, ctx);
                let chars = so.map(|o| o.chars.clone()).unwrap_or_default();
                self.can_pay_cost_optimistic(p, &flat, src, &chars)
            }
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
        // CR 702.24a: a repeated cost's total is determined as it's paid.
        let cost = &crate::kw::cumulative_upkeep::expand_repeated(self, cost, ctx);
        // Mana first (mana abilities must be activated before costs are paid, 601.2g),
        // but tapping the source for {T} must not be used for mana: reserve it.
        if let Some(m) = &cost.mana {
            if m.symbols.contains(&crate::mana::ManaSymbol::Infinity) {
                return Err(Illegal(
                    "unpayable cost: an object with no mana cost (CR 118.6)".into(),
                ));
            }
            let reserve = if cost.has_tap() { src } else { None };
            // CR 609.4b: "as though it were mana of any color" changes only how it's paid.
            let m = &crate::as_though::payment_cost(self, p, m);
            let spent = crate::mana_abilities::pay_mana(self, p, m, spend, reserve)
                .ok_or_else(|| Illegal("can't pay mana".into()))?;
            self.players[p.idx()].mana_spent_this_turn += spent.len() as u32;
            paid.mana_spent = spent;
        }
        // Check all other parts are payable before paying any of them. (Mana abilities
        // activated above may have changed what's available, CR 121.8; callers roll back
        // a failed payment.)
        for part in &cost.parts {
            if !self.cost_part_payable(p, part, src, ctx) {
                return Err(Illegal(format!("can't pay {part:?}")));
            }
        }
        // CR 601.2h: costs that involve random elements or moving objects from a library
        // to a public zone are paid after all other costs.
        let (late, early): (Vec<&CostPart>, Vec<&CostPart>) =
            cost.parts.iter().partition(|c| cost_part_pays_last(c));
        for part in early.into_iter().chain(late) {
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
                    && self.remove_counters_by(
                        Entity::Object(s),
                        counters::LOYALTY,
                        (-*n) as u32,
                        Some(p),
                    ) < (-*n) as u32
                {
                    return bad("not enough loyalty");
                }
            }
            CostPart::SacrificeSelf => {
                let s = src.ok_or_else(|| Illegal("no source".into()))?;
                paid.objects.push(s);
                paid.sacrificed.push(s);
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
                    paid.sacrificed.push(o);
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
                    .filter(|c| {
                        Some(*c) != src && crate::draw_rules::usable_for_cost(self, *c, filter, ctx)
                    })
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
                    .filter(|c| {
                        Some(*c) != src && crate::draw_rules::usable_for_cost(self, *c, filter, ctx)
                    })
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
                if self.remove_counters_by(Entity::Object(s), kind, n, Some(p)) < n {
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
                        let r = self.remove_counters_by(Entity::Object(o), &k, n, Some(p));
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
                if self.remove_counters_by(Entity::Player(p), counters::ENERGY, n, Some(p)) < n {
                    return bad("not enough energy");
                }
            }
            CostPart::PayPlayerCounters { kind, count } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                if self.remove_counters_by(Entity::Player(p), kind, n, Some(p)) < n {
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
                    .filter(|c| {
                        Some(*c) != src && crate::draw_rules::usable_for_cost(self, *c, filter, ctx)
                    })
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
                    .filter(|c| {
                        Some(*c) != src && crate::draw_rules::usable_for_cost(self, *c, filter, ctx)
                    })
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
                // The player paying the cost performs the action ("you" is that player).
                let mut c = ctx.clone();
                c.controller = p;
                self.exec(e, &mut c);
            }
            CostPart::PayManaCostOf(s) => {
                if !crate::mana_abilities::pay_mana_cost_of(self, p, s, src, ctx) {
                    return bad("can't pay mana cost");
                }
            }
            CostPart::Repeated { .. } => {
                let one = Cost {
                    mana: None,
                    parts: vec![part.clone()],
                };
                let flat = crate::kw::cumulative_upkeep::expand_repeated(self, &one, ctx);
                let spend = SpendContext {
                    is_ability: true,
                    source: src,
                    ..Default::default()
                };
                let sub = self.pay_total_cost(p, &flat, src, &spend, ctx)?;
                paid.mana_spent.extend(sub.mana_spent);
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

/// A view of the game in which one object has substitute characteristics — used to check
/// rules and effects against the characteristics a card would have as the spell being
/// proposed (CR 601.3a–e).
struct WithChars<'c> {
    id: ObjectId,
    chars: &'c Characteristics,
}

impl crate::eval::View for WithChars<'_> {
    fn chars<'a>(&'a self, g: &'a Game, id: ObjectId) -> &'a Characteristics {
        if id == self.id {
            self.chars
        } else {
            &g.obj(id).chars
        }
    }
    fn controller(&self, g: &Game, id: ObjectId) -> PlayerId {
        g.obj(id).controller
    }
}

/// A filter describing spells ("creature spells", "spells with mana value 3") applied to
/// a card that would become such a spell: the "is on the stack" parts are dropped.
pub(crate) fn as_spell_filter(f: &Filter) -> Filter {
    match f {
        Filter::Spell | Filter::InZone(ZoneKind::Stack) => Filter::Any,
        Filter::And(v) => Filter::And(v.iter().map(as_spell_filter).collect()),
        Filter::Or(v) => Filter::Or(v.iter().map(as_spell_filter).collect()),
        Filter::Not(x) => match **x {
            // "nonspell" stays as written.
            Filter::Spell | Filter::InZone(ZoneKind::Stack) => f.clone(),
            _ => Filter::Not(Box::new(as_spell_filter(x))),
        },
        other => other.clone(),
    }
}

/// Whether choices made while proposing a spell can change the qualities that
/// prohibitions look at (CR 601.3a): the value of X changes its mana value.
fn proposal_may_change_qualities(chars: &Characteristics) -> bool {
    chars.mana_cost.as_ref().is_some_and(|m| m.has_x())
}

/// What was paid for a cost.
#[derive(Clone, Debug, Default)]
pub struct PaidCost {
    pub mana_spent: Vec<ManaType>,
    pub objects: Vec<ObjectId>,
    /// The permanents among `objects` that were sacrificed.
    pub sacrificed: Vec<ObjectId>,
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

/// Cost parts involving random elements or moving objects from a library to a public
/// zone (CR 601.2h).
fn cost_part_pays_last(c: &CostPart) -> bool {
    matches!(
        c,
        CostPart::Discard { random: true, .. }
            | CostPart::Mill(_)
            | CostPart::Exile {
                zone: ZoneKind::Library,
                ..
            }
    )
}

pub(crate) fn cost_part_has_x(c: &CostPart) -> bool {
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
