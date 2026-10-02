//! Rulings for choices of several objects that must have a relationship (gap-targeting),
//! targeted or not: searches and returns "with total mana value N or less", "with
//! different mana values", "with different names"; exchanges of control between targets
//! that share a card type; targets "from a single graveyard"; targets chosen for each
//! opponent; and modes that must each target a different player.

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s21_common::castable;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::eval::Ctx;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Executes `effect` as if a spell or ability `p` controls resolved.
fn run(t: &mut TestGame, p: PlayerId, effect: Effect) {
    let mut ctx = Ctx::new(None, p);
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

fn gain_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let id = t.g.current(id);
    run(
        t,
        p,
        Effect::GainControl {
            what: Sel::All(Filter::Objects(vec![id])),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
    );
}

fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) {
    let id = t.g.current(id);
    t.g.move_object(id, zone, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.settle();
}

fn names_in_hand(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .player(p)
        .hand
        .iter()
        .map(|o| t.obj(*o).name().to_string())
        .collect();
    v.sort();
    v
}

fn names_on_battlefield(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .permanents()
        .filter(|o| o.controller == p)
        .map(|o| o.name().to_string())
        .collect();
    v.sort();
    v
}

fn sorted(v: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = v.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

// ---------------------------------------------------------------------------
// Untargeted choices with a relationship
// ---------------------------------------------------------------------------

#[test]
fn protean_hulk_finds_creatures_with_total_mana_value_6_or_less() {
    cr!("701.23a", "202.3e");
    ruling!(
        "Protean Hulk",
        "You can find any number of creature cards, so long as their total mana value is 6 or less. For example, you could find eight creature cards with mana value 0 and three with mana value 2, but you couldn't find two with mana value 4."
    );
    ruling!(
        "Protean Hulk",
        "If a card in your library has {X} in its mana cost, X is considered to be 0."
    );
    supported("Protean Hulk");
    let hulk_dies = |t: &mut TestGame| {
        let hulk = t.battlefield(P0, "Protean Hulk");
        t.g.destroy(hulk, None);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
    };
    // Two Hill Giants (4 each): only one of them can be found.
    let mut t = TestGame::new(2);
    let g1 = t.library_top(P0, "Hill Giant");
    let g2 = t.library_top(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(g1), Entity::Object(g2)]);
    hulk_dies(&mut t);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);

    // Two Ornithopters (0), two Grizzly Bears (2) and a Hangarback Walker ({X}{X}, so 0):
    // 0 + 0 + 2 + 2 + 0 = 4, all found.
    let mut t = TestGame::new(2);
    let mut found = Vec::new();
    for n in [
        "Ornithopter",
        "Ornithopter",
        "Grizzly Bears",
        "Grizzly Bears",
        "Hangarback Walker",
    ] {
        found.push(Entity::Object(t.library_top(P0, n)));
    }
    t.answer_choose(P0, &found);
    hulk_dies(&mut t);
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 2);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    // Hangarback Walker entered (a 0/0, it then died).
    assert!(t.in_graveyard(P0, "Hangarback Walker"));
}

#[test]
fn lively_dirge_returns_creatures_with_total_mana_value_4_or_less() {
    cr!("702.172a", "608.2c");
    ruling!(
        "Lively Dirge",
        "The second mode doesn’t target any cards. You choose which ones you’re returning as Lively Dirge is resolving. You may choose the one you put into the graveyard with the first mode, if applicable, or ones that were already there."
    );
    supported("Lively Dirge");
    // Both modes: a Grizzly Bears is searched for and put into the graveyard; then it and
    // the Bears already there (2 + 2 = 4) are returned.
    let mut t = TestGame::new(2);
    let old = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let new = t.library_top(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Lively Dirge");
    add_mana(&mut t, P0, ManaType::B, 5);
    t.cast(P0, spell).modes(&[0, 1]).go();
    // Nothing is chosen for the second mode as the spell is cast.
    assert!(t.asked().iter().all(|(_, d)| !matches!(
        d,
        mtg_engine::decision::Decision::ChooseTargets { .. }
    )));
    // As it resolves: the searched card (the next object created, CR 400.7) is in the
    // graveyard by the time the cards to return are chosen.
    let searched = ObjectId(t.g.objects.len() as u32);
    t.answer_choose(P0, &[Entity::Object(new)]);
    t.answer_choose(P0, &[Entity::Object(searched), Entity::Object(old)]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert!(t.in_graveyard(P0, "Hill Giant"));

    // A Bears and the Hill Giant (2 + 4 = 6): only the Bears is returned.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    let spell = t.hand(P0, "Lively Dirge");
    add_mana(&mut t, P0, ManaType::B, 4);
    t.cast(P0, spell).modes(&[1]).go();
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn seasons_past_returns_cards_with_different_mana_values() {
    cr!("202.3a", "608.2c");
    ruling!(
        "Seasons Past",
        "You choose which cards to return as Seasons Past resolves, not as you cast it."
    );
    ruling!(
        "Seasons Past",
        "The phrase \"different mana values\" compares the mana costs of cards in your graveyard to one another, not to Seasons Past. You may return a card with a mana value of 6."
    );
    ruling!(
        "Seasons Past",
        "A card with no mana cost, such as a land, has a mana value of 0."
    );
    supported("Seasons Past");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let runeclaw = t.graveyard(P0, "Runeclaw Bear");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let wurm = t.graveyard(P0, "Craw Wurm");
    let forest = t.graveyard(P0, "Forest");
    let thopter = t.graveyard(P0, "Ornithopter");
    let spell = t.hand(P0, "Seasons Past");
    add_mana(&mut t, P0, ManaType::G, 6);
    t.cast(P0, spell).go();
    // Chosen as it resolves: the Bears (2), the Runeclaw Bear (2 again: not returned),
    // Llanowar Elves (1), Craw Wurm (6, like Seasons Past), Forest (no mana cost: 0) and
    // Ornithopter (0 again: not returned).
    t.answer_choose(
        P0,
        &[bears, runeclaw, elves, wurm, forest, thopter].map(Entity::Object),
    );
    t.resolve();
    assert_eq!(
        names_in_hand(&t, P0),
        sorted(&["Craw Wurm", "Forest", "Grizzly Bears", "Llanowar Elves"])
    );
    assert!(t.in_graveyard(P0, "Runeclaw Bear"));
    assert!(t.in_graveyard(P0, "Ornithopter"));
}

#[test]
fn eerie_ultimatum_returns_permanent_cards_with_different_names() {
    cr!("201.2", "608.2c");
    ruling!(
        "Eerie Ultimatum",
        "You may choose to return just one permanent card, regardless of its name."
    );
    ruling!(
        "Eerie Ultimatum",
        "You choose which permanent cards to return while Eerie Ultimatum is resolving. No player may take actions between the time you choose and the time those cards return to the battlefield."
    );
    supported("Eerie Ultimatum");
    let setup = |t: &mut TestGame| -> Vec<ObjectId> {
        let v = vec![
            t.graveyard(P0, "Grizzly Bears"),
            t.graveyard(P0, "Grizzly Bears"),
            t.graveyard(P0, "Hill Giant"),
            t.graveyard(P0, "Forest"),
        ];
        t.graveyard(P0, "Shock");
        let spell = t.hand(P0, "Eerie Ultimatum");
        for c in [ManaType::W, ManaType::B, ManaType::G] {
            add_mana(t, P0, c, 3);
        }
        t.cast(P0, spell).go();
        v
    };
    // Two Bears, a Hill Giant and a Forest: one Bears, the Giant and the Forest return.
    let mut t = TestGame::new(2);
    let v = setup(&mut t);
    t.answer_choose(P0, &v.iter().copied().map(Entity::Object).collect::<Vec<_>>());
    t.resolve();
    assert_eq!(
        names_on_battlefield(&t, P0),
        sorted(&["Forest", "Grizzly Bears", "Hill Giant"])
    );
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Shock"));
    // Just one.
    let mut t = TestGame::new(2);
    let v = setup(&mut t);
    t.answer_choose(P0, &[Entity::Object(v[0])]);
    t.resolve();
    assert_eq!(names_on_battlefield(&t, P0), sorted(&["Grizzly Bears"]));
}

// ---------------------------------------------------------------------------
// Exchanges between targets that must share a card type
// ---------------------------------------------------------------------------

fn untap_thief(t: &mut TestGame, thief: ObjectId) {
    let thief = t.g.current(thief);
    t.g.objects[thief.0 as usize].tapped = true;
    t.g.untap(thief);
    t.g.flush_events();
    t.settle();
}

#[test]
fn daring_thief_targets_share_a_card_type() {
    cr!("115.1", "601.2c", "701.12a", "603.3d");
    ruling!(
        "Daring Thief",
        "Although the permanent you control can't be a land, the other target can, provided it shares a card type with the first target. For example, you could target an artifact you control and an artifact land controlled by an opponent."
    );
    ruling!(
        "Daring Thief",
        "If the full exchange can't happen, perhaps because one of the targets is illegal as the inspired ability tries to resolve, then nothing happens. No permanents change controllers."
    );
    ruling!(
        "Daring Thief",
        "The exchange of control lasts indefinitely. It doesn't expire when Daring Thief leaves the battlefield."
    );
    supported("Daring Thief");
    // P0's Sol Ring (an artifact) and P1's Seat of the Synod (an artifact land) share a
    // card type; P1's Forest and Grizzly Bears don't share one with the Sol Ring.
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Daring Thief");
    let ring = t.battlefield(P0, "Sol Ring");
    let seat = t.battlefield(P1, "Seat of the Synod");
    t.battlefield(P1, "Forest");
    t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.answer_targets(P0, &[Entity::Object(seat)]);
    t.answer_yes(P0, true);
    untap_thief(&mut t, thief);
    let choices: Vec<Vec<Entity>> = asked_since(&t, from)
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    // The Thief itself is a creature, which P1's Bears shares; the Sol Ring's partner can
    // only be the Seat.
    assert_eq!(choices[1], vec![Entity::Object(seat)]);
    t.resolve_all();
    assert_eq!(t.obj_now(ring).controller, P1);
    assert_eq!(t.obj_now(seat).controller, P0);
    // The exchange lasts after the Thief leaves.
    t.g.destroy(thief, None);
    t.settle();
    assert_eq!(t.obj_now(ring).controller, P1);
    assert_eq!(t.obj_now(seat).controller, P0);

    // One target leaves before the ability resolves: nothing changes hands.
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Daring Thief");
    let ring = t.battlefield(P0, "Sol Ring");
    let seat = t.battlefield(P1, "Seat of the Synod");
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.answer_targets(P0, &[Entity::Object(seat)]);
    t.answer_yes(P0, true);
    untap_thief(&mut t, thief);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, seat, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.obj_now(ring).controller, P0);

    // With nothing of P1's sharing a type with the Sol Ring or the Thief, the ability has
    // no legal targets and is removed from the stack.
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Daring Thief");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P1, "Forest");
    untap_thief(&mut t, thief);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn legerdemain_targets_share_artifact_or_creature() {
    cr!("115.1", "601.2c", "608.2b", "701.12a");
    ruling!(
        "Legerdemain",
        "You can’t cast Legerdemain unless both targets share a type at that time. It will also check to make sure they share a type as it resolves. If they no longer do so, the exchange will not occur."
    );
    supported("Legerdemain");
    // P0's Grizzly Bears and P1's Forest share no type: Legerdemain can't be cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Legerdemain");
    add_mana(&mut t, P0, ManaType::U, 4);
    assert!(!castable(&mut t, P0, spell));
    // P1's Dryad Arbor is a creature land: it shares "creature" with the Bears.
    let arbor = t.battlefield(P1, "Dryad Arbor");
    assert!(castable(&mut t, P0, spell));
    t.cast(P0, spell).target(bears).target(arbor).go();
    // The Arbor stops being a creature: they no longer share artifact or creature, so
    // the exchange doesn't happen.
    run(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![arbor])),
            mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
    );
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(arbor).controller, P1);

    // An artifact land and a land share a card type, but not one of "those types"
    // (artifact or creature): P1's Forest isn't a legal second target for P0's Seat of
    // the Synod.
    let mut t = TestGame::new(2);
    let seat = t.battlefield(P0, "Seat of the Synod");
    t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Legerdemain");
    add_mana(&mut t, P0, ManaType::U, 4);
    assert!(!castable(&mut t, P0, spell));
    let ring = t.battlefield(P1, "Sol Ring");
    assert!(castable(&mut t, P0, spell));
    t.cast(P0, spell).target(seat).target(ring).go();
    t.resolve();
    assert_eq!(t.obj_now(seat).controller, P1);
    assert_eq!(t.obj_now(ring).controller, P0);
}

#[test]
fn shifting_loyalties_two_permanents_that_share_a_card_type() {
    cr!("115.1", "608.2b", "701.12a");
    ruling!(
        "Shifting Loyalties",
        "Either target can have card types the other does not, as long as they share at least one card type. For example, you could target a creature and an artifact creature."
    );
    ruling!(
        "Shifting Loyalties",
        "You don’t have to control either target permanent."
    );
    ruling!(
        "Shifting Loyalties",
        "If one of the target permanents is an illegal target when Shifting Loyalties resolves, the exchange won’t happen. If both permanents are illegal targets (perhaps because they no longer share a card type), Shifting Loyalties won’t resolve."
    );
    ruling!(
        "Shifting Loyalties",
        "If the same player controls both permanents when Shifting Loyalties resolves, nothing happens."
    );
    supported("Shifting Loyalties");
    // A creature (P1's) and an artifact creature (P2's), neither P0's: exchanged.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P2, "Ornithopter");
    let spell = t.hand(P0, "Shifting Loyalties");
    add_mana(&mut t, P0, ManaType::U, 6);
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(thopter)])
        .go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P2);
    assert_eq!(t.obj_now(thopter).controller, P1);

    // One leaves: no exchange (the spell still resolves).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Shifting Loyalties");
    add_mana(&mut t, P0, ManaType::U, 6);
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    move_to(&mut t, giant, Zone::Hand(P1));
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);

    // They no longer share a card type: both are illegal and the spell doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let arbor = t.battlefield(P1, "Dryad Arbor");
    let spell = t.hand(P0, "Shifting Loyalties");
    add_mana(&mut t, P0, ManaType::U, 6);
    let id = t
        .cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(arbor)])
        .go();
    run(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![arbor])),
            mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
    );
    t.resolve();
    assert!(!t.g.is_live(id));
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(arbor).controller, P1);

    // The same player controls both: nothing happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Shifting Loyalties");
    add_mana(&mut t, P0, ManaType::U, 6);
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    gain_control(&mut t, P0, giant);
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(giant).controller, P0);
}

#[test]
fn trickster_gods_heist_chapter_two_targets_share_a_card_type() {
    cr!("714.2b", "701.12a", "608.2b");
    ruling!(
        "The Trickster-God's Heist",
        "For the chapter II ability, either target can have card types the other does not. For example, you can target an artifact land and an artifact."
    );
    ruling!(
        "The Trickster-God's Heist",
        "For those two abilities, you don't have to control either target."
    );
    ruling!(
        "The Trickster-God's Heist",
        "If one of the target permanents of the chapter I or chapter II ability is an illegal target as that ability resolves, the exchange won't happen. If both targets are illegal, the ability doesn't resolve."
    );
    use crate::r_s19_common::add_lore;
    // Chapter II: P1's Seat of the Synod (an artifact land) and P2's Sol Ring (an
    // artifact) share a card type; neither is P0's.
    let mut t = TestGame::new(3);
    let seat = t.battlefield(P1, "Seat of the Synod");
    let ring = t.battlefield(P2, "Sol Ring");
    let saga = t.battlefield(P0, "The Trickster-God's Heist");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    t.answer_targets(P0, &[Entity::Object(seat), Entity::Object(ring)]);
    t.answer_yes(P0, true);
    add_lore(&mut t, saga, 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(seat).controller, P2);
    assert_eq!(t.obj_now(ring).controller, P1);

    // One target leaves: the exchange doesn't happen.
    let mut t = TestGame::new(3);
    let seat = t.battlefield(P1, "Seat of the Synod");
    let ring = t.battlefield(P2, "Sol Ring");
    let saga = t.battlefield(P0, "The Trickster-God's Heist");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    t.answer_targets(P0, &[Entity::Object(seat), Entity::Object(ring)]);
    t.answer_yes(P0, true);
    add_lore(&mut t, saga, 1);
    move_to(&mut t, ring, Zone::Hand(P2));
    t.resolve_all();
    assert_eq!(t.obj_now(seat).controller, P1);
}
