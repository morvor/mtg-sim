//! Combat (CR 506–511): declaring attackers and blockers, evasion and combat
//! restrictions/requirements (with requirement maximization, CR 508.1d / 509.1c), costs to
//! attack and block, combat damage assignment (including trample, first strike, double
//! strike, deathtouch), creatures put onto the battlefield attacking or blocking, removal
//! from combat, and combat timing windows (CR 506.8).

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::object::{EventInfo, Zone};
use crate::turn::Step;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttackerInfo {
    pub id: ObjectId,
    /// What it's attacking (player, planeswalker, or battle). `None` if that was removed
    /// from combat (CR 506.4c).
    pub target: Option<Entity>,
    /// What it was attacking when declared (CR 508.5, 508.7a).
    pub original_target: Option<Entity>,
    /// Declared as an attacker (vs put onto the battlefield attacking, CR 508.4).
    pub declared: bool,
    pub blocked: bool,
    /// Blocking creatures in the order they were declared.
    pub blockers: Vec<ObjectId>,
    /// Band this attacker is part of (CR 702.22).
    pub band: Option<u32>,
    /// The defending player for this creature (CR 508.5): the player it's attacking, the
    /// controller of the planeswalker it's attacking, or the protector of the battle it's
    /// attacking, as of the time it started attacking that player or permanent.
    pub defending_player: Option<PlayerId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockerInfo {
    pub id: ObjectId,
    pub blocking: Vec<ObjectId>,
    pub declared: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CombatState {
    pub attacking_player: Option<PlayerId>,
    /// All attacking players (more than one with the shared team turns option, CR 506.2b).
    pub attacking_players: Vec<PlayerId>,
    pub defending_players: Vec<PlayerId>,
    pub attackers: Vec<AttackerInfo>,
    pub blockers: Vec<BlockerInfo>,
    /// A first-strike combat damage step has happened this combat.
    pub first_strike_step: bool,
    /// Creatures that had first strike or double strike as the first combat damage step began.
    pub first_strikers: Vec<ObjectId>,
    /// Creatures that "had to attack" (CR 506.7).
    pub had_to_attack: Vec<ObjectId>,
    pub attackers_declared: bool,
    pub blockers_declared: bool,
    /// A combat damage step began this combat (CR 506.8).
    pub damage_step_began: bool,
    /// Any creature was declared as an attacker or put onto the battlefield attacking this
    /// combat (CR 508.8).
    pub any_attackers: bool,
    /// Attackers as declared (CR 506.5, 506.6, 508.6, 508.7a).
    pub declared_attackers: Vec<(ObjectId, Entity)>,
    /// Creatures declared as blockers (CR 506.5).
    pub declared_blockers: Vec<ObjectId>,
    /// Attackers removed from combat, with the defending player they had (CR 508.5).
    pub removed_attackers: Vec<(ObjectId, PlayerId)>,
}

impl CombatState {
    pub fn attacker(&self, id: ObjectId) -> Option<&AttackerInfo> {
        self.attackers.iter().find(|a| a.id == id)
    }
    pub fn attack_target(&self, id: ObjectId) -> Option<Entity> {
        self.attacker(id).and_then(|a| a.target)
    }
    pub fn is_blocked(&self, id: ObjectId) -> bool {
        self.attacker(id).is_some_and(|a| a.blocked)
    }
    /// An unblocked creature: an attacking creature that isn't blocked, once blockers have
    /// been declared (CR 509.1h) or when it entered attacking after that (CR 508.4d).
    pub fn is_unblocked(&self, id: ObjectId) -> bool {
        self.blockers_declared && self.attacker(id).is_some_and(|a| !a.blocked)
    }
    pub fn blockers_of(&self, attacker: ObjectId) -> Vec<ObjectId> {
        self.attacker(attacker)
            .map(|a| a.blockers.clone())
            .unwrap_or_default()
    }
    pub fn blocking(&self, blocker: ObjectId) -> Vec<ObjectId> {
        self.blockers
            .iter()
            .find(|b| b.id == blocker)
            .map(|b| b.blocking.clone())
            .unwrap_or_default()
    }
    /// Defending player for an attacking creature (CR 508.5). If it's no longer attacking,
    /// the player it was attacking before it was removed from combat.
    pub fn defending_player_of(&self, g: &Game, attacker: ObjectId) -> Option<PlayerId> {
        let Some(a) = self.attacker(attacker) else {
            return self
                .removed_attackers
                .iter()
                .rev()
                .find(|(id, _)| *id == attacker)
                .map(|(_, p)| *p);
        };
        if let Some(p) = a.defending_player {
            return Some(p);
        }
        let t = a.target.or(a.original_target)?;
        Some(entity_defender(g, t))
    }
    /// Whether the creature was the only creature declared as an attacker (CR 506.5).
    pub fn attacked_alone(&self, id: ObjectId) -> bool {
        self.declared_attackers.len() == 1 && self.declared_attackers[0].0 == id
    }
}

/// The player an attack on `t` is directed at: the player, the planeswalker's controller,
/// or the battle's protector (CR 508.5).
pub fn entity_defender(g: &Game, t: Entity) -> PlayerId {
    match t {
        Entity::Player(p) => p,
        Entity::Object(o) => {
            let ob = g.obj(o);
            if ob.is(CardType::Battle) {
                crate::battle::protector(g, o).unwrap_or(ob.controller)
            } else {
                ob.controller
            }
        }
    }
}

impl Game {
    pub fn is_attacking(&self, id: ObjectId) -> bool {
        self.combat
            .as_ref()
            .is_some_and(|c| c.attackers.iter().any(|a| a.id == id))
    }
    pub fn is_blocking(&self, id: ObjectId) -> bool {
        self.combat
            .as_ref()
            .is_some_and(|c| c.blockers.iter().any(|b| b.id == id))
    }
    pub fn attackers(&self) -> Vec<ObjectId> {
        self.combat
            .as_ref()
            .map(|c| c.attackers.iter().map(|a| a.id).collect())
            .unwrap_or_default()
    }
    pub fn blockers(&self) -> Vec<ObjectId> {
        self.combat
            .as_ref()
            .map(|c| c.blockers.iter().map(|b| b.id).collect())
            .unwrap_or_default()
    }
    /// Whether the creature "had to attack" this combat (CR 506.7).
    pub fn had_to_attack(&self, id: ObjectId) -> bool {
        self.combat
            .as_ref()
            .is_some_and(|c| c.had_to_attack.contains(&id))
    }

    /// Restrictions from statics and resolved effects, with their contexts.
    pub fn all_restrictions(
        &self,
    ) -> Vec<(
        Option<ObjectId>,
        PlayerId,
        Restriction,
        Option<Vec<ObjectId>>,
    )> {
        let mut v: Vec<(
            Option<ObjectId>,
            PlayerId,
            Restriction,
            Option<Vec<ObjectId>>,
        )> = self
            .statics
            .restrictions
            .iter()
            .map(|(s, c, r)| (Some(*s), *c, r.clone(), None))
            .collect();
        for e in &self.rule_effects {
            v.push((
                e.source,
                e.controller,
                e.restriction.clone(),
                e.objects.clone(),
            ));
        }
        v
    }

    /// Whether a restriction's object filter applies to `id`. Effects that named specific
    /// objects ("target creature can't block this turn") are locked onto those objects; the
    /// filter's references to targets/the source were resolved then.
    fn restriction_applies(
        &self,
        id: ObjectId,
        f: &Filter,
        ctx: &Ctx,
        locked: &Option<Vec<ObjectId>>,
    ) -> bool {
        match locked {
            Some(v) => v.contains(&id) && self.matches_unlocked_parts(id, f, ctx),
            None => self.matches(id, f, ctx),
        }
    }

    /// Matches the parts of a locked filter that don't refer to specific objects.
    fn matches_unlocked_parts(&self, id: ObjectId, f: &Filter, ctx: &Ctx) -> bool {
        match f {
            Filter::In(_) | Filter::Source | Filter::AttachedToSource => true,
            Filter::And(v) => v.iter().all(|x| self.matches_unlocked_parts(id, x, ctx)),
            Filter::Or(v) => v.iter().any(|x| self.matches_unlocked_parts(id, x, ctx)),
            other => self.matches(id, other, ctx),
        }
    }

    /// Number of restrictions of a kind applying to `id`.
    fn count_restrictions(
        &self,
        id: ObjectId,
        pick: impl Fn(&Restriction) -> Option<&Filter>,
    ) -> u32 {
        self.all_restrictions()
            .iter()
            .filter(|(s, c, r, locked)| match pick(r) {
                Some(f) => self.restriction_applies(id, f, &Ctx::new(*s, *c), locked),
                None => false,
            })
            .count() as u32
    }

    fn restricted_obj(&self, id: ObjectId, pick: impl Fn(&Restriction) -> Option<&Filter>) -> bool {
        self.count_restrictions(id, pick) > 0
    }

    /// Whether a creature can be declared as an attacker at all (CR 508.1a, 508.1c).
    pub fn can_attack(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        if o.zone != Zone::Battlefield
            || o.phased_out
            || !o.is_creature()
            || o.is(CardType::Battle)
            || o.tapped
        {
            return false;
        }
        if o.summoning_sick && !o.has_keyword(KeywordKind::Haste) {
            return false;
        }
        // CR 702.3b, unless an effect lets it attack as though it didn't have defender.
        if o.has_keyword(KeywordKind::Defender)
            && !self.restricted_obj(id, |r| match r {
                Restriction::AttackDespiteDefender(f) => Some(f),
                _ => None,
            })
        {
            return false;
        }
        !self.restricted_obj(id, |r| match r {
            Restriction::CantAttack(f) | Restriction::CantAttackOrBlock(f) => Some(f),
            _ => None,
        })
    }

    /// Whether a creature can attack a specific player/planeswalker/battle.
    pub fn can_attack_target(&self, id: ObjectId, target: Entity) -> bool {
        let defender = entity_defender(self, target);
        // Goaded creatures can't attack the goading player if able to attack another (CR 701.15b) — enforced as a requirement.
        !self.all_restrictions().iter().any(|(s, c, r, _)| match r {
            Restriction::CantAttackPlayer {
                attackers,
                defender: pf,
            } => {
                let ctx = Ctx::new(*s, *c);
                self.matches(id, attackers, &ctx) && self.player_filter_matches(pf, defender, &ctx)
            }
            _ => false,
        })
    }

    /// Whether a creature can block at all (CR 509.1a).
    pub fn can_block_at_all(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        if o.zone != Zone::Battlefield
            || o.phased_out
            || !o.is_creature()
            || o.is(CardType::Battle)
            || o.tapped
        {
            return false;
        }
        !self.restricted_obj(id, |r| match r {
            Restriction::CantBlock(f) | Restriction::CantAttackOrBlock(f) => Some(f),
            _ => None,
        })
    }

    /// Whether `blocker` may block `attacker`, considering evasion and restrictions on
    /// individual blocks (CR 509.1b).
    pub fn can_block(&self, blocker: ObjectId, attacker: ObjectId) -> bool {
        if !self.can_block_at_all(blocker) || !self.is_attacking(attacker) {
            return false;
        }
        if !self.could_block_pair(blocker, attacker) {
            return false;
        }
        let b = self.obj(blocker);
        let a = self.obj(attacker);
        let bk = &b.chars;
        let ak = &a.chars;
        // Flying (702.9b)
        if ak.has_keyword(KeywordKind::Flying)
            && !bk.has_keyword(KeywordKind::Flying)
            && !bk.has_keyword(KeywordKind::Reach)
        {
            return false;
        }
        // Shadow (702.28b)
        if ak.has_keyword(KeywordKind::Shadow) != bk.has_keyword(KeywordKind::Shadow) {
            return false;
        }
        // Horsemanship (702.31b)
        if ak.has_keyword(KeywordKind::Horsemanship) && !bk.has_keyword(KeywordKind::Horsemanship) {
            return false;
        }
        // Fear (702.36b)
        if ak.has_keyword(KeywordKind::Fear)
            && !(bk.is(CardType::Artifact) || bk.colors.contains(Color::Black))
        {
            return false;
        }
        // Intimidate (702.13b)
        if ak.has_keyword(KeywordKind::Intimidate)
            && !(bk.is(CardType::Artifact) || bk.colors.intersects(ak.colors))
        {
            return false;
        }
        // Skulk (702.118b)
        if ak.has_keyword(KeywordKind::Skulk) && b.power() > a.power() {
            return false;
        }
        // Landwalk (702.14c)
        for kw in ak.keywords().filter(|k| k.kind == KeywordKind::Landwalk) {
            if crate::kw::landwalk::landwalk_ignored(self, kw) {
                continue;
            }
            if let Some(f) = &kw.filter {
                let ctx = Ctx::new(Some(attacker), a.controller);
                if self.permanents().any(|o| {
                    o.controller == b.controller && o.chars.is_land() && self.matches(o.id, f, &ctx)
                }) {
                    return false;
                }
            }
        }
        // Protection (702.16e): can't be blocked by creatures with the quality.
        if self.protected_from(attacker, blocker) {
            return false;
        }
        // Restrictions from effects.
        for (s, c, r, locked) in self.all_restrictions() {
            let ctx = Ctx::new(s, c);
            match &r {
                Restriction::CantBeBlocked(f)
                    if self.restriction_applies(attacker, f, &ctx, &locked) =>
                {
                    return false
                }
                Restriction::CantBeBlockedBy {
                    attacker: af,
                    blocker: bf,
                } if self.restriction_applies(attacker, af, &ctx, &locked)
                    && self.matches(blocker, bf, &ctx) =>
                {
                    return false
                }
                Restriction::CanBlockOnly {
                    blocker: bf,
                    attackers: af,
                } if self.matches(blocker, bf, &ctx) && !self.matches(attacker, af, &ctx) => {
                    return false
                }
                _ => {}
            }
        }
        crate::keyword_impls::extra_block_restrictions(self, blocker, attacker)
    }

    /// The blocker's controller must be defending against the attacker (CR 509.1a): the
    /// attacker is attacking them, a planeswalker they control, or a battle they protect.
    /// With shared team turns, any player on the defending team (CR 805.10d).
    fn could_block_pair(&self, blocker: ObjectId, attacker: ObjectId) -> bool {
        let Some(c) = &self.combat else { return false };
        let Some(dp) = c.defending_player_of(self, attacker) else {
            return false;
        };
        let bc = self.obj(blocker).controller;
        if shared_team_turns(self) {
            !self.are_opponents(dp, bc)
        } else {
            dp == bc
        }
    }

    /// Maximum number of attackers this creature can block (CR 509.1a; 702.? "can block
    /// an additional creature").
    pub fn max_blocks(&self, blocker: ObjectId) -> Option<u32> {
        let mut n = 1u32;
        for (s, c, r, _) in self.all_restrictions() {
            if let Restriction::ExtraBlocks {
                blocker: bf,
                n: extra,
            } = &r
            {
                if self.matches(blocker, bf, &Ctx::new(s, c)) {
                    match extra {
                        None => return None,
                        Some(k) => n += k,
                    }
                }
            }
        }
        Some(n)
    }

    /// Minimum number of blockers an attacker requires (menace etc.).
    pub fn min_blockers(&self, attacker: ObjectId) -> u32 {
        let mut n = 1;
        if self.obj(attacker).has_keyword(KeywordKind::Menace) {
            n = 2; // CR 702.110b
        }
        for (s, c, r, _) in self.all_restrictions() {
            if let Restriction::MinBlockers { attacker: af, n: k } = &r {
                if self.matches(attacker, af, &Ctx::new(s, c)) {
                    n = n.max(*k);
                }
            }
        }
        n
    }

    /// Whether `t` could currently be attacked by a creature entering attacking or having
    /// its attack reselected: a defending player in the game, or a planeswalker a
    /// defending player controls / battle a defending player protects (CR 506.3c, 508.4a).
    pub fn valid_attack_target(&self, t: Entity) -> bool {
        let defending = self
            .combat
            .as_ref()
            .map(|c| c.defending_players.clone())
            .unwrap_or_default();
        match t {
            Entity::Player(p) => self.player(p).in_game() && defending.contains(&p),
            Entity::Object(o) => {
                let ob = self.obj(o);
                if !self.is_live(o) || ob.zone != Zone::Battlefield || ob.phased_out {
                    return false;
                }
                (ob.is(CardType::Planeswalker) && defending.contains(&ob.controller))
                    || (ob.is(CardType::Battle)
                        && crate::battle::protector(self, o)
                            .is_some_and(|p| defending.contains(&p)))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Beginning of combat
// ---------------------------------------------------------------------------

/// Whether the game uses the shared team turns option (CR 805; always used in
/// Two-Headed Giant, CR 810).
pub fn shared_team_turns(g: &Game) -> bool {
    g.config.variant == Variant::TwoHeadedGiant && g.config.teams.is_some()
}

/// The attacking player(s) (CR 506.2, 506.2b): the active player, or with shared team
/// turns every player on the active team.
pub fn attacking_players(g: &Game) -> Vec<PlayerId> {
    if let Some(c) = &g.combat {
        if !c.attacking_players.is_empty() {
            return c.attacking_players.clone();
        }
    }
    active_team(g)
}

/// The active player, plus teammates with shared team turns (CR 805.4a).
fn active_team(g: &Game) -> Vec<PlayerId> {
    let ap = g.turn.active;
    let mut v = vec![ap];
    if shared_team_turns(g) {
        v.extend(g.teammates(ap));
    }
    v
}

/// Whether `b` is within `a`'s range of influence (CR 801.2): at most N seats away,
/// counting only players still in the game. Unlimited range if the option isn't used.
pub fn within_range(g: &Game, a: PlayerId, b: PlayerId) -> bool {
    let Some(n) = g.config.range_of_influence else {
        return true;
    };
    if a == b {
        return true;
    }
    let seats: Vec<PlayerId> = g.players_in_game();
    let (Some(i), Some(j)) = (
        seats.iter().position(|p| *p == a),
        seats.iter().position(|p| *p == b),
    ) else {
        return false;
    };
    let len = seats.len();
    let d = (i + len - j) % len;
    d.min(len - d) as u32 <= n
}

/// Beginning of combat (CR 507): set up combat and choose the defending player.
pub fn begin_combat(g: &mut Game) {
    let ap = g.turn.active;
    let attacking = active_team(g);
    // CR 801.3: only opponents within range of influence can be attacked.
    let opponents: Vec<PlayerId> = g
        .opponents(ap)
        .into_iter()
        .filter(|p| within_range(g, ap, *p))
        .collect();
    let defending =
        if opponents.len() <= 1 || g.config.attack_multiple_players || shared_team_turns(g) {
            // CR 506.2, 802.2, 805.10a: all opponents are defending players.
            opponents.clone()
        } else {
            // CR 507.1 / 506.2a: the active player chooses one opponent.
            let cands: Vec<Entity> = opponents.iter().map(|p| Entity::Player(*p)).collect();
            g.ask_entities(ap, None, "Choose the defending player", cands, 1, 1)
                .into_iter()
                .filter_map(|e| e.player())
                .collect()
        };
    g.combat = Some(CombatState {
        attacking_player: Some(ap),
        attacking_players: attacking,
        defending_players: defending,
        ..Default::default()
    });
}

/// Possible attack targets for creatures of the attacking player (CR 508.1b).
pub fn attack_targets(g: &Game) -> Vec<Entity> {
    let Some(c) = &g.combat else { return vec![] };
    let mut out = Vec::new();
    for p in &c.defending_players {
        if !g.player(*p).in_game() {
            continue;
        }
        out.push(Entity::Player(*p));
        for o in g.permanents() {
            if o.is(CardType::Planeswalker) && o.controller == *p {
                out.push(Entity::Object(o.id));
            }
            if o.is(CardType::Battle)
                && crate::battle::protector(g, o.id) == Some(*p)
                && !out.contains(&Entity::Object(o.id))
            {
                out.push(Entity::Object(o.id));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Declare attackers (CR 508.1)
// ---------------------------------------------------------------------------

/// A requirement that applies to the declaration of attackers (CR 508.1d).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttackRequirement {
    /// "[creature] attacks (each combat) if able".
    Attacks(ObjectId),
    /// "[creature] attacks a player other than [players] if able" (goad, CR 701.15b).
    AttacksPlayerOtherThan(ObjectId, Vec<PlayerId>),
}

impl AttackRequirement {
    pub fn creature(&self) -> ObjectId {
        match self {
            AttackRequirement::Attacks(c) | AttackRequirement::AttacksPlayerOtherThan(c, _) => *c,
        }
    }
    fn obeyed(&self, decl: &[(ObjectId, Entity)]) -> bool {
        match self {
            AttackRequirement::Attacks(c) => decl.iter().any(|(a, _)| a == c),
            AttackRequirement::AttacksPlayerOtherThan(c, ps) => decl
                .iter()
                .any(|(a, t)| a == c && matches!(t, Entity::Player(p) if !ps.contains(p))),
        }
    }
}

/// Creatures that could attack, and what each could attack (CR 508.1a–b).
pub fn attack_options(g: &Game) -> Vec<(ObjectId, Vec<Entity>)> {
    let players = attacking_players(g);
    let targets = attack_targets(g);
    g.permanents()
        .filter(|o| players.contains(&o.controller))
        .map(|o| o.id)
        .filter(|id| g.can_attack(*id))
        .map(|c| {
            let ctl = g.obj(c).controller;
            (
                c,
                targets
                    .iter()
                    .copied()
                    .filter(|t| {
                        g.can_attack_target(c, *t) && within_range(g, ctl, entity_defender(g, *t))
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, t)| !t.is_empty())
        .collect()
}

/// All requirements on the attacking players' creatures (CR 508.1d).
pub fn attack_requirements(g: &Game) -> Vec<AttackRequirement> {
    let players = attacking_players(g);
    let mut out = Vec::new();
    let creatures: Vec<ObjectId> = g
        .permanents()
        .filter(|o| players.contains(&o.controller) && o.is_creature())
        .map(|o| o.id)
        .collect();
    for id in creatures {
        let n = g.count_restrictions(id, |r| match r {
            Restriction::MustAttack(f) => Some(f),
            _ => None,
        });
        for _ in 0..n {
            out.push(AttackRequirement::Attacks(id));
        }
        // CR 701.15b: a goaded creature attacks each combat if able and attacks a player
        // other than the goading player if able.
        let goaders = g.obj(id).goaded_by.clone();
        if !goaders.is_empty() {
            out.push(AttackRequirement::Attacks(id));
            out.push(AttackRequirement::AttacksPlayerOtherThan(id, goaders));
        }
    }
    out
}

fn obeyed_attack_requirements(reqs: &[AttackRequirement], decl: &[(ObjectId, Entity)]) -> u32 {
    reqs.iter().filter(|r| r.obeyed(decl)).count() as u32
}

/// Restrictions that apply to the declaration as a whole (CR 508.1c).
struct AttackRules {
    cant_alone: BTreeSet<ObjectId>,
    max: Option<usize>,
}

fn attack_rules(g: &Game, options: &[(ObjectId, Vec<Entity>)]) -> AttackRules {
    let mut cant_alone = BTreeSet::new();
    for (c, _) in options {
        if g.restricted_obj(*c, |r| match r {
            Restriction::CantAttackAlone(f) => Some(f),
            _ => None,
        }) {
            cant_alone.insert(*c);
        }
    }
    let max = g
        .all_restrictions()
        .iter()
        .filter_map(|(_, _, r, _)| match r {
            Restriction::MaxAttackers(n) => Some(*n as usize),
            _ => None,
        })
        .min();
    AttackRules { cant_alone, max }
}

fn attack_restrictions_ok(
    g: &Game,
    rules: &AttackRules,
    options: &[(ObjectId, Vec<Entity>)],
    decl: &[(ObjectId, Entity)],
) -> bool {
    let mut seen = BTreeSet::new();
    for (a, t) in decl {
        if !seen.insert(*a) {
            return false;
        }
        match options.iter().find(|(id, _)| id == a) {
            Some((_, ts)) if ts.contains(t) => {}
            _ => return false,
        }
    }
    if decl.len() == 1 && rules.cant_alone.contains(&decl[0].0) {
        return false;
    }
    if rules.max.is_some_and(|m| decl.len() > m) {
        return false;
    }
    crate::keyword_impls::attack_declaration_extra_checks(g, decl)
}

/// The cost required to attack with `creature` attacking `target` (CR 508.1d, 508.1h),
/// if any.
pub fn required_attack_cost(g: &Game, creature: ObjectId, target: Entity) -> Option<Cost> {
    let mut total = Cost::default();
    let mut any = false;
    for (s, c, r, locked) in g.all_restrictions() {
        if let Restriction::AttackCost {
            attackers,
            defender,
            planeswalkers,
            cost,
        } = &r
        {
            let ctx = Ctx::new(s, c);
            if !g.restriction_applies(creature, attackers, &ctx, &locked) {
                continue;
            }
            let hit = match target {
                Entity::Player(p) => g.player_filter_matches(defender, p, &ctx),
                Entity::Object(o) => {
                    *planeswalkers
                        && g.obj(o).is(CardType::Planeswalker)
                        && g.player_filter_matches(defender, g.obj(o).controller, &ctx)
                }
            };
            if hit {
                crate::casting::add_cost(&mut total, cost);
                any = true;
            }
        }
    }
    (any && !total.is_free()).then_some(total)
}

/// Total required cost to attack for a declaration (CR 508.1h).
pub fn total_attack_cost(g: &Game, decl: &[(ObjectId, Entity)]) -> Cost {
    let mut total = Cost::default();
    for (a, t) in decl {
        if let Some(c) = required_attack_cost(g, *a, *t) {
            crate::casting::add_cost(&mut total, &c);
        }
    }
    total
}

/// Searches for a declaration obeying the maximum number of requirements without
/// disobeying restrictions and without paying any costs (CR 508.1d). Returns (the maximum,
/// a declaration achieving it with as few attackers as the search finds).
pub fn best_attack(
    g: &Game,
    options: &[(ObjectId, Vec<Entity>)],
    reqs: &[AttackRequirement],
) -> (u32, Vec<(ObjectId, Entity)>) {
    let rules = attack_rules(g, options);
    // Creatures that can't attack without paying a cost aren't required to (CR 508.1d).
    let free: Vec<(ObjectId, Vec<Entity>)> = options
        .iter()
        .map(|(c, ts)| {
            (
                *c,
                ts.iter()
                    .copied()
                    .filter(|t| required_attack_cost(g, *c, *t).is_none())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let mut best = (obeyed_attack_requirements(reqs, &[]), Vec::new());
    let space: f64 = free.iter().map(|(_, t)| (t.len() + 1) as f64).product();
    if space <= 100_000.0 {
        // Requirements still obeyable by creatures from index i on (for pruning).
        let mut suffix = vec![0u32; free.len() + 1];
        for i in (0..free.len()).rev() {
            let n = reqs.iter().filter(|r| r.creature() == free[i].0).count() as u32;
            suffix[i] = suffix[i + 1] + n;
        }
        let mut cur = Vec::new();
        attack_dfs(
            g, &rules, options, &free, reqs, &suffix, 0, &mut cur, &mut best,
        );
    } else {
        // Too many combinations: greedily attack with every creature that has requirements.
        let mut decl = Vec::new();
        for (c, ts) in &free {
            let mine: Vec<&AttackRequirement> =
                reqs.iter().filter(|r| r.creature() == *c).collect();
            if mine.is_empty() {
                continue;
            }
            let t = ts
                .iter()
                .copied()
                .max_by_key(|t| mine.iter().filter(|r| r.obeyed(&[(*c, *t)])).count());
            if let Some(t) = t {
                decl.push((*c, t));
            }
        }
        let n = obeyed_attack_requirements(reqs, &decl);
        if n > best.0 && attack_restrictions_ok(g, &rules, options, &decl) {
            best = (n, decl);
        }
    }
    best
}

#[allow(clippy::too_many_arguments)]
fn attack_dfs(
    g: &Game,
    rules: &AttackRules,
    options: &[(ObjectId, Vec<Entity>)],
    free: &[(ObjectId, Vec<Entity>)],
    reqs: &[AttackRequirement],
    suffix: &[u32],
    i: usize,
    cur: &mut Vec<(ObjectId, Entity)>,
    best: &mut (u32, Vec<(ObjectId, Entity)>),
) {
    let now = obeyed_attack_requirements(reqs, cur);
    if i == free.len() {
        if now > best.0 && attack_restrictions_ok(g, rules, options, cur) {
            *best = (now, cur.clone());
        }
        return;
    }
    // Prune branches that can't beat the best found (the empty declaration is always a
    // candidate, so `best` is always a legal declaration).
    if now + suffix[i] <= best.0 {
        return;
    }
    attack_dfs(g, rules, options, free, reqs, suffix, i + 1, cur, best);
    for t in free[i].1.clone() {
        cur.push((free[i].0, t));
        attack_dfs(g, rules, options, free, reqs, suffix, i + 1, cur, best);
        cur.pop();
    }
}

/// Whether a proposed declaration of attackers is legal (CR 508.1a–d), not counting
/// whether its costs can be paid.
pub fn attack_declaration_legal(
    g: &Game,
    options: &[(ObjectId, Vec<Entity>)],
    decl: &[(ObjectId, Entity)],
) -> bool {
    let reqs = attack_requirements(g);
    let (max, _) = best_attack(g, options, &reqs);
    attack_declaration_legal_with(g, options, decl, &reqs, max)
}

fn attack_declaration_legal_with(
    g: &Game,
    options: &[(ObjectId, Vec<Entity>)],
    decl: &[(ObjectId, Entity)],
    reqs: &[AttackRequirement],
    max: u32,
) -> bool {
    let rules = attack_rules(g, options);
    attack_restrictions_ok(g, &rules, options, decl)
        && obeyed_attack_requirements(reqs, decl) >= max
}

/// Declare attackers step turn-based action (CR 508.1).
pub fn declare_attackers_step(g: &mut Game) {
    if g.combat.is_none() {
        begin_combat(g);
    }
    g.recompute();
    let ap = g.turn.active;
    let options = attack_options(g);
    let reqs = attack_requirements(g);
    let (max, best) = best_attack(g, &options, &reqs);
    let declared: Vec<(ObjectId, Entity)> = if options.is_empty() {
        vec![]
    } else {
        match g.ask(
            ap,
            Decision::DeclareAttackers {
                options: options.clone(),
            },
        ) {
            Answer::Attackers(v)
                if attack_declaration_legal_with(g, &options, &v, &reqs, max)
                    && g.can_pay_cost(ap, &total_attack_cost(g, &v), None, &Ctx::new(None, ap)) =>
            {
                v
            }
            // CR 508.1: an illegal declaration is undone; the engine declares a legal one.
            _ => best.clone(),
        }
    };
    let had_to: Vec<ObjectId> = {
        let mut v: Vec<ObjectId> = reqs.iter().map(|r| r.creature()).collect();
        v.dedup();
        v
    };
    // Costs may fail to be paid; keep a snapshot to return to (CR 508.1).
    let may_cost = !total_attack_cost(g, &declared).is_free()
        || declared.iter().any(|(a, _)| {
            g.obj(*a).chars.abilities.iter().any(|ab| {
                matches!(&ab.kind, AbilityKind::Static(s)
                    if matches!(s.effect, StaticEffect::OptionalAttackCost { .. }))
            })
        });
    let snapshot = may_cost.then(|| g.clone());
    if !perform_attack_declaration(g, ap, declared, &had_to, true) {
        let Some(snapshot) = snapshot else { return };
        // CR 508.1j / 733: the costs couldn't be paid; return to the moment before the
        // declaration and declare a legal attack that needs no payment.
        let agents = g.agents.clone();
        *g = snapshot;
        g.agents = agents;
        perform_attack_declaration(g, ap, best, &had_to, false);
    }
}

/// Performs a declaration of attackers: taps (CR 508.1f), costs (508.1g–j), attacking
/// creatures (508.1k), triggers (508.1m). Returns false if the costs couldn't be paid.
fn perform_attack_declaration(
    g: &mut Game,
    ap: PlayerId,
    declared: Vec<(ObjectId, Entity)>,
    had_to_attack: &[ObjectId],
    allow_optional: bool,
) -> bool {
    // CR 508.1e: bands.
    let bands = announce_bands(g, ap, &declared);
    // CR 508.1f: tap attackers (vigilance: 702.20b).
    for (a, _) in &declared {
        if !g.obj(*a).has_keyword(KeywordKind::Vigilance) {
            g.tap(*a);
        }
    }
    // CR 508.1g: optional costs to attack ("as it attacks").
    let mut optional: Vec<(ObjectId, Cost, Option<Body>)> = Vec::new();
    if allow_optional {
        g.recompute();
        for (a, _) in &declared {
            let o = g.obj(*a).clone();
            for ab in &o.chars.abilities {
                let AbilityKind::Static(s) = &ab.kind else {
                    continue;
                };
                let StaticEffect::OptionalAttackCost { cost, then } = &s.effect else {
                    continue;
                };
                let ctx = Ctx::new(Some(*a), o.controller);
                if !g.can_pay_cost(o.controller, cost, Some(*a), &ctx) {
                    continue;
                }
                let prompt = format!("Pay an optional cost as {} attacks?", o.chars.name);
                if g.ask_yes_no(o.controller, Some(*a), &prompt, false) {
                    optional.push((*a, cost.clone(), then.as_deref().cloned()));
                }
            }
        }
    }
    // CR 508.1h–j: the total cost is locked in, then paid (mana abilities may be activated).
    let required = total_attack_cost(g, &declared);
    if !required.is_free() && !g.pay_cost(ap, &required, None, &Ctx::new(None, ap)) {
        return false;
    }
    for (a, cost, _) in &optional {
        let ctl = g.obj(*a).controller;
        if !g.pay_cost(ctl, cost, Some(*a), &Ctx::new(Some(*a), ctl)) {
            return false;
        }
    }
    // Keyword-specific costs (e.g. banding announcements) are handled by keyword modules.
    crate::keyword_impls::pay_attack_costs(g, ap, &declared);
    let players = attacking_players(g);
    // CR 508.1k: each chosen creature still controlled by an attacking player attacks.
    let final_decl: Vec<(ObjectId, Entity)> = declared
        .iter()
        .copied()
        .filter(|(a, _)| {
            g.is_live(*a)
                && g.obj(*a).zone == Zone::Battlefield
                && players.contains(&g.obj(*a).controller)
        })
        .collect();
    let defenders: Vec<PlayerId> = final_decl
        .iter()
        .map(|(_, t)| entity_defender(g, *t))
        .collect();
    // CR 508.6: record which players attacked which players.
    for (a, t) in &final_decl {
        if let Entity::Player(p) = t {
            let pair = (g.obj(*a).controller, *p);
            if !g.turn.attacked_players.contains(&pair) {
                g.turn.attacked_players.push(pair);
            }
        }
    }
    let c = g.combat.get_or_insert_with(CombatState::default);
    c.attackers_declared = true;
    c.had_to_attack = had_to_attack.to_vec();
    c.declared_attackers = final_decl.clone();
    if !final_decl.is_empty() {
        c.any_attackers = true;
    }
    for ((a, t), dp) in final_decl.iter().zip(defenders) {
        c.attackers.push(AttackerInfo {
            id: *a,
            target: Some(*t),
            original_target: Some(*t),
            declared: true,
            blocked: false,
            blockers: vec![],
            band: bands.iter().find(|(x, _)| x == a).map(|(_, b)| *b),
            defending_player: Some(dp),
        });
    }
    // Reflexive triggers of optional attack costs ("When you do, ...", CR 603.12).
    for (a, _, then) in optional {
        if let Some(body) = then {
            let ctl = g.obj(a).controller;
            g.trigger_order += 1;
            let order = g.trigger_order;
            g.pending_triggers.push(PendingTrigger {
                source: a,
                controller: ctl,
                ability: AbilityDef::new(
                    AbilityKind::Triggered(TriggeredAbility::new(
                        TriggerCond::Custom("reflexive".into()),
                        body.clone(),
                    )),
                    "reflexive trigger",
                ),
                event: EventInfo {
                    object: Some(a),
                    ..Default::default()
                },
                source_lki: None,
                saved: None,
                body: Some(body),
                order,
            });
        }
    }
    // CR 508.1m: abilities that trigger on attackers being declared.
    if !final_decl.is_empty() {
        g.log(|_g| format!("{ap} attacks with {} creature(s)", final_decl.len()));
        g.emit(Event::AttackersDeclared {
            player: ap,
            attackers: final_decl,
        });
    }
    g.dirty = true;
    true
}

/// CR 508.1e, 702.22c–d: the active player announces which attacking creatures are banded
/// together. A band has creatures with banding and at most one without, all attacking the
/// same player, planeswalker, or battle. Returns (creature, band id) pairs.
fn announce_bands(
    g: &mut Game,
    ap: PlayerId,
    declared: &[(ObjectId, Entity)],
) -> Vec<(ObjectId, u32)> {
    let has_banding = |g: &Game, id: ObjectId| g.obj(id).has_keyword(KeywordKind::Banding);
    let mut out: Vec<(ObjectId, u32)> = Vec::new();
    let mut next = 1u32;
    for (a, t) in declared {
        if !has_banding(g, *a) || out.iter().any(|(x, _)| x == a) {
            continue;
        }
        let cands: Vec<ObjectId> = declared
            .iter()
            .filter(|(x, u)| x != a && u == t && !out.iter().any(|(y, _)| y == x))
            .map(|(x, _)| *x)
            .collect();
        if cands.is_empty() {
            continue;
        }
        let name = g.obj(*a).chars.name.clone();
        let chosen = g.ask_objects(
            ap,
            Some(*a),
            &format!("Choose attacking creatures to band with {name}"),
            cands.clone(),
            0,
            cands.len() as u32,
        );
        if chosen.is_empty() {
            continue;
        }
        let without = chosen.iter().filter(|x| !has_banding(g, **x)).count();
        if without > 1 {
            continue; // Not a legal band (CR 702.22c).
        }
        out.push((*a, next));
        for x in chosen {
            out.push((x, next));
        }
        next += 1;
    }
    out
}

/// An effect states that a creature is attacking (CR 508.4): it becomes an attacking
/// creature attacking `target` without being declared as an attacker, unaffected by
/// requirements and restrictions on declaring attackers (CR 508.4c). Returns false if it
/// doesn't become attacking (CR 506.3b–c, 506.3g, 508.4b).
pub fn make_attacking(g: &mut Game, id: ObjectId, target: Entity) -> bool {
    if g.combat.is_none() || g.is_attacking(id) || !g.is_live(id) {
        return false;
    }
    let o = g.obj(id);
    if o.zone != Zone::Battlefield
        || !o.is_creature()
        || o.is(CardType::Battle)
        || !attacking_players(g).contains(&o.controller)
        || !g.valid_attack_target(target)
    {
        return false;
    }
    put_onto_battlefield_attacking(g, id, target);
    g.is_attacking(id)
}

/// "[player a] is attacking [player b]" (CR 508.6): a controls a creature attacking b.
pub fn player_is_attacking(g: &Game, a: PlayerId, b: PlayerId) -> bool {
    g.combat.as_ref().is_some_and(|c| {
        c.attackers
            .iter()
            .any(|x| x.target == Some(Entity::Player(b)) && g.obj(x.id).controller == a)
    })
}

/// "[player a] has attacked [player b]" this turn (CR 508.6): a declared one or more
/// creatures as attackers attacking b.
pub fn player_has_attacked(g: &Game, a: PlayerId, b: PlayerId) -> bool {
    g.turn.attacked_players.contains(&(a, b))
}

// ---------------------------------------------------------------------------
// Declare blockers (CR 509.1)
// ---------------------------------------------------------------------------

/// A requirement that applies to the declaration of blockers (CR 509.1c).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockRequirement {
    /// "[creature] blocks (each combat) if able".
    Blocks(ObjectId),
    /// "[attacker] must be blocked if able".
    AttackerBlocked(ObjectId),
    /// "All creatures able to block [attacker] do so": this creature blocks that attacker.
    BlocksAttacker(ObjectId, ObjectId),
}

impl BlockRequirement {
    fn obeyed(&self, decl: &[(ObjectId, ObjectId)]) -> bool {
        match self {
            BlockRequirement::Blocks(b) => decl.iter().any(|(x, _)| x == b),
            BlockRequirement::AttackerBlocked(a) => decl.iter().any(|(_, y)| y == a),
            BlockRequirement::BlocksAttacker(b, a) => decl.iter().any(|(x, y)| x == b && y == a),
        }
    }
}

fn obeyed_block_requirements(reqs: &[BlockRequirement], decl: &[(ObjectId, ObjectId)]) -> u32 {
    reqs.iter().filter(|r| r.obeyed(decl)).count() as u32
}

/// Creatures of the given defending players that could block, and which attackers each
/// could block (CR 509.1a–b).
pub fn block_options(g: &Game, defenders: &[PlayerId]) -> Vec<(ObjectId, Vec<ObjectId>)> {
    let attackers = g.attackers();
    g.permanents()
        .filter(|o| defenders.contains(&o.controller))
        .map(|o| o.id)
        .filter(|id| g.can_block_at_all(*id))
        .map(|b| {
            (
                b,
                attackers
                    .iter()
                    .copied()
                    .filter(|a| g.can_block(b, *a))
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, a)| !a.is_empty())
        .collect()
}

/// Requirements on blocks (CR 509.1c) relevant to these options.
pub fn block_requirements(
    g: &Game,
    options: &[(ObjectId, Vec<ObjectId>)],
) -> Vec<BlockRequirement> {
    let mut out = Vec::new();
    for (b, _) in options {
        let n = g.count_restrictions(*b, |r| match r {
            Restriction::MustBlock(f) => Some(f),
            _ => None,
        });
        for _ in 0..n {
            out.push(BlockRequirement::Blocks(*b));
        }
    }
    let mut attackers: Vec<ObjectId> = options.iter().flat_map(|(_, a)| a.clone()).collect();
    attackers.sort();
    attackers.dedup();
    for a in attackers {
        let n = g.count_restrictions(a, |r| match r {
            Restriction::MustBeBlocked(f) => Some(f),
            _ => None,
        });
        for _ in 0..n {
            out.push(BlockRequirement::AttackerBlocked(a));
        }
        let all = g.count_restrictions(a, |r| match r {
            Restriction::MustBeBlockedByAll(f) => Some(f),
            _ => None,
        });
        for _ in 0..all {
            for (b, atts) in options {
                if atts.contains(&a) {
                    out.push(BlockRequirement::BlocksAttacker(*b, a));
                }
            }
        }
    }
    out
}

struct BlockRules {
    cant_alone: BTreeSet<ObjectId>,
    max: Option<usize>,
    max_blocks: BTreeMap<ObjectId, Option<u32>>,
    min_blockers: BTreeMap<ObjectId, u32>,
}

fn block_rules(g: &Game, options: &[(ObjectId, Vec<ObjectId>)]) -> BlockRules {
    let mut cant_alone = BTreeSet::new();
    let mut max_blocks = BTreeMap::new();
    let mut min_blockers = BTreeMap::new();
    for (b, atts) in options {
        if g.restricted_obj(*b, |r| match r {
            Restriction::CantBlockAlone(f) => Some(f),
            _ => None,
        }) {
            cant_alone.insert(*b);
        }
        max_blocks.insert(*b, g.max_blocks(*b));
        for a in atts {
            min_blockers.entry(*a).or_insert_with(|| g.min_blockers(*a));
        }
    }
    let max = g
        .all_restrictions()
        .iter()
        .filter_map(|(_, _, r, _)| match r {
            Restriction::MaxBlockers(n) => Some(*n as usize),
            _ => None,
        })
        .min();
    BlockRules {
        cant_alone,
        max,
        max_blocks,
        min_blockers,
    }
}

fn block_restrictions_ok(
    g: &Game,
    rules: &BlockRules,
    options: &[(ObjectId, Vec<ObjectId>)],
    decl: &[(ObjectId, ObjectId)],
) -> bool {
    let mut per_blocker: BTreeMap<ObjectId, u32> = BTreeMap::new();
    let mut per_attacker: BTreeMap<ObjectId, u32> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for (b, a) in decl {
        if !seen.insert((*b, *a)) {
            return false;
        }
        match options.iter().find(|(id, _)| id == b) {
            Some((_, atts)) if atts.contains(a) => {}
            _ => return false,
        }
        *per_blocker.entry(*b).or_insert(0) += 1;
        *per_attacker.entry(*a).or_insert(0) += 1;
    }
    for (b, n) in &per_blocker {
        if let Some(Some(max)) = rules.max_blocks.get(b) {
            if n > max {
                return false;
            }
        }
    }
    // Menace / minimum blockers (CR 702.110b).
    for (a, n) in &per_attacker {
        if *n < rules.min_blockers.get(a).copied().unwrap_or(1) {
            return false;
        }
    }
    if per_blocker.len() == 1
        && rules
            .cant_alone
            .contains(per_blocker.keys().next().unwrap())
    {
        return false;
    }
    if rules.max.is_some_and(|m| per_blocker.len() > m) {
        return false;
    }
    crate::keyword_impls::block_declaration_extra_checks(g, options, decl)
}

/// The cost required for `blocker` to block (CR 509.1d), if any.
pub fn required_block_cost(g: &Game, blocker: ObjectId) -> Option<Cost> {
    let mut total = Cost::default();
    let mut any = false;
    for (s, c, r, locked) in g.all_restrictions() {
        if let Restriction::BlockCost { blockers, cost } = &r {
            if g.restriction_applies(blocker, blockers, &Ctx::new(s, c), &locked) {
                crate::casting::add_cost(&mut total, cost);
                any = true;
            }
        }
    }
    (any && !total.is_free()).then_some(total)
}

/// Total cost to block for each player (CR 509.1d).
pub fn block_costs_by_player(g: &Game, decl: &[(ObjectId, ObjectId)]) -> Vec<(PlayerId, Cost)> {
    let mut blockers: Vec<ObjectId> = decl.iter().map(|(b, _)| *b).collect();
    blockers.sort();
    blockers.dedup();
    let mut out: Vec<(PlayerId, Cost)> = Vec::new();
    for b in blockers {
        if let Some(c) = required_block_cost(g, b) {
            let p = g.obj(b).controller;
            match out.iter_mut().find(|(q, _)| *q == p) {
                Some((_, t)) => crate::casting::add_cost(t, &c),
                None => out.push((p, c)),
            }
        }
    }
    out
}

/// Searches for a block obeying the maximum number of requirements without disobeying
/// restrictions and without paying costs (CR 509.1c).
pub fn best_blocks(
    g: &Game,
    options: &[(ObjectId, Vec<ObjectId>)],
    reqs: &[BlockRequirement],
) -> (u32, Vec<(ObjectId, ObjectId)>) {
    let rules = block_rules(g, options);
    // Per blocker, the sets of attackers it could block (not counting creatures that can
    // block only by paying a cost, CR 509.1c).
    let choices: Vec<(ObjectId, Vec<Vec<ObjectId>>)> = options
        .iter()
        .map(|(b, atts)| {
            let mut sets: Vec<Vec<ObjectId>> = Vec::new();
            if required_block_cost(g, *b).is_none() {
                let max = rules
                    .max_blocks
                    .get(b)
                    .copied()
                    .flatten()
                    .map(|m| m as usize)
                    .unwrap_or(atts.len());
                let n = atts.len().min(16);
                for mask in 1u32..(1u32 << n) {
                    if (mask.count_ones() as usize) <= max {
                        sets.push(
                            (0..n)
                                .filter(|i| mask & (1 << i) != 0)
                                .map(|i| atts[i])
                                .collect(),
                        );
                    }
                }
                sets.sort_by_key(|s| s.len());
            }
            (*b, sets)
        })
        .collect();
    let mut best = (obeyed_block_requirements(reqs, &[]), Vec::new());
    if reqs.is_empty() {
        // Nothing to maximize: not blocking is always legal.
        return best;
    }
    let space: f64 = choices.iter().map(|(_, s)| (s.len() + 1) as f64).product();
    if space <= 100_000.0 {
        // Requirements specific to blockers from index i on (for pruning).
        let mut suffix = vec![0u32; choices.len() + 1];
        for i in (0..choices.len()).rev() {
            let b = choices[i].0;
            let n = reqs
                .iter()
                .filter(|r| {
                    matches!(r, BlockRequirement::Blocks(x) | BlockRequirement::BlocksAttacker(x, _) if *x == b)
                })
                .count() as u32;
            suffix[i] = suffix[i + 1] + n;
        }
        let mut cur = Vec::new();
        block_dfs(
            g, &rules, options, &choices, reqs, &suffix, 0, &mut cur, &mut best,
        );
    } else {
        // Too many combinations: each creature with a requirement blocks a single attacker.
        let mut decl = Vec::new();
        for r in reqs {
            match r {
                BlockRequirement::Blocks(b) | BlockRequirement::BlocksAttacker(b, _) => {
                    if decl.iter().any(|(x, _)| x == b) {
                        continue;
                    }
                    let a = match r {
                        BlockRequirement::BlocksAttacker(_, a) => Some(*a),
                        _ => options
                            .iter()
                            .find(|(x, _)| x == b)
                            .and_then(|(_, atts)| atts.first().copied()),
                    };
                    if let Some(a) = a {
                        decl.push((*b, a));
                    }
                }
                BlockRequirement::AttackerBlocked(_) => {}
            }
        }
        let n = obeyed_block_requirements(reqs, &decl);
        if n > best.0 && block_restrictions_ok(g, &rules, options, &decl) {
            best = (n, decl);
        }
    }
    best
}

#[allow(clippy::too_many_arguments)]
fn block_dfs(
    g: &Game,
    rules: &BlockRules,
    options: &[(ObjectId, Vec<ObjectId>)],
    choices: &[(ObjectId, Vec<Vec<ObjectId>>)],
    reqs: &[BlockRequirement],
    suffix: &[u32],
    i: usize,
    cur: &mut Vec<(ObjectId, ObjectId)>,
    best: &mut (u32, Vec<(ObjectId, ObjectId)>),
) {
    let now = obeyed_block_requirements(reqs, cur);
    if i == choices.len() {
        if now > best.0 && block_restrictions_ok(g, rules, options, cur) {
            *best = (now, cur.clone());
        }
        return;
    }
    // Upper bound: blocker-specific requirements of the remaining blockers, plus
    // "must be blocked" requirements not yet obeyed.
    let open_attacker_reqs = reqs
        .iter()
        .filter(|r| matches!(r, BlockRequirement::AttackerBlocked(_)) && !r.obeyed(cur))
        .count() as u32;
    if now + suffix[i] + open_attacker_reqs <= best.0 {
        return;
    }
    block_dfs(g, rules, options, choices, reqs, suffix, i + 1, cur, best);
    let b = choices[i].0;
    for set in &choices[i].1 {
        let len = cur.len();
        cur.extend(set.iter().map(|a| (b, *a)));
        block_dfs(g, rules, options, choices, reqs, suffix, i + 1, cur, best);
        cur.truncate(len);
    }
}

/// Whether a proposed declaration of blockers is legal (CR 509.1a–c), not counting whether
/// its costs can be paid.
pub fn block_declaration_legal(
    g: &Game,
    options: &[(ObjectId, Vec<ObjectId>)],
    decl: &[(ObjectId, ObjectId)],
) -> bool {
    let reqs = block_requirements(g, options);
    let (max, _) = best_blocks(g, options, &reqs);
    let rules = block_rules(g, options);
    block_restrictions_ok(g, &rules, options, decl) && obeyed_block_requirements(&reqs, decl) >= max
}

/// Declare blockers step turn-based action (CR 509.1).
pub fn declare_blockers_step(g: &mut Game) {
    g.recompute();
    let Some(combat) = g.combat.clone() else {
        return;
    };
    // Each defending player declares blockers in APNAP order (CR 802.4); with shared team
    // turns, the defending team declares one combined block (CR 805.10d).
    let groups: Vec<Vec<PlayerId>> = if shared_team_turns(g) {
        let dps: Vec<PlayerId> = g
            .apnap()
            .into_iter()
            .filter(|p| combat.defending_players.contains(p))
            .collect();
        if dps.is_empty() {
            vec![]
        } else {
            vec![dps]
        }
    } else {
        g.apnap()
            .into_iter()
            .filter(|p| combat.defending_players.contains(p))
            .map(|p| vec![p])
            .collect()
    };
    let mut all_blocks: Vec<(ObjectId, ObjectId)> = Vec::new();
    for group in groups {
        let decider = group[0];
        let options = block_options(g, &group);
        if options.is_empty() {
            continue;
        }
        let reqs = block_requirements(g, &options);
        let (max, best) = best_blocks(g, &options, &reqs);
        let rules = block_rules(g, &options);
        let blocks = match g.ask(
            decider,
            Decision::DeclareBlockers {
                options: options.clone(),
            },
        ) {
            Answer::Blockers(v)
                if block_restrictions_ok(g, &rules, &options, &v)
                    && obeyed_block_requirements(&reqs, &v) >= max
                    && block_costs_by_player(g, &v)
                        .iter()
                        .all(|(p, c)| g.can_pay_cost(*p, c, None, &Ctx::new(None, *p))) =>
            {
                v
            }
            // CR 509.1: an illegal declaration is undone; the engine declares a legal one.
            _ => best.clone(),
        };
        // CR 509.1d–f: pay the locked-in costs to block.
        let costs = block_costs_by_player(g, &blocks);
        let blocks = if costs.is_empty() {
            blocks
        } else {
            let snapshot = g.clone();
            let ok = costs
                .iter()
                .all(|(p, c)| g.pay_cost(*p, c, None, &Ctx::new(None, *p)));
            if ok {
                blocks
            } else {
                let agents = g.agents.clone();
                *g = snapshot;
                g.agents = agents;
                best
            }
        };
        all_blocks.extend(blocks);
    }
    perform_block_declaration(g, all_blocks);
}

fn perform_block_declaration(g: &mut Game, blocks: Vec<(ObjectId, ObjectId)>) {
    crate::keyword_impls::pay_block_costs(g, &blocks);
    let defenders = g
        .combat
        .as_ref()
        .map(|c| c.defending_players.clone())
        .unwrap_or_default();
    // CR 509.1g: each chosen creature still controlled by a defending player blocks.
    let blocks: Vec<(ObjectId, ObjectId)> = blocks
        .into_iter()
        .filter(|(b, a)| {
            g.is_live(*b)
                && g.obj(*b).zone == Zone::Battlefield
                && defenders.contains(&g.obj(*b).controller)
                && g.is_attacking(*a)
        })
        .collect();
    let Some(c) = g.combat.as_mut() else { return };
    c.blockers_declared = true;
    for (b, a) in &blocks {
        if !c.declared_blockers.contains(b) {
            c.declared_blockers.push(*b);
        }
        match c.blockers.iter_mut().find(|x| x.id == *b) {
            Some(bi) => bi.blocking.push(*a),
            None => c.blockers.push(BlockerInfo {
                id: *b,
                blocking: vec![*a],
                declared: true,
            }),
        }
        // CR 509.1h: an attacker with blockers declared for it becomes blocked.
        if let Some(ai) = c.attackers.iter_mut().find(|x| x.id == *a) {
            ai.blocked = true;
            ai.blockers.push(*b);
        }
    }
    let attackers = c.attackers.clone();
    // CR 509.1i: abilities that trigger on blockers being declared.
    if !blocks.is_empty() {
        g.emit(Event::BlockersDeclared {
            blocks: blocks.clone(),
        });
    }
    for ai in attackers {
        if ai.blocked {
            g.emit(Event::BecameBlocked {
                attacker: ai.id,
                blockers: ai.blockers.clone(),
            });
        } else {
            g.emit(Event::AttackerUnblocked { attacker: ai.id });
        }
    }
    g.dirty = true;
}

// ---------------------------------------------------------------------------
// Combat damage (CR 510)
// ---------------------------------------------------------------------------

/// Whether any attacking or blocking creature has first strike or double strike (CR 510.4).
pub fn any_first_strike(g: &Game) -> bool {
    let Some(c) = &g.combat else { return false };
    c.attackers
        .iter()
        .map(|a| a.id)
        .chain(c.blockers.iter().map(|b| b.id))
        .any(|id| {
            let o = g.obj(id);
            o.has_keyword(KeywordKind::FirstStrike) || o.has_keyword(KeywordKind::DoubleStrike)
        })
}

/// Lethal damage for assignment purposes (CR 702.19b, 702.2c): toughness minus damage
/// already marked, or at most 1 if the source has deathtouch.
pub fn lethal_damage(g: &Game, source: ObjectId, creature: ObjectId) -> u32 {
    let o = g.obj(creature);
    let lethal = (o.toughness() - o.damage as i32).max(0) as u32;
    if g.obj(source).has_keyword(KeywordKind::Deathtouch) {
        // Any nonzero amount is lethal (and none is needed if it already has lethal
        // damage marked, CR 702.19b).
        return lethal.min(1);
    }
    lethal
}

/// Combat damage step (CR 510).
pub fn combat_damage_step(g: &mut Game, first_strike_step: bool) {
    g.recompute();
    if let Some(c) = g.combat.as_mut() {
        c.damage_step_began = true;
    }
    let Some(combat) = g.combat.clone() else {
        return;
    };
    let has_fs = |g: &Game, id: ObjectId| {
        let o = g.obj(id);
        o.has_keyword(KeywordKind::FirstStrike) || o.has_keyword(KeywordKind::DoubleStrike)
    };
    let participants: Vec<ObjectId> = combat
        .attackers
        .iter()
        .map(|a| a.id)
        .chain(combat.blockers.iter().map(|b| b.id))
        .collect();
    let deals: Vec<ObjectId> = if first_strike_step {
        let v: Vec<ObjectId> = participants
            .iter()
            .copied()
            .filter(|id| has_fs(g, *id))
            .collect();
        if let Some(c) = g.combat.as_mut() {
            c.first_strike_step = true;
            c.first_strikers = v.clone();
        }
        v
    } else if combat.first_strike_step {
        // CR 510.4: creatures without first/double strike as the first step began, plus
        // those that currently have double strike.
        participants
            .iter()
            .copied()
            .filter(|id| {
                !combat.first_strikers.contains(id)
                    || g.obj(*id).has_keyword(KeywordKind::DoubleStrike)
            })
            .collect()
    } else {
        participants.clone()
    };
    let mut assignments: Vec<(ObjectId, Entity, u32)> = Vec::new();
    // Attacking player assigns first, then defending players (CR 510.1).
    for ai in &combat.attackers {
        if !deals.contains(&ai.id) || !g.is_live(ai.id) || g.obj(ai.id).zone != Zone::Battlefield {
            continue;
        }
        assignments.extend(assign_attacker_damage(g, ai));
    }
    for bi in &combat.blockers {
        if !deals.contains(&bi.id) || !g.is_live(bi.id) || g.obj(bi.id).zone != Zone::Battlefield {
            continue;
        }
        assignments.extend(assign_blocker_damage(g, bi));
    }
    // CR 510.2: all combat damage is dealt simultaneously.
    crate::keyword_impls::before_combat_damage(g, &mut assignments);
    g.deal_damage_batch(assignments, true);
}

fn damage_amount(g: &Game, id: ObjectId) -> u32 {
    crate::keyword_impls::combat_damage_amount(g, id)
}

fn assign_attacker_damage(g: &mut Game, ai: &AttackerInfo) -> Vec<(ObjectId, Entity, u32)> {
    let id = ai.id;
    let power = damage_amount(g, id);
    if power == 0 {
        return vec![];
    }
    let controller = g.obj(id).controller;
    let trample = g.obj(id).has_keyword(KeywordKind::Trample);
    let target = ai.target.filter(|t| g.valid_damage_recipient(*t));
    if !ai.blocked {
        // CR 510.1b
        return match target {
            Some(t) => vec![(id, t, power)],
            None => vec![],
        };
    }
    let blockers: Vec<ObjectId> = ai
        .blockers
        .iter()
        .copied()
        .filter(|b| g.is_live(*b) && g.obj(*b).zone == Zone::Battlefield && g.is_blocking(*b))
        .collect();
    if blockers.is_empty() {
        // CR 510.1c: blocked with no blockers assigns no damage, unless trample (702.19e).
        return match (trample, target) {
            (true, Some(t)) => vec![(id, t, power)],
            _ => vec![],
        };
    }
    if crate::keyword_impls::assigns_as_though_unblocked(g, id) {
        if let Some(t) = target {
            return vec![(id, t, power)];
        }
    }
    let lethal: Vec<u32> = blockers.iter().map(|b| lethal_damage(g, id, *b)).collect();
    let mut recipients: Vec<Entity> = blockers.iter().map(|b| Entity::Object(*b)).collect();
    if trample {
        if let Some(t) = target {
            recipients.push(t);
        }
    }
    if blockers.len() == 1 && !trample {
        return vec![(id, Entity::Object(blockers[0]), power)];
    }
    let default = default_assignment(power, &lethal, trample && target.is_some());
    let ans = g.ask(
        controller,
        Decision::AssignCombatDamage {
            creature: id,
            amount: power,
            recipients: recipients.clone(),
            lethal: lethal.clone(),
            trample,
        },
    );
    let assignment: Vec<u32> = match ans {
        Answer::Numbers(v) if valid_assignment(&v, power, &lethal, trample && target.is_some()) => {
            v.into_iter().map(|x| x as u32).collect()
        }
        _ => default,
    };
    recipients
        .into_iter()
        .zip(assignment)
        .filter(|(_, n)| *n > 0)
        .map(|(r, n)| (id, r, n))
        .collect()
}

/// Default assignment: lethal to each blocker in order, remainder to the player if
/// trampling, else to the last blocker.
fn default_assignment(power: u32, lethal: &[u32], trample_to_player: bool) -> Vec<u32> {
    let mut left = power;
    let mut out: Vec<u32> = Vec::new();
    for l in lethal {
        let x = (*l).min(left);
        out.push(x);
        left -= x;
    }
    if trample_to_player {
        out.push(left);
    } else if left > 0 {
        if let Some(last) = out.last_mut() {
            *last += left;
        }
    }
    out
}

fn valid_assignment(v: &[i64], power: u32, lethal: &[u32], trample_to_player: bool) -> bool {
    let n = lethal.len() + usize::from(trample_to_player);
    if v.len() != n || v.iter().any(|x| *x < 0) || v.iter().sum::<i64>() != power as i64 {
        return false;
    }
    // CR 702.19b: damage can be assigned to the player only if all blockers are assigned lethal.
    if trample_to_player && v[n - 1] > 0 {
        for (i, l) in lethal.iter().enumerate() {
            if (v[i] as u32) < *l {
                return false;
            }
        }
    }
    true
}

fn assign_blocker_damage(g: &mut Game, bi: &BlockerInfo) -> Vec<(ObjectId, Entity, u32)> {
    let id = bi.id;
    let power = damage_amount(g, id);
    if power == 0 {
        return vec![];
    }
    let attackers: Vec<ObjectId> = bi
        .blocking
        .iter()
        .copied()
        .filter(|a| g.is_live(*a) && g.is_attacking(*a))
        .collect();
    match attackers.len() {
        0 => vec![], // CR 510.1d
        1 => vec![(id, Entity::Object(attackers[0]), power)],
        _ => {
            let controller = g.obj(id).controller;
            let lethal: Vec<u32> = attackers.iter().map(|a| lethal_damage(g, id, *a)).collect();
            let recipients: Vec<Entity> = attackers.iter().map(|a| Entity::Object(*a)).collect();
            let default = default_assignment(power, &lethal, false);
            let ans = g.ask(
                controller,
                Decision::AssignCombatDamage {
                    creature: id,
                    amount: power,
                    recipients: recipients.clone(),
                    lethal,
                    trample: false,
                },
            );
            let assignment: Vec<u32> = match ans {
                Answer::Numbers(v)
                    if v.len() == recipients.len()
                        && v.iter().all(|x| *x >= 0)
                        && v.iter().sum::<i64>() == power as i64 =>
                {
                    v.into_iter().map(|x| x as u32).collect()
                }
                _ => default,
            };
            recipients
                .into_iter()
                .zip(assignment)
                .filter(|(_, n)| *n > 0)
                .map(|(r, n)| (id, r, n))
                .collect()
        }
    }
}

// ---------------------------------------------------------------------------
// End of combat, removal from combat, entering attacking/blocking
// ---------------------------------------------------------------------------

/// End of combat: remove everything from combat (CR 511.3).
pub fn end_combat(g: &mut Game) {
    g.combat = None;
    g.dirty = true;
}

/// Removes a permanent from combat (CR 506.4).
pub fn remove_from_combat(g: &mut Game, id: ObjectId) {
    let dp = g
        .combat
        .as_ref()
        .and_then(|c| c.attacker(id).map(|_| c.defending_player_of(g, id)))
        .flatten();
    let Some(c) = g.combat.as_mut() else { return };
    if let Some(p) = dp {
        c.removed_attackers.push((id, p));
    }
    c.attackers.retain(|a| a.id != id);
    for a in c.attackers.iter_mut() {
        // CR 509.1h: an attacker remains blocked even if its blockers are removed.
        a.blockers.retain(|b| *b != id);
        // An attacked planeswalker/battle removed from combat (CR 506.4c).
        if a.target == Some(Entity::Object(id)) {
            a.target = None;
        }
    }
    c.blockers.retain(|b| b.id != id);
    for b in c.blockers.iter_mut() {
        b.blocking.retain(|a| *a != id);
    }
    g.dirty = true;
}

/// Stops a permanent from being a blocking creature while it continues to be attacked
/// (CR 506.4d).
fn stop_blocking(g: &mut Game, id: ObjectId) {
    let Some(c) = g.combat.as_mut() else { return };
    c.blockers.retain(|b| b.id != id);
    for a in c.attackers.iter_mut() {
        a.blockers.retain(|b| *b != id);
    }
}

/// Stops a permanent from being attacked while it continues to be a blocking creature
/// (CR 506.4c–e).
fn stop_being_attacked(g: &mut Game, id: ObjectId) {
    let Some(c) = g.combat.as_mut() else { return };
    for a in c.attackers.iter_mut() {
        if a.target == Some(Entity::Object(id)) {
            a.target = None;
        }
    }
}

/// Removes permanents from combat whose types or protector changed (CR 506.4, 506.4d–e).
/// Called whenever characteristics are recomputed.
pub fn update_combat_membership(g: &mut Game) {
    let Some(c) = &g.combat else { return };
    let attackers: Vec<ObjectId> = c.attackers.iter().map(|a| a.id).collect();
    let blockers: Vec<ObjectId> = c.blockers.iter().map(|b| b.id).collect();
    let mut attacked: Vec<(ObjectId, Option<PlayerId>)> = c
        .attackers
        .iter()
        .filter_map(|a| match a.target {
            Some(Entity::Object(o)) => Some((o, a.defending_player)),
            _ => None,
        })
        .collect();
    attacked.sort();
    attacked.dedup_by_key(|x| x.0);
    let alive = |g: &Game, id: ObjectId| g.is_live(id) && g.obj(id).zone == Zone::Battlefield;
    // Attacking creatures that stop being creatures or become battles.
    for id in attackers {
        if alive(g, id) {
            let o = g.obj(id);
            if !o.is_creature() || o.is(CardType::Battle) {
                remove_from_combat(g, id);
            }
        }
    }
    for (id, dp) in attacked {
        if !alive(g, id) {
            continue;
        }
        let o = g.obj(id);
        let pw = o.is(CardType::Planeswalker);
        let battle = o.is(CardType::Battle);
        let blocking_creature = g.is_blocking(id) && o.is_creature() && !battle;
        let protector = crate::battle::protector(g, id);
        let still_attacked = if battle {
            // CR 506.4: removed if its protector changes.
            dp.is_none() || protector == dp
        } else if pw {
            // CR 506.4e: stopped being a battle but still a planeswalker: removed only if
            // it isn't controlled by its protector (the defending player).
            dp.is_none_or(|p| o.controller == p)
        } else {
            false
        };
        if !still_attacked {
            if blocking_creature {
                stop_being_attacked(g, id); // CR 506.4d
            } else {
                remove_from_combat(g, id);
            }
        }
    }
    // Blocking creatures that stop being creatures or become battles.
    for id in blockers {
        if !alive(g, id) {
            continue;
        }
        let o = g.obj(id);
        if !o.is_creature() || o.is(CardType::Battle) {
            let attacked_pw = o.is(CardType::Planeswalker)
                && g.combat.as_ref().is_some_and(|c| {
                    c.attackers
                        .iter()
                        .any(|a| a.target == Some(Entity::Object(id)))
                });
            if attacked_pw {
                stop_blocking(g, id); // CR 506.4d
            } else {
                remove_from_combat(g, id);
            }
        }
    }
}

/// Puts a creature onto the battlefield attacking (CR 508.4): called as it enters.
pub fn put_onto_battlefield_attacking(g: &mut Game, id: ObjectId, target: Entity) {
    if g.combat.is_none() {
        return;
    }
    let o = g.obj(id);
    // CR 506.3b: only a creature controlled by an attacking player.
    if !attacking_players(g).contains(&o.controller) {
        return;
    }
    // CR 506.3a, 506.3f.
    if !o.is_creature() || o.is(CardType::Battle) {
        return;
    }
    // CR 506.3c, 508.4a.
    if !g.valid_attack_target(target) {
        return;
    }
    let dp = entity_defender(g, target);
    let Some(c) = g.combat.as_mut() else { return };
    c.any_attackers = true;
    // CR 508.4d: after blockers are declared it enters as an unblocked creature (blocked =
    // false); before, blockers haven't been declared yet.
    c.attackers.push(AttackerInfo {
        id,
        target: Some(target),
        original_target: Some(target),
        declared: false,
        blocked: false,
        blockers: vec![],
        band: None,
        defending_player: Some(dp),
    });
    g.dirty = true;
}

/// Chooses what a creature entering attacking attacks when the effect doesn't say (CR
/// 508.4): its controller chooses a defending player, a planeswalker a defending player
/// controls, or a battle a defending player protects.
pub fn choose_attack_target_for_new_attacker(g: &mut Game, controller: PlayerId) -> Option<Entity> {
    let targets: Vec<Entity> = attack_targets(g)
        .into_iter()
        .filter(|t| g.valid_attack_target(*t))
        .collect();
    if targets.len() <= 1 {
        return targets.first().copied();
    }
    g.ask_entities(
        controller,
        None,
        "Choose what the creature is attacking",
        targets,
        1,
        1,
    )
    .first()
    .copied()
}

/// Puts a creature onto the battlefield blocking `attacker` (CR 509.4): called as it enters.
pub fn put_onto_battlefield_blocking(g: &mut Game, id: ObjectId, attacker: ObjectId) {
    // CR 506.3a, 506.3f: only a creature that isn't a battle.
    let o = g.obj(id);
    if !o.is_creature() || o.is(CardType::Battle) {
        return;
    }
    // CR 509.4a, 506.3e: the attacker must still be attacking the creature's controller
    // (or a planeswalker/battle of theirs).
    if !g.is_attacking(attacker) || !g.could_block_pair(id, attacker) {
        return;
    }
    add_block(g, id, attacker, true);
}

/// Chooses which attacking creature a creature entering the battlefield blocking will block
/// when the effect doesn't say (CR 509.4): its controller chooses among creatures attacking
/// them, a planeswalker they control, or a battle they protect.
pub fn choose_attacker_to_block(g: &mut Game, controller: PlayerId) -> Option<ObjectId> {
    let cands: Vec<ObjectId> = g
        .attackers()
        .into_iter()
        .filter(|a| {
            g.combat
                .as_ref()
                .and_then(|c| c.defending_player_of(g, *a))
                .is_some_and(|dp| {
                    dp == controller || (shared_team_turns(g) && !g.are_opponents(dp, controller))
                })
        })
        .collect();
    if cands.len() <= 1 {
        return cands.first().copied();
    }
    g.ask_objects(
        controller,
        None,
        "Choose the attacking creature it blocks",
        cands,
        1,
        1,
    )
    .first()
    .copied()
}

/// Makes an existing creature block `attacker` because an effect says so (CR 509.3a–b,
/// 509.3d). Returns false if it can't (CR 506.3g).
pub fn block_by_effect(g: &mut Game, blocker: ObjectId, attacker: ObjectId) -> bool {
    let o = g.obj(blocker);
    if !g.is_live(blocker)
        || o.zone != Zone::Battlefield
        || !o.is_creature()
        || o.is(CardType::Battle)
        || !g.is_attacking(attacker)
        || !g.could_block_pair(blocker, attacker)
        || g.combat
            .as_ref()
            .is_some_and(|c| c.blockers_of(attacker).contains(&blocker))
    {
        return false;
    }
    add_block(g, blocker, attacker, false);
    true
}

fn add_block(g: &mut Game, blocker: ObjectId, attacker: ObjectId, entered: bool) {
    let Some(c) = g.combat.as_mut() else { return };
    let was_blocking = c.blockers.iter().any(|b| b.id == blocker);
    let was_blocked = c.is_blocked(attacker);
    match c.blockers.iter_mut().find(|x| x.id == blocker) {
        Some(bi) => bi.blocking.push(attacker),
        None => c.blockers.push(BlockerInfo {
            id: blocker,
            blocking: vec![attacker],
            declared: false,
        }),
    }
    if let Some(ai) = c.attackers.iter_mut().find(|x| x.id == attacker) {
        ai.blocked = true;
        ai.blockers.push(blocker);
    }
    g.emit(Event::BlockAdded {
        blocker,
        attacker,
        entered,
        was_blocking,
        was_blocked,
    });
    if !was_blocked {
        g.emit(Event::BecameBlocked {
            attacker,
            blockers: vec![blocker],
        });
    }
    g.dirty = true;
}

/// "[attacking creature] becomes unblocked" (CR 509.1h): an effect makes a blocked
/// attacking creature an unblocked creature. Returns true if it became unblocked.
pub fn become_unblocked(g: &mut Game, attacker: ObjectId) -> bool {
    let Some(c) = g.combat.as_mut() else {
        return false;
    };
    let Some(ai) = c.attackers.iter_mut().find(|x| x.id == attacker) else {
        return false;
    };
    if !ai.blocked {
        return false;
    }
    ai.blocked = false;
    g.dirty = true;
    true
}

/// "[attacking creature] becomes blocked" (CR 509.1h): an effect makes an attacking
/// creature blocked without any creature blocking it. Returns true if it became blocked.
pub fn become_blocked(g: &mut Game, attacker: ObjectId) -> bool {
    let Some(c) = g.combat.as_mut() else {
        return false;
    };
    let Some(ai) = c.attackers.iter_mut().find(|x| x.id == attacker) else {
        return false;
    };
    if ai.blocked {
        return false;
    }
    ai.blocked = true;
    g.emit(Event::BecameBlocked {
        attacker,
        blockers: vec![],
    });
    g.dirty = true;
    true
}

/// Reselects what an attacking creature is attacking (CR 508.7). The creature isn't
/// affected by requirements or restrictions on declaring attackers (CR 508.7b) and isn't
/// considered to have attacked again (CR 508.7a). Returns false if the choice is illegal.
pub fn reselect_attack_target(g: &mut Game, attacker: ObjectId, new: Entity) -> bool {
    if !g.is_attacking(attacker) {
        return false;
    }
    let ctl = g.obj(attacker).controller;
    let defender = match new {
        Entity::Player(p) => p,
        Entity::Object(o) => {
            let ob = g.obj(o);
            if !g.is_live(o) || ob.zone != Zone::Battlefield {
                return false;
            }
            if ob.is(CardType::Battle) {
                match crate::battle::protector(g, o) {
                    Some(p) => p,
                    None => return false,
                }
            } else if ob.is(CardType::Planeswalker) {
                ob.controller
            } else {
                return false;
            }
        }
    };
    // CR 508.7c: an opponent of the attacking creature's controller (or their permanent).
    if !g.player(defender).in_game() || !g.are_opponents(ctl, defender) {
        return false;
    }
    // CR 508.7d: without the attack multiple players option, the chosen defending player.
    let multiplayer = g.players.len() > 2;
    if multiplayer
        && !g.config.attack_multiple_players
        && !shared_team_turns(g)
        && !g
            .combat
            .as_ref()
            .is_some_and(|c| c.defending_players.contains(&defender))
    {
        return false;
    }
    // CR 508.7e: within the controller's range of influence (a planeswalker via its
    // controller; for a battle, its protector must be within range).
    if !within_range(g, ctl, defender) {
        return false;
    }
    let Some(c) = g.combat.as_mut() else {
        return false;
    };
    if let Some(ai) = c.attackers.iter_mut().find(|a| a.id == attacker) {
        ai.target = Some(new);
        ai.defending_player = Some(defender);
    }
    g.dirty = true;
    true
}

// ---------------------------------------------------------------------------
// Combat timing windows (CR 506.8)
// ---------------------------------------------------------------------------

/// Position of a step in the turn, for combat timing comparisons.
fn step_ord(s: Step) -> u8 {
    match s {
        Step::Untap => 0,
        Step::Upkeep => 1,
        Step::Draw => 2,
        Step::PrecombatMain => 3,
        Step::BeginningOfCombat => 4,
        Step::DeclareAttackers => 5,
        Step::DeclareBlockers => 6,
        Step::FirstStrikeDamage | Step::CombatDamage => 7,
        Step::EndOfCombat => 8,
        Step::PostcombatMain => 9,
        Step::End => 10,
        Step::Cleanup => 11,
    }
}

fn point_ord(p: CombatPoint) -> u8 {
    match p {
        CombatPoint::Combat => 4,
        CombatPoint::AttackersDeclared => 5,
        CombatPoint::BlockersDeclared => 6,
        CombatPoint::CombatDamageStep => 7,
        CombatPoint::EndOfCombatStep => 8,
    }
}

/// Whether a spell or ability with a combat timing restriction may be cast/activated now
/// (CR 506.8, 506.8a–g).
pub fn combat_timing_ok(g: &Game, t: CombatTiming) -> bool {
    let now = step_ord(g.turn.step);
    let p = point_ord(t.point);
    if t.during_combat {
        // CR 506.8c: "during combat": any combat phase, relative to the current one.
        if !g.turn.step.is_combat() {
            return false;
        }
        if !t.after {
            // "during combat before [point]": skipped steps count as passed (CR 506.8e).
            return now < p;
        }
        // "during combat after [point]": the point must actually have happened in this
        // combat (CR 506.8f).
        let c = g.combat.as_ref();
        return match t.point {
            CombatPoint::Combat => true,
            CombatPoint::AttackersDeclared => now >= p,
            CombatPoint::BlockersDeclared => c.is_some_and(|c| c.blockers_declared),
            CombatPoint::CombatDamageStep => c.is_some_and(|c| c.damage_step_began),
            CombatPoint::EndOfCombatStep => g.turn.step == Step::EndOfCombat,
        };
    }
    // CR 506.8d: relative to the first combat phase of the turn. A point is passed once
    // the turn has reached it or any later position; if the combat's later steps or the
    // whole combat phase are skipped, they're passed when the turn moves on (CR 506.8e).
    let reached = |ord: u8| g.turn.step_log.iter().any(|s| step_ord(*s) >= ord) || now >= ord;
    if !t.after {
        return !reached(p);
    }
    match t.point {
        // "after combat": after the (first) combat phase has ended.
        CombatPoint::Combat => {
            g.turn
                .step_log
                .iter()
                .any(|s| step_ord(*s) >= 9 || *s == Step::EndOfCombat)
                && !(g.turn.step == Step::EndOfCombat && g.turn.combat_phases <= 1)
        }
        _ => reached(p),
    }
}

/// Whether the card's own "cast this spell only ..." restrictions allow casting it now
/// (CR 506.8, 601.3).
pub fn spell_cast_restrictions_ok(
    g: &Game,
    p: PlayerId,
    card: ObjectId,
    chars: &crate::object::Characteristics,
) -> bool {
    chars.abilities.iter().all(|a| match &a.kind {
        AbilityKind::Static(s) => match &s.effect {
            StaticEffect::CastOnlyIf(c) => g.eval_cond(c, &Ctx::new(Some(card), p)),
            _ => true,
        },
        _ => true,
    })
}

// ---------------------------------------------------------------------------
// Combat filters and triggers
// ---------------------------------------------------------------------------

/// Evaluates the combat object filters "attacking alone", "blocking alone", and "had to
/// attack" (CR 506.5, 506.7).
pub fn combat_filter(g: &Game, f: &Filter, id: ObjectId) -> bool {
    let Some(c) = &g.combat else { return false };
    match f {
        Filter::AttackingAlone => c.attackers.len() == 1 && c.attackers[0].id == id,
        Filter::BlockingAlone => c.blockers.len() == 1 && c.blockers[0].id == id,
        Filter::AttackingPlayerAlone => match c.attack_target(id) {
            Some(t @ Entity::Player(_)) => {
                c.attackers.iter().filter(|a| a.target == Some(t)).count() == 1
            }
            _ => false,
        },
        Filter::HadToAttack => c.had_to_attack.contains(&id),
        _ => false,
    }
}

fn recipient_matches(g: &Game, r: &DamageRecipient, t: Entity, ctx: &Ctx) -> bool {
    match (r, t) {
        (DamageRecipient::Any, _) => true,
        (DamageRecipient::Player(rel), Entity::Player(p)) => g.player_rel_matches(*rel, p, ctx),
        (DamageRecipient::PlayerOrPlaneswalker(rel), Entity::Player(p)) => {
            g.player_rel_matches(*rel, p, ctx)
        }
        (DamageRecipient::PlayerOrPlaneswalker(rel), Entity::Object(o)) => {
            g.obj(o).is(CardType::Planeswalker)
                && g.player_rel_matches(*rel, g.obj(o).controller, ctx)
        }
        (DamageRecipient::Object(f), Entity::Object(o)) => g.matches(o, f, ctx),
        _ => false,
    }
}

/// Trigger matching for combat trigger conditions (CR 506.5–6, 508.3, 509.3) and for
/// blocks added by effects or by creatures entering blocking. Returns `None` if the
/// condition/event pair isn't handled here.
pub fn combat_trigger_matches(
    g: &Game,
    cond: &TriggerCond,
    ctx: &Ctx,
    ev: &Event,
) -> Option<Vec<EventInfo>> {
    match (cond, ev) {
        (TriggerCond::AttacksAlone(f), Event::AttackersDeclared { attackers, .. }) => {
            let v = if attackers.len() == 1 && g.matches(attackers[0].0, f, ctx) {
                vec![EventInfo {
                    object: Some(attackers[0].0),
                    player: Some(entity_defender(g, attackers[0].1)),
                    other: attackers[0].1.object(),
                    ..Default::default()
                }]
            } else {
                vec![]
            };
            Some(v)
        }
        (TriggerCond::AttacksPlayerAlone(f), Event::AttackersDeclared { attackers, .. }) => Some(
            attackers
                .iter()
                .filter(|(a, t)| {
                    matches!(t, Entity::Player(_))
                        && attackers.iter().filter(|(_, u)| u == t).count() == 1
                        && g.matches(*a, f, ctx)
                })
                .map(|(a, t)| EventInfo {
                    object: Some(*a),
                    player: t.player(),
                    ..Default::default()
                })
                .collect(),
        ),
        (
            TriggerCond::AttacksRecipient {
                attacker,
                recipient,
            },
            Event::AttackersDeclared { attackers, .. },
        ) => Some(
            attackers
                .iter()
                .filter(|(a, t)| {
                    g.matches(*a, attacker, ctx) && recipient_matches(g, recipient, *t, ctx)
                })
                .map(|(a, t)| EventInfo {
                    object: Some(*a),
                    player: Some(entity_defender(g, *t)),
                    other: t.object(),
                    ..Default::default()
                })
                .collect(),
        ),
        (TriggerCond::IsAttacked(recipient), Event::AttackersDeclared { attackers, .. }) => {
            let mut targets: Vec<Entity> = Vec::new();
            for (_, t) in attackers {
                if !targets.contains(t) {
                    targets.push(*t);
                }
            }
            Some(
                targets
                    .into_iter()
                    .filter(|t| recipient_matches(g, recipient, *t, ctx))
                    .map(|t| {
                        let objs: Vec<ObjectId> = attackers
                            .iter()
                            .filter(|(_, u)| *u == t)
                            .map(|(a, _)| *a)
                            .collect();
                        EventInfo {
                            player: Some(entity_defender(g, t)),
                            other: t.object(),
                            amount: objs.len() as i32,
                            objects: objs,
                            ..Default::default()
                        }
                    })
                    .collect(),
            )
        }
        (
            TriggerCond::PlayerAttacksWith { who, filter, min },
            Event::AttackersDeclared { attackers, .. },
        ) => {
            let mut by: BTreeMap<PlayerId, Vec<ObjectId>> = BTreeMap::new();
            for (a, _) in attackers {
                if g.matches(*a, filter, ctx) {
                    by.entry(g.obj(*a).controller).or_default().push(*a);
                }
            }
            Some(
                by.into_iter()
                    .filter(|(p, v)| {
                        v.len() as u32 >= (*min).max(1) && g.player_rel_matches(*who, *p, ctx)
                    })
                    .map(|(p, v)| EventInfo {
                        player: Some(p),
                        amount: v.len() as i32,
                        objects: v,
                        ..Default::default()
                    })
                    .collect(),
            )
        }
        (
            TriggerCond::PlayerAttacksPlayer { attacker, defender },
            Event::AttackersDeclared { attackers, .. },
        ) => {
            let mut pairs: BTreeMap<(PlayerId, PlayerId), Vec<ObjectId>> = BTreeMap::new();
            for (a, t) in attackers {
                // CR 508.3e: attacks on planeswalkers and battles don't count.
                if let Entity::Player(p) = t {
                    pairs
                        .entry((g.obj(*a).controller, *p))
                        .or_default()
                        .push(*a);
                }
            }
            Some(
                pairs
                    .into_iter()
                    .filter(|((ap, dp), _)| {
                        g.player_rel_matches(*attacker, *ap, ctx)
                            && g.player_rel_matches(*defender, *dp, ctx)
                    })
                    .map(|((_, dp), v)| EventInfo {
                        player: Some(dp),
                        amount: v.len() as i32,
                        objects: v,
                        ..Default::default()
                    })
                    .collect(),
            )
        }
        (TriggerCond::BlocksCreature { blocker, attacker }, Event::BlockersDeclared { blocks }) => {
            Some(
                blocks
                    .iter()
                    .filter(|(b, a)| g.matches(*b, blocker, ctx) && g.matches(*a, attacker, ctx))
                    .map(|(b, a)| EventInfo {
                        object: Some(*a),
                        other: Some(*b),
                        player: Some(g.obj(*a).controller),
                        ..Default::default()
                    })
                    .collect(),
            )
        }
        (
            TriggerCond::BlockedByCreature { attacker, blocker },
            Event::BlockersDeclared { blocks },
        ) => Some(
            blocks
                .iter()
                .filter(|(b, a)| g.matches(*a, attacker, ctx) && g.matches(*b, blocker, ctx))
                .map(|(b, a)| EventInfo {
                    object: Some(*b),
                    other: Some(*a),
                    player: Some(g.obj(*b).controller),
                    ..Default::default()
                })
                .collect(),
        ),
        (TriggerCond::BlockedByN { attacker, n }, Event::BlockersDeclared { blocks }) => {
            let mut per: BTreeMap<ObjectId, Vec<ObjectId>> = BTreeMap::new();
            for (b, a) in blocks {
                per.entry(*a).or_default().push(*b);
            }
            Some(
                per.into_iter()
                    .filter(|(a, bs)| bs.len() as u32 >= *n && g.matches(*a, attacker, ctx))
                    .map(|(a, bs)| EventInfo {
                        object: Some(a),
                        amount: bs.len() as i32,
                        objects: bs,
                        ..Default::default()
                    })
                    .collect(),
            )
        }
        // Blocks added by effects or by creatures entering blocking (CR 509.3a–e, 509.4).
        (
            cond,
            Event::BlockAdded {
                blocker: b,
                attacker: a,
                entered,
                was_blocking,
                was_blocked,
            },
        ) => {
            let (b, a) = (*b, *a);
            let blocker_side = EventInfo {
                object: Some(b),
                other: Some(a),
                ..Default::default()
            };
            match cond {
                // CR 509.3a: only if it wasn't already blocking; never when entering.
                TriggerCond::Blocks(f) => {
                    Some(if !entered && !was_blocking && g.matches(b, f, ctx) {
                        vec![blocker_side]
                    } else {
                        vec![]
                    })
                }
                // CR 509.3b: once per attacker; never when entering.
                TriggerCond::BlocksCreature { blocker, attacker } => Some(
                    if !entered && g.matches(b, blocker, ctx) && g.matches(a, attacker, ctx) {
                        vec![EventInfo {
                            object: Some(a),
                            other: Some(b),
                            player: Some(g.obj(a).controller),
                            ..Default::default()
                        }]
                    } else {
                        vec![]
                    },
                ),
                // CR 509.3d: also when a creature is put onto the battlefield blocking.
                TriggerCond::BlockedByCreature { attacker, blocker } => Some(
                    if g.matches(a, attacker, ctx) && g.matches(b, blocker, ctx) {
                        vec![EventInfo {
                            object: Some(b),
                            other: Some(a),
                            player: Some(g.obj(b).controller),
                            ..Default::default()
                        }]
                    } else {
                        vec![]
                    },
                ),
                // CR 509.3e: effects that add blockers can make the count reach N.
                TriggerCond::BlockedByN { attacker, n } => {
                    let count = g
                        .combat
                        .as_ref()
                        .map(|c| c.blockers_of(a).len() as u32)
                        .unwrap_or(0);
                    Some(
                        if count >= *n
                            && count.saturating_sub(1) < *n
                            && g.matches(a, attacker, ctx)
                        {
                            vec![EventInfo {
                                object: Some(a),
                                amount: count as i32,
                                ..Default::default()
                            }]
                        } else {
                            vec![]
                        },
                    )
                }
                TriggerCond::BlocksOrBecomesBlocked(f) => {
                    let mut v = Vec::new();
                    if !entered && !was_blocking && g.matches(b, f, ctx) {
                        v.push(blocker_side);
                    }
                    if !was_blocked && g.matches(a, f, ctx) {
                        v.push(EventInfo {
                            object: Some(a),
                            other: Some(b),
                            ..Default::default()
                        });
                    }
                    Some(v)
                }
                _ => None,
            }
        }
        _ => None,
    }
}
