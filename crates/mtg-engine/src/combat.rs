//! Combat (CR 506–511): declaring attackers and blockers, evasion and combat
//! restrictions/requirements, combat damage assignment (including trample, first strike,
//! double strike, deathtouch), and removal from combat.

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttackerInfo {
    pub id: ObjectId,
    /// What it's attacking (player, planeswalker, or battle). `None` if that was removed.
    pub target: Option<Entity>,
    /// What it was attacking when declared (CR 508.5).
    pub original_target: Option<Entity>,
    /// Declared as an attacker (vs put onto the battlefield attacking, CR 508.4).
    pub declared: bool,
    pub blocked: bool,
    /// Blocking creatures in the order they were declared.
    pub blockers: Vec<ObjectId>,
    /// Band this attacker is part of (CR 702.22).
    pub band: Option<u32>,
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
    /// Defending player for an attacking creature (CR 508.5).
    pub fn defending_player_of(&self, g: &Game, attacker: ObjectId) -> Option<PlayerId> {
        let a = self.attacker(attacker)?;
        let t = a.target.or(a.original_target)?;
        Some(match t {
            Entity::Player(p) => p,
            Entity::Object(o) => {
                let ob = g.obj(o);
                if ob.is(CardType::Battle) {
                    crate::battle::protector(g, o).unwrap_or(ob.controller)
                } else {
                    ob.controller
                }
            }
        })
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

    fn restricted_obj(&self, id: ObjectId, pick: impl Fn(&Restriction) -> Option<&Filter>) -> bool {
        self.all_restrictions()
            .iter()
            .any(|(s, c, r, locked)| match pick(r) {
                Some(f) => {
                    locked.as_ref().is_none_or(|v| v.contains(&id))
                        && self.matches(id, f, &Ctx::new(*s, *c))
                }
                None => false,
            })
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
        let defender = match target {
            Entity::Player(p) => p,
            Entity::Object(o) => self.obj(o).controller,
        };
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
        let b = self.obj(blocker);
        let a = self.obj(attacker);
        // The attacker must be attacking the blocker's controller or their permanents.
        if let Some(c) = &self.combat {
            if c.defending_player_of(self, attacker) != Some(b.controller) {
                return false;
            }
        }
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
            let lock_ok = |id: ObjectId| locked.as_ref().is_none_or(|v| v.contains(&id));
            match &r {
                Restriction::CantBeBlocked(f)
                    if lock_ok(attacker) && self.matches(attacker, f, &ctx) =>
                {
                    return false
                }
                Restriction::CantBeBlockedBy {
                    attacker: af,
                    blocker: bf,
                } if lock_ok(attacker)
                    && self.matches(attacker, af, &ctx)
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

    /// Maximum number of creatures that can block an attacker ("can't be blocked by more
    /// than one creature"), if limited (CR 509.1b).
    pub fn max_blockers(&self, attacker: ObjectId) -> Option<u32> {
        let mut max: Option<u32> = None;
        for (s, c, r, locked) in self.all_restrictions() {
            if let Restriction::MaxBlockers { attacker: af, n } = &r {
                if locked.as_ref().is_none_or(|v| v.contains(&attacker))
                    && self.matches(attacker, af, &Ctx::new(s, c))
                {
                    max = Some(max.map_or(*n, |m| m.min(*n)));
                }
            }
        }
        max
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
}

/// Beginning of combat (CR 507): set up combat and choose the defending player.
pub fn begin_combat(g: &mut Game) {
    let ap = g.turn.active;
    let opponents = g.opponents(ap);
    let defending = if opponents.len() <= 1 || g.config.attack_multiple_players {
        opponents.clone()
    } else {
        // CR 507.1 / 703.4h: the active player chooses one opponent.
        let cands: Vec<Entity> = opponents.iter().map(|p| Entity::Player(*p)).collect();
        g.ask_entities(ap, None, "Choose the defending player", cands, 1, 1)
            .into_iter()
            .filter_map(|e| e.player())
            .collect()
    };
    g.combat = Some(CombatState {
        attacking_player: Some(ap),
        defending_players: defending,
        ..Default::default()
    });
}

/// Possible attack targets for creatures of the attacking player (CR 508.1b).
pub fn attack_targets(g: &Game) -> Vec<Entity> {
    let Some(c) = &g.combat else { return vec![] };
    let mut out = Vec::new();
    for p in &c.defending_players {
        out.push(Entity::Player(*p));
        for o in g.permanents() {
            if o.is(CardType::Planeswalker) && o.controller == *p {
                out.push(Entity::Object(o.id));
            }
            if o.is(CardType::Battle) && crate::battle::protector(g, o.id) == Some(*p) {
                out.push(Entity::Object(o.id));
            }
        }
    }
    out
}

/// Declare attackers step turn-based action (CR 508.1).
pub fn declare_attackers_step(g: &mut Game) {
    if g.combat.is_none() {
        begin_combat(g);
    }
    g.recompute();
    let ap = g.turn.active;
    let targets = attack_targets(g);
    let candidates: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.controller == ap)
        .map(|o| o.id)
        .filter(|id| g.can_attack(*id))
        .collect();
    let options: Vec<(ObjectId, Vec<Entity>)> = candidates
        .iter()
        .map(|c| {
            (
                *c,
                targets
                    .iter()
                    .copied()
                    .filter(|t| g.can_attack_target(*c, *t))
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, t)| !t.is_empty())
        .collect();
    let requirements = attack_requirements(g, &options);
    let declared: Vec<(ObjectId, Entity)> = if options.is_empty() {
        vec![]
    } else {
        match g.ask(
            ap,
            Decision::DeclareAttackers {
                options: options.clone(),
            },
        ) {
            Answer::Attackers(v) if attack_declaration_legal(g, &options, &v, &requirements) => v,
            _ => default_attack(g, &options, &requirements),
        }
    };
    perform_attack_declaration(g, ap, declared, &requirements);
}

/// Creatures that must attack if able (CR 508.1d), including goad (CR 701.15b).
fn attack_requirements(g: &Game, options: &[(ObjectId, Vec<Entity>)]) -> Vec<ObjectId> {
    let mut out = Vec::new();
    for (id, _) in options {
        let must = g.restricted_obj(*id, |r| match r {
            Restriction::MustAttack(f) => Some(f),
            _ => None,
        }) || !g.obj(*id).goaded_by.is_empty();
        if must {
            out.push(*id);
        }
    }
    out
}

fn attack_declaration_legal(
    g: &Game,
    options: &[(ObjectId, Vec<Entity>)],
    decl: &[(ObjectId, Entity)],
    requirements: &[ObjectId],
) -> bool {
    let mut seen = Vec::new();
    for (a, t) in decl {
        if seen.contains(a) {
            return false;
        }
        seen.push(*a);
        match options.iter().find(|(id, _)| id == a) {
            Some((_, ts)) if ts.contains(t) => {}
            _ => return false,
        }
        // Goaded creatures attack a player other than the goader if able (CR 701.15b).
        let goaders = &g.obj(*a).goaded_by;
        if !goaders.is_empty() {
            if let Entity::Player(p) = t {
                if goaders.contains(p) {
                    let ts = &options.iter().find(|(id, _)| id == a).unwrap().1;
                    if ts
                        .iter()
                        .any(|x| matches!(x, Entity::Player(q) if !goaders.contains(q)))
                    {
                        return false;
                    }
                }
            }
        }
    }
    // CR 508.1d: every "attacks if able" requirement that can be obeyed must be.
    for r in requirements {
        if !decl.iter().any(|(a, _)| a == r) {
            return false;
        }
    }
    crate::keyword_impls::attack_declaration_extra_checks(g, decl)
}

fn default_attack(
    g: &Game,
    options: &[(ObjectId, Vec<Entity>)],
    requirements: &[ObjectId],
) -> Vec<(ObjectId, Entity)> {
    let mut out = Vec::new();
    for r in requirements {
        if let Some((_, ts)) = options.iter().find(|(id, _)| id == r) {
            let goaders = &g.obj(*r).goaded_by;
            let t = ts
                .iter()
                .find(|t| !matches!(t, Entity::Player(p) if goaders.contains(p)))
                .or(ts.first())
                .copied();
            if let Some(t) = t {
                out.push((*r, t));
            }
        }
    }
    out
}

fn perform_attack_declaration(
    g: &mut Game,
    ap: PlayerId,
    declared: Vec<(ObjectId, Entity)>,
    requirements: &[ObjectId],
) {
    // CR 508.1f: tap attackers (vigilance: 702.20b).
    for (a, _) in &declared {
        if !g.obj(*a).has_keyword(KeywordKind::Vigilance) {
            g.tap(*a);
        }
    }
    // Costs to attack (CR 508.1g–j) are handled by keyword/rule modules.
    crate::keyword_impls::pay_attack_costs(g, ap, &declared);
    let c = g.combat.get_or_insert_with(CombatState::default);
    c.attackers_declared = true;
    c.had_to_attack = requirements.to_vec();
    for (a, t) in &declared {
        c.attackers.push(AttackerInfo {
            id: *a,
            target: Some(*t),
            original_target: Some(*t),
            declared: true,
            blocked: false,
            blockers: vec![],
            band: None,
        });
    }
    if !declared.is_empty() {
        g.log(|_g| format!("{ap} attacks with {} creature(s)", declared.len()));
        g.emit(Event::AttackersDeclared {
            player: ap,
            attackers: declared,
        });
    }
    g.dirty = true;
}

/// Declare blockers step turn-based action (CR 509.1).
pub fn declare_blockers_step(g: &mut Game) {
    g.recompute();
    let Some(combat) = g.combat.clone() else {
        return;
    };
    let attackers: Vec<ObjectId> = combat.attackers.iter().map(|a| a.id).collect();
    let mut all_blocks: Vec<(ObjectId, ObjectId)> = Vec::new();
    for dp in g.apnap() {
        if !combat.defending_players.contains(&dp) {
            continue;
        }
        let blockers: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.controller == dp)
            .map(|o| o.id)
            .filter(|id| g.can_block_at_all(*id))
            .collect();
        let options: Vec<(ObjectId, Vec<ObjectId>)> = blockers
            .iter()
            .map(|b| {
                (
                    *b,
                    attackers
                        .iter()
                        .copied()
                        .filter(|a| g.can_block(*b, *a))
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, a)| !a.is_empty())
            .collect();
        if options.is_empty() {
            continue;
        }
        let blocks = match g.ask(
            dp,
            Decision::DeclareBlockers {
                options: options.clone(),
            },
        ) {
            Answer::Blockers(v) if block_declaration_legal(g, &options, &v) => v,
            _ => default_blocks(g, &options),
        };
        all_blocks.extend(blocks);
    }
    perform_block_declaration(g, all_blocks);
}

fn block_requirements(g: &Game, options: &[(ObjectId, Vec<ObjectId>)]) -> Vec<ObjectId> {
    options
        .iter()
        .filter(|(b, _)| {
            g.restricted_obj(*b, |r| match r {
                Restriction::MustBlock(f) => Some(f),
                _ => None,
            })
        })
        .map(|(b, _)| *b)
        .collect()
}

pub fn block_declaration_legal(
    g: &Game,
    options: &[(ObjectId, Vec<ObjectId>)],
    decl: &[(ObjectId, ObjectId)],
) -> bool {
    // Each block must be allowed; each blocker within its block limit.
    let mut per_blocker: std::collections::BTreeMap<ObjectId, u32> = Default::default();
    let mut seen = Vec::new();
    for (b, a) in decl {
        if seen.contains(&(*b, *a)) {
            return false;
        }
        seen.push((*b, *a));
        match options.iter().find(|(id, _)| id == b) {
            Some((_, atts)) if atts.contains(a) => {}
            _ => return false,
        }
        *per_blocker.entry(*b).or_insert(0) += 1;
    }
    for (b, n) in &per_blocker {
        if let Some(max) = g.max_blocks(*b) {
            if *n > max {
                return false;
            }
        }
    }
    // Menace / minimum blockers (CR 702.110b): an attacker blocked by fewer is illegal.
    let mut per_attacker: std::collections::BTreeMap<ObjectId, u32> = Default::default();
    for (_, a) in decl {
        *per_attacker.entry(*a).or_insert(0) += 1;
    }
    for (a, n) in &per_attacker {
        if *n < g.min_blockers(*a) {
            return false;
        }
        if g.max_blockers(*a).is_some_and(|max| *n > max) {
            return false;
        }
    }
    // Requirements (CR 509.1c), simplified: each "must block" creature that can block
    // without violating restrictions must block.
    for r in block_requirements(g, options) {
        if !decl.iter().any(|(b, _)| *b == r) {
            // It's acceptable only if it can't legally block anything given menace etc.
            let can = options
                .iter()
                .find(|(id, _)| *id == r)
                .is_some_and(|(_, atts)| {
                    atts.iter().any(|a| {
                        g.min_blockers(*a) <= 1 || per_attacker.get(a).copied().unwrap_or(0) >= 1
                    })
                });
            if can {
                return false;
            }
        }
    }
    // Attackers that must be blocked (lure).
    crate::keyword_impls::block_declaration_extra_checks(g, options, decl)
}

fn default_blocks(g: &Game, options: &[(ObjectId, Vec<ObjectId>)]) -> Vec<(ObjectId, ObjectId)> {
    let mut out = Vec::new();
    for r in block_requirements(g, options) {
        if let Some((_, atts)) = options.iter().find(|(id, _)| *id == r) {
            if let Some(a) = atts.iter().find(|a| g.min_blockers(**a) <= 1) {
                out.push((r, *a));
            }
        }
    }
    if block_declaration_legal(g, options, &out) {
        out
    } else {
        vec![]
    }
}

fn perform_block_declaration(g: &mut Game, blocks: Vec<(ObjectId, ObjectId)>) {
    crate::keyword_impls::pay_block_costs(g, &blocks);
    let Some(c) = g.combat.as_mut() else { return };
    c.blockers_declared = true;
    for (b, a) in &blocks {
        match c.blockers.iter_mut().find(|x| x.id == *b) {
            Some(bi) => bi.blocking.push(*a),
            None => c.blockers.push(BlockerInfo {
                id: *b,
                blocking: vec![*a],
                declared: true,
            }),
        }
        if let Some(ai) = c.attackers.iter_mut().find(|x| x.id == *a) {
            ai.blocked = true;
            ai.blockers.push(*b);
        }
    }
    let attackers = c.attackers.clone();
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
/// already marked, or 1 if the source has deathtouch.
pub fn lethal_damage(g: &Game, source: ObjectId, creature: ObjectId) -> u32 {
    if g.obj(source).has_keyword(KeywordKind::Deathtouch) {
        return 1;
    }
    let o = g.obj(creature);
    (o.toughness() - o.damage as i32).max(0) as u32
}

/// Combat damage step (CR 510).
pub fn combat_damage_step(g: &mut Game, first_strike_step: bool) {
    g.recompute();
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

/// End of combat: remove everything from combat (CR 511.3).
pub fn end_combat(g: &mut Game) {
    g.combat = None;
    g.dirty = true;
}

/// Removes a permanent from combat (CR 506.4).
pub fn remove_from_combat(g: &mut Game, id: ObjectId) {
    let Some(c) = g.combat.as_mut() else { return };
    c.attackers.retain(|a| a.id != id);
    for a in c.attackers.iter_mut() {
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
}

/// Puts a creature onto the battlefield attacking (CR 508.4).
pub fn put_onto_battlefield_attacking(g: &mut Game, id: ObjectId, target: Entity) {
    let ap = g.turn.active;
    if g.obj(id).controller != ap || !g.obj(id).is_creature() || g.obj(id).is(CardType::Battle) {
        return; // CR 506.3a–b, 506.3f
    }
    if !g.valid_attack_target(target) {
        return; // CR 506.3c, 508.4a
    }
    let after_blockers = g.combat.as_ref().is_some_and(|c| c.blockers_declared);
    let Some(c) = g.combat.as_mut() else { return };
    c.attackers.push(AttackerInfo {
        id,
        target: Some(target),
        original_target: Some(target),
        declared: false,
        blocked: false,
        blockers: vec![],
        band: None,
    });
    let _ = after_blockers; // CR 508.4d: enters unblocked (blocked = false).
}

impl Game {
    pub fn valid_attack_target(&self, t: Entity) -> bool {
        match t {
            Entity::Player(p) => self.player(p).in_game(),
            Entity::Object(o) => {
                let ob = self.obj(o);
                self.is_live(o)
                    && ob.zone == Zone::Battlefield
                    && (ob.is(CardType::Planeswalker) || ob.is(CardType::Battle))
            }
        }
    }
}
