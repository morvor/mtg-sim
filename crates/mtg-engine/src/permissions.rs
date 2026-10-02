//! Permissions to play cards from zones other than the hand (CR 601.3, 305.1), and which
//! one a player uses.
//!
//! A card outside its owner's hand can be cast only if a rule or effect allows it
//! (CR 601.3): an effect's permission for that card ("Until end of turn, you may cast that
//! card", a [`crate::casting::PlayGrant`]), a static ability's or a resolved effect's
//! permission to play cards with certain qualities from a zone ("You may play lands from
//! your graveyard", a `PlayPermission`), or a rule or ability that is its own permission
//! (flashback, foretell, a commander in the command zone, ...). The permission is checked
//! against the characteristics the card would have as it's played (CR 601.3e).
//!
//! When several permissions allow it, the player announces which one they're using as they
//! begin to play the card (CR 601.2, 305.1; Muldrotha, the Gravetide ruling). That is part
//! of the way the card is played ([`crate::casting::CastOption::permission`]): the player
//! gets that permission's terms ([`PlayTerms`]) — "you may cast that card" doesn't allow
//! playing it as a land; a required alternative cost ("If you cast a spell this way, pay
//! life equal to its mana value rather than pay its mana cost", CR 118.9b) replaces the mana
//! cost and can't be combined with another alternative cost (CR 118.9a); "as though it had
//! flash" (CR 702.8a); "costs {2} more" (CR 601.2f); "enters tapped" (CR 614.1d) — and uses
//! up exactly that permission if it can be used only once each turn
//! (`kw/once_each_turn_cast.rs`).
//!
//! Which ways of casting a card need such a permission is found by computing them twice:
//! with only the card's own permissions ([`own_permissions_only`]: rules and keywords such
//! as flashback, which let it be cast however other permissions stand) and with all of
//! them. A way of casting it that exists only with the others is offered once for each
//! permission that allows it, with that permission's terms ([`attach`]).

use crate::ability::*;
use crate::casting::{matches_with_chars, CastOption};
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;
use std::cell::Cell;

/// Which permission it is, as the game state is now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionKind {
    /// An effect's permission for that card: `Game::play_grants[i]`.
    Grant(usize),
    /// A permission to play cards with certain qualities from a zone:
    /// `Game::statics.play_permissions[i]`.
    Static(usize),
}

/// A permission a player uses to play a card from where it is (CR 601.3, 305.1).
#[derive(Clone, Debug)]
pub struct CastPermission {
    pub kind: PermissionKind,
    /// The object whose ability or effect gives it.
    pub source: Option<ObjectId>,
    /// "Without paying its mana cost": spells are cast with it only that way
    /// (`CastMethod::Free`, CR 118.9b).
    pub free: bool,
    /// Its terms.
    pub terms: PlayTerms,
    /// The once-each-turn use it is (its object and slot), if it's one.
    pub once: Option<(ObjectId, SmolStr)>,
    /// For a permission to play cards with certain qualities: those qualities (as a spell),
    /// with the object and player they're judged relative to. The spell must still have
    /// them once its proposal is complete (with the value chosen for X, CR 601.2e).
    pub qualities: Option<(Filter, Option<ObjectId>, PlayerId)>,
}

impl PartialEq for CastPermission {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl CastPermission {
    /// Whether spells can be cast with it only for an alternative cost it requires.
    pub fn requires_cost(&self) -> bool {
        self.free || self.terms.alt_cost.is_some()
    }
}

thread_local! {
    static OWN_ONLY: Cell<bool> = const { Cell::new(false) };
}

/// Whether only the card's own permissions (rules, its own abilities) count right now.
pub fn own_only() -> bool {
    OWN_ONLY.with(|c| c.get())
}

/// Runs `f` with only cards' own permissions counting: no effect's permission to play
/// cards from other zones (see the module documentation).
pub fn own_permissions_only<R>(f: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            OWN_ONLY.with(|c| c.set(self.0));
        }
    }
    let _restore = Restore(OWN_ONLY.with(|c| c.replace(true)));
    f()
}

/// The terms of a static permission: one to cast spells from somewhere other than the
/// hand for an alternative cost requires that cost (CR 118.9b) ("You may cast spells from
/// your hand without paying their mana costs" offers an alternative cost instead, see
/// `casting.rs`); spells cast with one may have flash.
fn static_terms(perm: &PlayPermission) -> (bool, PlayTerms) {
    let mut terms = PlayTerms {
        flash: perm.flash,
        ..Default::default()
    };
    let mut free = false;
    if perm.zone != ZoneKind::Hand {
        match &perm.cost {
            Some(c) if c.is_free() => free = true,
            Some(c) => terms.alt_cost = Some(c.clone()),
            None => {}
        }
    }
    (free, terms)
}

/// Whether the zone of a static permission holds `card` for `p`.
fn in_permission_zone(g: &Game, p: PlayerId, card: ObjectId, perm: &PlayPermission) -> bool {
    let o = g.obj(card);
    match perm.zone {
        ZoneKind::Library => {
            if perm.top_only {
                g.library_top(p) == Some(card)
            } else {
                o.zone == Zone::Library(p)
            }
        }
        ZoneKind::Graveyard => o.zone == Zone::Graveyard(p),
        ZoneKind::Exile => o.zone == Zone::Exile,
        ZoneKind::Hand => o.zone == Zone::Hand(p),
        ZoneKind::Command => o.zone == Zone::Command,
        _ => false,
    }
}

/// The effects' permissions (not rules') that let `p` play `card` from where it is: as a
/// land (`land`) or as a spell with the characteristics `chars` it would have (CR 601.3e).
/// Ordered as a player would usually prefer them: permissions that can be used any number
/// of times before once-each-turn ones, then fewer terms first.
pub fn allowing(
    g: &Game,
    p: PlayerId,
    card: ObjectId,
    chars: &Characteristics,
    land: bool,
) -> Vec<CastPermission> {
    if own_only() {
        return vec![];
    }
    let o = g.obj(card);
    let mut out = Vec::new();
    // Grants from resolved effects name specific cards ("you may play that card").
    for (i, gr) in g.play_grants.iter().enumerate() {
        if gr.player != p || gr.object != card || (land && gr.terms.spells_only) {
            continue;
        }
        out.push(CastPermission {
            kind: PermissionKind::Grant(i),
            source: gr.source,
            free: gr.free,
            terms: gr.terms.clone(),
            once: None,
            qualities: None,
        });
    }
    // CR 601.3f, 406.3b: a face-down card in exile can be cast because of a permission to
    // cast spells "with certain qualities" only by a player who may look at it (and then
    // only if the resulting spell has those qualities).
    let hidden = o.zone == Zone::Exile && o.face_down && !crate::zones::may_look(g, p, card);
    for (i, (src, ctl, perm, once)) in g.statics.play_permissions.iter().enumerate() {
        if hidden || (land && !perm.lands) || (!land && !perm.spells) {
            continue;
        }
        let ctx = Ctx::new(Some(*src), *ctl);
        if !g.player_rel_matches(perm.who, p, &ctx) || !in_permission_zone(g, p, card, perm) {
            continue;
        }
        let f = if land {
            perm.what.clone()
        } else {
            crate::casting::as_spell_filter(&perm.what)
        };
        if !matches_with_chars(g, card, chars, &f, &ctx) {
            continue;
        }
        let (free, terms) = static_terms(perm);
        out.push(CastPermission {
            kind: PermissionKind::Static(i),
            source: Some(*src),
            free,
            terms,
            once: once.clone().map(|slot| (*src, slot)),
            qualities: Some((f, Some(*src), *ctl)),
        });
    }
    out.sort_by_key(|c| {
        (
            c.once.is_some(),
            c.terms.cost_increase,
            c.terms.lands_enter_tapped,
        )
    });
    out
}

/// Whether an effect's permission (not a rule's) may let `p` play `card` from where it is,
/// whatever it's played as.
pub fn may_be_permitted(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    if own_only() {
        return false;
    }
    g.play_grants
        .iter()
        .any(|gr| gr.player == p && gr.object == card)
        || g.statics
            .play_permissions
            .iter()
            .any(|(src, ctl, perm, _)| {
                g.player_rel_matches(perm.who, p, &Ctx::new(Some(*src), *ctl))
                    && in_permission_zone(g, p, card, perm)
            })
}

/// The cards outside `p`'s hand that effects' permissions may let them play.
pub fn effect_permitted_cards(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    let mut out = Vec::new();
    if own_only() {
        return out;
    }
    for (src, ctl, perm, _) in &g.statics.play_permissions {
        let ctx = Ctx::new(Some(*src), *ctl);
        if !g.player_rel_matches(perm.who, p, &ctx) {
            continue;
        }
        let cards: Vec<ObjectId> = match perm.zone {
            ZoneKind::Library => {
                if perm.top_only {
                    g.library_top(p).into_iter().collect()
                } else {
                    g.player(p).library.clone()
                }
            }
            ZoneKind::Graveyard => g.player(p).graveyard.clone(),
            ZoneKind::Exile => g.exile.clone(),
            ZoneKind::Hand => g.player(p).hand.clone(),
            ZoneKind::Command => g.command.clone(),
            _ => vec![],
        };
        for c in cards {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    for gnt in &g.play_grants {
        if gnt.player == p && g.is_live(gnt.object) && !out.contains(&gnt.object) {
            out.push(gnt.object);
        }
    }
    out
}

/// Two ways of casting a card that are the same, whatever permission they'd use.
pub fn same_way(a: &CastOption, b: &CastOption) -> bool {
    a.method == b.method
        && a.face == b.face
        && a.tag == b.tag
        && a.flash == b.flash
        && a.any_time == b.any_time
        && format!("{:?}", a.alt_cost) == format!("{:?}", b.alt_cost)
        && format!("{:?}", a.extra_cost) == format!("{:?}", b.extra_cost)
}

/// The ways `p` may cast `card` from outside their hand: `own`, the ways its own
/// permissions allow, and the others among `all` (the ways any permission allows), once
/// for each permission that allows the spell it would become, with that permission's
/// terms. A way the card's own permission allows is offered with another permission too if
/// that one gives it flash or an alternative cost, unless it's already cast for an
/// alternative cost of its own (flashback's cost comes with flashback's permission).
pub fn attach(
    g: &Game,
    p: PlayerId,
    card: ObjectId,
    own: Vec<CastOption>,
    all: Vec<CastOption>,
) -> Vec<CastOption> {
    let mut out = own.clone();
    for opt in all {
        let is_own = own.iter().any(|o| same_way(o, &opt));
        if is_own && opt.alt_cost.is_some() {
            continue;
        }
        let chars = g.option_characteristics(card, &opt);
        for perm in allowing(g, p, card, &chars, false) {
            if is_own && !(perm.terms.flash || perm.requires_cost()) {
                continue;
            }
            if let Some(o) = with_permission(&chars, &opt, perm) {
                out.push(o);
            }
        }
    }
    out
}

/// `opt` cast with the permission `perm`, as the spell with characteristics `chars`: with
/// the alternative cost it requires (none if `opt` already has one, CR 118.9a) and flash
/// if it gives that.
fn with_permission(
    chars: &Characteristics,
    opt: &CastOption,
    perm: CastPermission,
) -> Option<CastOption> {
    let mut o = opt.clone();
    let face_way = matches!(o.method, CastMethod::Normal | CastMethod::Half(_));
    // X is 0 for costs relative to the spell (CR 107.3b).
    let mv = chars
        .mana_cost
        .as_ref()
        .map_or(0, |m| m.mana_value_with_x(0));
    if perm.free {
        if o.alt_cost.is_some() {
            return None;
        }
        if face_way {
            o.method = CastMethod::Free;
        }
        o.alt_cost = Some(Cost::free());
    } else if let Some(c) = &perm.terms.alt_cost {
        if o.alt_cost.is_some() {
            return None;
        }
        o.alt_cost = Some(spell_relative_cost(c, mv));
        if face_way {
            o.method = CastMethod::Alternative(crate::casting::PERMISSION_COST);
        }
    }
    // An additional cost that comes with the permission (CR 601.2f).
    if let Some(e) = &perm.terms.extra_cost {
        let e = spell_relative_cost(e, mv);
        o.extra_cost = Some(match o.extra_cost.take() {
            Some(mut c) => {
                crate::casting::add_cost(&mut c, &e);
                c
            }
            None => e,
        });
    }
    if perm.terms.flash {
        o.flash = true;
    }
    o.permission = Some(perm);
    Some(o)
}

/// `cost` with amounts relative to the spell ("life equal to its mana value") given for a
/// spell with mana value `mv`.
pub fn spell_relative_cost(cost: &Cost, mv: u32) -> Cost {
    let fix = |v: &Value| -> Value {
        match v {
            Value::ManaValueOf(s) if matches!(**s, Sel::This) => Value::c(mv as i32),
            other => other.clone(),
        }
    };
    Cost {
        mana: cost.mana.clone(),
        parts: cost
            .parts
            .iter()
            .map(|part| match part {
                CostPart::PayLife(v) => CostPart::PayLife(fix(v)),
                CostPart::PayEnergy(v) => CostPart::PayEnergy(fix(v)),
                other => other.clone(),
            })
            .collect(),
    }
}

/// The permission `p` uses to play `card` as a land, chosen among those that allow it
/// (CR 305.1): none if it's in their hand or a rule or its own ability lets them play it.
pub fn choose_land_permission(g: &mut Game, p: PlayerId, card: ObjectId) -> Option<CastPermission> {
    if g.obj(card).zone == Zone::Hand(p) || crate::kw::playable_lands(g, p).contains(&card) {
        return None;
    }
    let chars = g.obj(card).chars.clone();
    let chars = if chars.is_land() {
        chars
    } else {
        g.face_characteristics(card, FaceState::Back)
    };
    let mut perms = allowing(g, p, card, &chars, true);
    if perms.is_empty() {
        return None;
    }
    let refs: Vec<&CastPermission> = perms.iter().collect();
    let i = choose(g, p, card, &refs, 0, "play this land with");
    Some(perms.swap_remove(i))
}

/// The way among `ways` (the same way of casting `card`, with different permissions) that
/// `p` uses: the player announces which permission they're using (CR 601.2, 601.3). By
/// default, the first one whose costs could be paid.
pub fn choose_cast_permission(
    g: &mut Game,
    p: PlayerId,
    card: ObjectId,
    mut ways: Vec<CastOption>,
) -> CastOption {
    if ways.len() == 1 {
        return ways.swap_remove(0);
    }
    let default = ways
        .iter()
        .position(|o| g.can_begin_cast(p, card, o))
        .unwrap_or(0);
    let i = {
        let perms: Vec<&CastPermission> =
            ways.iter().filter_map(|o| o.permission.as_ref()).collect();
        if perms.len() == ways.len() {
            choose(g, p, card, &perms, default, "cast this card with")
        } else {
            // A way with the card's own permission among them.
            let labels = ways
                .iter()
                .map(|o| match &o.permission {
                    Some(c) => label(g, c),
                    None => "its own permission".to_string(),
                })
                .collect();
            g.ask_option(p, Some(card), "Choose the permission you're using", labels)
        }
    };
    ways.swap_remove(i.min(ways.len() - 1))
}

/// Asks `p` which of the permissions `perms` they're using to `what`: by choosing the
/// object each comes from if those differ, otherwise by name. `default` if they don't say.
fn choose(
    g: &mut Game,
    p: PlayerId,
    card: ObjectId,
    perms: &[&CastPermission],
    default: usize,
    what: &str,
) -> usize {
    if perms.len() <= 1 {
        return 0;
    }
    let prompt = format!("Choose the permission you're using to {what}");
    let sources: Vec<ObjectId> = perms.iter().filter_map(|c| c.source).collect();
    let distinct = sources.len() == perms.len()
        && sources
            .iter()
            .enumerate()
            .all(|(i, s)| !sources[..i].contains(s));
    if distinct {
        // The default first: what an agent that doesn't choose gets.
        let mut order: Vec<usize> = (0..perms.len()).collect();
        order.swap(0, default.min(perms.len() - 1));
        let cands = order.iter().map(|i| Entity::Object(sources[*i])).collect();
        let chosen = g.ask_entities(p, Some(card), &prompt, cands, 1, 1);
        return chosen
            .first()
            .and_then(|e| e.object())
            .and_then(|o| sources.iter().position(|s| *s == o))
            .unwrap_or(default);
    }
    let labels: Vec<String> = perms.iter().map(|c| label(g, c)).collect();
    match g.ask(
        p,
        crate::decision::Decision::ChooseOption {
            source: Some(card),
            prompt,
            options: labels,
        },
    ) {
        crate::decision::Answer::Index(i) if i < perms.len() => i,
        _ => default,
    }
}

/// Whether the permission `perm` the card `card` is being cast with still allows the
/// spell it became as proposed, which has the characteristics `proposed` (with the value
/// chosen for X in its mana cost): its qualities are judged with the choices made in the
/// proposal, such as its mana value with that X (CR 601.2e, 601.3e; Lurrus of the
/// Dream-Den ruling), and otherwise as the card was where it was cast from. A permission
/// for that card allows it however it's proposed.
pub fn still_allows(
    g: &Game,
    perm: &CastPermission,
    card: ObjectId,
    proposed: &Characteristics,
) -> bool {
    match &perm.qualities {
        Some((f, src, ctl)) => matches_with_chars(g, card, proposed, f, &Ctx::new(*src, *ctl)),
        None => true,
    }
}

/// The mana a spell cast with the permission `perm` asks for: "you may spend mana as
/// though it were mana of any color to cast that spell" and "mana of any type can be spent
/// to cast it" apply to spells cast with that permission (CR 609.4b, 118.14).
pub fn spend_terms(perm: Option<&CastPermission>, cost: &mut Cost) {
    let Some(c) = perm else {
        return;
    };
    if c.terms.spend_any_type || c.terms.spend_as_any_color {
        crate::cost_rules::pay_with_any_mana(cost, c.terms.spend_any_type);
    }
}

/// Records that `perm` was used to play a card: a once-each-turn permission is used up.
pub fn record_use(g: &mut Game, perm: Option<&CastPermission>) {
    if let Some((src, slot)) = perm.and_then(|c| c.once.clone()) {
        g.history.once_permissions_used.push((src, slot));
        g.dirty = true;
    }
}

/// How a permission is named to the player choosing among them.
pub fn label(g: &Game, perm: &CastPermission) -> String {
    let from = perm
        .source
        .and_then(|s| g.try_obj(s))
        .map(|o| o.chars.name.to_string());
    let mut s = match (perm.kind, from) {
        (PermissionKind::Grant(_), Some(n)) => format!("the permission {n} gave for this card"),
        (PermissionKind::Grant(_), None) => "an effect's permission for this card".to_string(),
        (PermissionKind::Static(_), Some(n)) => format!("{n}'s permission"),
        (PermissionKind::Static(_), None) => "an effect's permission".to_string(),
    };
    if let Some((_, slot)) = &perm.once {
        if !slot.is_empty() {
            s.push_str(&format!(" (as your {slot})"));
        }
    }
    s
}
