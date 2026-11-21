//! Mana spending restrictions (CR 106.6; cards3-costs-mana-replacements): mana spent only
//! to turn permanents face up (the special action, CR 116.2b) — any way, or only for a
//! morph cost or a manifested creature's mana cost — or to cast face-down spells; mana
//! that "can't be spent to cast spells"; mana spent only on "spells you don't own"; and
//! "Activate only if this creature has another activated ability".

use mtg_engine::ability::Duration;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

/// The `i`th of the five colors in a "choose mana type" answer.
fn color(t: &mut TestGame, p: PlayerId, ty: ManaType) {
    let i = [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G]
        .iter()
        .position(|x| *x == ty)
        .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

fn turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let obj = t.g.current(obj);
    t.g.perform_action(p, Action::Special(SpecialAction::TurnFaceUp { obj }))
        .is_ok()
}

/// Manifests `top` with Soul Summons; returns the face-down permanent.
fn manifest(t: &mut TestGame, top: &str) -> ObjectId {
    let card = t.library_top(P0, top);
    let lands = t.lands(P0, "Plains", 2);
    let spell = t.hand(P0, "Soul Summons");
    t.cast(P0, spell).go();
    t.resolve_all();
    for l in lands {
        t.g.tap(l);
    }
    let id = t.g.current(card);
    assert!(t.g.obj(id).face_down && t.on_battlefield(id));
    id
}

/// Casts `card` face down with `kw` (for {3}) and resolves it.
fn cast_face_down(t: &mut TestGame, card: ObjectId, kw: KeywordKind) -> ObjectId {
    t.cast(P0, card).method(CastMethod::FaceDown(kw)).go();
    t.resolve();
    let id = t.g.current(card);
    assert!(t.g.obj(id).face_down && t.on_battlefield(id));
    id
}

#[test]
fn overgrown_zealot_mana_turns_a_manifested_creature_face_up_but_casts_nothing() {
    cr!("106.6", "116.2b", "701.40b");
    ruling!(
        "Overgrown Zealot",
        "to pay costs associated with turning a manifested or cloaked permanent face up"
    );
    assert_supported("Overgrown Zealot");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let fd = manifest(&mut t, "Grizzly Bears");
    let zealot = t.battlefield(P0, "Overgrown Zealot");
    color(&mut t, P0, ManaType::G);
    t.activate(P0, zealot, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), 2);
    // It can't cast a spell.
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(t.cast(P0, elves).try_go().is_err());
    assert_eq!(pool(&t, P0), 2);
    // It pays the manifested Grizzly Bears's mana cost {1}{G}.
    assert!(turn_up(&mut t, P0, fd));
    assert_eq!(pool(&t, P0), 0);
    assert!(!t.g.obj(t.g.current(fd)).face_down);
}

#[test]
fn overgrown_zealot_mana_pays_a_morph_cost() {
    cr!("106.6", "702.37e");
    ruling!(
        "Overgrown Zealot",
        "can be used to pay morph or disguise costs when turning a face-down permanent face up"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let lands = t.lands(P0, "Island", 3);
    let mystic = t.hand(P0, "Rattleclaw Mystic");
    let fd = cast_face_down(&mut t, mystic, KeywordKind::Morph);
    assert!(lands.iter().all(|l| t.g.obj(*l).tapped));
    let zealot = t.battlefield(P0, "Overgrown Zealot");
    color(&mut t, P0, ManaType::G);
    t.activate(P0, zealot, 1, &[]).unwrap();
    // Morph {2}.
    assert!(turn_up(&mut t, P0, fd));
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.g.obj(t.g.current(fd)).chars.name, "Rattleclaw Mystic");
}

#[test]
fn qarsi_deceiver_pays_face_down_spells_and_morph_costs_but_not_disguise_costs() {
    cr!("106.6", "702.37e", "702.168d");
    assert_supported("Qarsi Deceiver");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // A disguised land, cast face down with Islands.
    let islands = t.lands(P0, "Island", 3);
    let branch = t.hand(P0, "Branch of Vitu-Ghazi");
    let branch = cast_face_down(&mut t, branch, KeywordKind::Disguise);
    assert!(islands.iter().all(|l| t.g.obj(*l).tapped));
    let qarsis = t.lands(P0, "Qarsi Deceiver", 3);
    for q in &qarsis {
        t.activate(P0, *q, 0, &[]).unwrap();
    }
    assert_eq!(pool(&t, P0), 3);
    // Not a disguise cost ({3}), and not a face-up spell.
    assert!(!turn_up(&mut t, P0, branch));
    let stone = t.hand(P0, "Mind Stone");
    assert!(t.cast(P0, stone).try_go().is_err());
    assert_eq!(pool(&t, P0), 3);
    // A face-down creature spell ({3}).
    let mystic = t.hand(P0, "Rattleclaw Mystic");
    let fd = cast_face_down(&mut t, mystic, KeywordKind::Morph);
    assert_eq!(pool(&t, P0), 0);
    // Its morph cost {2}.
    for q in &qarsis[..2] {
        t.g.untap(*q);
        t.activate(P0, *q, 0, &[]).unwrap();
    }
    assert!(turn_up(&mut t, P0, fd));
    assert_eq!(pool(&t, P0), 0);
    assert!(!t.g.obj(t.g.current(fd)).face_down);
}

#[test]
fn qarsi_deceiver_pays_a_manifested_creatures_mana_cost() {
    cr!("106.6", "701.40b");
    ruling!(
        "Qarsi Deceiver",
        "can’t be used to cast a spell that instructs you to manifest cards"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let qarsis = t.lands(P0, "Qarsi Deceiver", 2);
    for q in &qarsis {
        t.activate(P0, *q, 0, &[]).unwrap();
    }
    // Soul Summons ({1}{W}) can't be cast with a Plains and that mana.
    let plains = t.battlefield(P0, "Plains");
    let summons = t.hand(P0, "Soul Summons");
    assert!(t.cast(P0, summons).try_go().is_err());
    assert_eq!(pool(&t, P0), 2);
    t.g.tap(plains);
    // It pays a manifested Bronze Sable's mana cost {2}.
    let fd = manifest(&mut t, "Bronze Sable");
    assert_eq!(pool(&t, P0), 2);
    assert!(turn_up(&mut t, P0, fd));
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.g.obj(t.g.current(fd)).chars.name, "Bronze Sable");
}

#[test]
fn tin_street_gossip_mana_turns_creatures_face_up() {
    cr!("106.6", "702.37e");
    assert_supported("Tin Street Gossip");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 3);
    let mystic = t.hand(P0, "Rattleclaw Mystic");
    let fd = cast_face_down(&mut t, mystic, KeywordKind::Morph);
    let gossip = t.battlefield(P0, "Tin Street Gossip");
    t.activate(P0, gossip, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), 2);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    assert!(turn_up(&mut t, P0, fd));
    assert_eq!(pool(&t, P0), 0);
}

#[test]
fn creeping_peeper_mana_casts_an_enchantment_but_not_an_instant() {
    cr!("106.6");
    assert_supported("Creeping Peeper");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let peeper = t.battlefield(P0, "Creeping Peeper");
    t.activate(P0, peeper, 0, &[]).unwrap();
    let opt = t.hand(P0, "Opt");
    assert!(t.cast(P0, opt).try_go().is_err());
    assert_eq!(pool(&t, P0), 1);
    let curiosity = t.hand(P0, "Curiosity");
    t.cast(P0, curiosity).target(peeper).go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(curiosity)), Zone::Battlefield);
}

#[test]
fn thran_turbine_mana_pays_for_abilities_not_spells() {
    cr!("106.6", "106.4");
    ruling!(
        "Thran Turbine",
        "Any of this mana that isn’t spent during your upkeep step will be removed before your draw step begins."
    );
    assert_supported("Thran Turbine");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thran Turbine");
    let stone = t.battlefield(P0, "Mind Stone");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
    let stone2 = t.hand(P0, "Mind Stone");
    assert!(t.cast(P0, stone2).try_go().is_err());
    assert_eq!(pool(&t, P0), 2);
    // Mind Stone's "{1}, {T}, Sacrifice this artifact: Draw a card." can.
    let hand = t.hand_size(P0);
    t.activate(P0, stone, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(pool(&t, P0), 1);
    // What's left is lost before the draw step.
    t.advance_to(P0, Step::Draw);
    assert_eq!(pool(&t, P0), 0);
}

#[test]
fn thieving_varmint_mana_casts_only_spells_you_dont_own() {
    cr!("106.6", "108.3");
    ruling!(
        "Thieving Varmint",
        "on the same spell you don’t own or on two different spells you don’t own"
    );
    assert_supported("Thieving Varmint");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let varmint = t.battlefield(P0, "Thieving Varmint");
    color(&mut t, P0, ManaType::G);
    t.activate(P0, varmint, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 19);
    assert_eq!(pool(&t, P0), 2);
    // Your own card: no.
    let own = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, own).try_go().is_err());
    // A card P1 owns that P0 may cast: yes.
    let theirs = t.exile(P1, "Grizzly Bears");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![theirs],
        Duration::Permanent,
        false,
        None,
    );
    t.cast(P0, theirs).go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    let bears = t.g.current(theirs);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.g.obj(bears).controller, P0);
}

#[test]
fn tazri_grants_mana_only_to_creatures_with_another_activated_ability() {
    cr!("106.6", "602.1b");
    ruling!(
        "Tazri, Stalwart Survivor",
        "may be spent only to activate an ability of a creature on the battlefield"
    );
    assert_supported("Tazri, Stalwart Survivor");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Tazri, Stalwart Survivor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let walla = t.battlefield(P0, "Rootwalla");
    // Grizzly Bears has no other activated ability.
    assert!(t.activate(P0, bears, 0, &[]).is_err());
    assert_eq!(pool(&t, P0), 0);
    // Rootwalla (a 2/2 with "{1}{G}: +2/+2, once each turn") has.
    t.activate(P0, walla, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), 1);
    // The mana can't cast a spell.
    let growth = t.hand(P0, "Giant Growth");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    assert_eq!(pool(&t, P0), 1);
    // It pays for Rootwalla's ability with a Forest.
    t.lands(P0, "Forest", 1);
    t.activate(P0, walla, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), 0);
    t.resolve();
    assert_eq!(t.pt(walla), (4, 4));
}
