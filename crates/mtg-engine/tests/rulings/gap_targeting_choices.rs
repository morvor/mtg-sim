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

/// Whether `p` could activate the `index`th activated ability of `source` now.
fn can_activate_nth(t: &mut TestGame, p: PlayerId, source: ObjectId, index: usize) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let source = t.g.current(source);
    let uid = t
        .obj(source)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(index)
        .map(|a| a.uid)
        .unwrap();
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, mtg_engine::decision::Action::Activate { source: s, ability } if *s == source && *ability == uid)
    })
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

// ---------------------------------------------------------------------------
// Targets from a single graveyard; up to N targets
// ---------------------------------------------------------------------------

#[test]
fn pestilent_cauldron_needs_four_cards_in_a_single_graveyard() {
    cr!("115.1", "602.2b", "601.2c");
    ruling!(
        "Pestilent Cauldron // Restorative Burst",
        "You must be able to target four cards in a single graveyard in order to activate Pestilent Cauldron’s last ability."
    );
    supported("Pestilent Cauldron // Restorative Burst");
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Pestilent Cauldron // Restorative Burst");
    t.lands(P0, "Swamp", 4);
    for _ in 0..2 {
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P1, "Grizzly Bears");
    }
    // Four cards in graveyards, but no four in one graveyard.
    assert!(!can_activate_nth(&mut t, P0, cauldron, 2));
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Forest");
    assert!(can_activate_nth(&mut t, P0, cauldron, 2));
    // Answering one of P0's cards among them: the four are fitted to P1's graveyard.
    let p1_cards: Vec<Entity> = t
        .g
        .player(P1)
        .graveyard
        .iter()
        .map(|o| Entity::Object(*o))
        .collect();
    let mut answer = p1_cards.clone();
    answer[0] = Entity::Object(t.g.player(P0).graveyard[0]);
    t.answer_targets(P0, &answer);
    let hand = t.hand_size(P0);
    let ab = t.activate(P0, cauldron, 2, &[]).unwrap().unwrap();
    let mut chosen: Vec<Entity> = t
        .obj(ab)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect();
    chosen.sort();
    let mut expect = p1_cards;
    expect.sort();
    assert_eq!(chosen, expect);
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn restorative_burst_with_no_targets_or_only_illegal_ones() {
    cr!("608.2b", "115.1");
    ruling!(
        "Pestilent Cauldron // Restorative Burst",
        "You may cast Restorative Burst with no targets. If you do, each player will gain 4 life and you’ll exile Restorative Burst. However, if you choose one or two target cards, and each of those cards is an illegal target as Restorative Burst tries to resolve (usually because something else moved them in response), Restorative Burst won’t resolve and none of its effects will happen. No one will gain life, and Restorative Burst won’t be exiled."
    );
    // No targets: everyone gains 4 life and the card is exiled.
    let mut t = TestGame::new(2);
    let burst = t.hand(P0, "Pestilent Cauldron // Restorative Burst");
    add_mana(&mut t, P0, ManaType::G, 5);
    t.cast(P0, burst).method(CastMethod::Half(1)).targets(&[]).go();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (24, 24));
    // The card is exiled, not put into the graveyard.
    assert_eq!(t.g.exile.len(), 1);
    assert_eq!(t.graveyard_size(P0), 0);

    // One target, which leaves the graveyard: the spell doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let burst = t.hand(P0, "Pestilent Cauldron // Restorative Burst");
    add_mana(&mut t, P0, ManaType::G, 5);
    t.cast(P0, burst)
        .method(CastMethod::Half(1))
        .targets(&[Entity::Object(bears)])
        .go();
    move_to(&mut t, bears, Zone::Exile);
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(t.g.exile.iter().all(|o| t.obj(*o).name() == "Grizzly Bears"));
}

#[test]
fn unbury_returns_the_other_card_by_last_known_creature_types() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Unbury",
        "If you choose the second mode and one of the two cards leaves your graveyard, you'll still return the other card to your hand as long as it has a creature type that the other card had as it left your graveyard."
    );
    supported("Unbury");
    // Llanowar Elves (Elf Druid) and Elvish Visionary (Elf Shaman) share Elf; the Elves
    // leave the graveyard; the Visionary still returns.
    let mut t = TestGame::new(2);
    let elves = t.graveyard(P0, "Llanowar Elves");
    let visionary = t.graveyard(P0, "Elvish Visionary");
    let spell = t.hand(P0, "Unbury");
    add_mana(&mut t, P0, ManaType::B, 2);
    t.cast(P0, spell)
        .modes(&[1])
        .targets(&[Entity::Object(elves), Entity::Object(visionary)])
        .go();
    move_to(&mut t, elves, Zone::Exile);
    t.resolve();
    assert!(t.in_hand(P0, "Elvish Visionary"));
    assert!(t.in_exile("Llanowar Elves"));
}

// ---------------------------------------------------------------------------
// Targets chosen for each opponent
// ---------------------------------------------------------------------------

#[test]
fn in_the_darkness_bind_them_chapter_four() {
    cr!("608.2b", "701.54a", "115.1");
    ruling!(
        "In the Darkness Bind Them",
        "When In the Darkness Bind Them's final chapter ability triggers, you can choose to target no creatures just so that the Ring tempts you. However, if you do choose at least one target, and all of those targets are illegal at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. The Ring won't tempt you."
    );
    ruling!(
        "In the Darkness Bind Them",
        "If a creature targeted by In the Darkness Bind Them's final chapter ability changes controllers before the ability resolves, that creature is no longer a legal target."
    );
    ruling!(
        "In the Darkness Bind Them",
        "Some spells and abilities that cause the Ring to tempt you may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. The Ring won't tempt you."
    );
    use crate::r_s19_common::add_lore;
    supported("In the Darkness Bind Them");
    let chapter_four = |t: &mut TestGame| {
        let saga = t.battlefield(P0, "In the Darkness Bind Them");
        t.g.objects[saga.0 as usize]
            .counters
            .insert("lore".into(), 3);
        add_lore(t, saga, 1);
        assert_eq!(t.stack_len(), 1);
    };
    // No target: the Ring still tempts P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[]);
    chapter_four(&mut t);
    let level = t.g.players[0].ring_level;
    t.resolve();
    assert_eq!(t.g.players[0].ring_level, level + 1);

    // P1's Hill Giant targeted; P0 gains control of it in response: it's no longer a
    // creature P1 controls, so the only target is illegal and nothing happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    chapter_four(&mut t);
    let level = t.g.players[0].ring_level;
    gain_control(&mut t, P0, giant);
    t.g.objects[giant.0 as usize].tapped = true;
    t.resolve();
    assert_eq!(t.g.players[0].ring_level, level);
    // "Untap those creatures" didn't happen either.
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn hideous_taskmaster_cast_trigger_resolves_first() {
    cr!("603.3", "405.5", "608.2b");
    ruling!(
        "Hideous Taskmaster",
        "Hideous Taskmaster's second ability will resolve before Hideous Taskmaster does. If Hideous Taskmaster is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    supported("Hideous Taskmaster");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.g.objects[giant.0 as usize].tapped = true;
    let spell = t.hand(P0, "Hideous Taskmaster");
    add_mana(&mut t, P0, ManaType::R, 7);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let id = t.cast(P0, spell).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The spell is countered in response; the trigger still resolves.
    assert!(t.g.counter(id, None));
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hideous Taskmaster"));
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(giant).controller, P0);
    assert!(!t.obj_now(giant).tapped);
}

#[test]
fn exiled_until_it_leaves_if_it_left_before_the_ability_resolved() {
    cr!("610.3b", "610.3c");
    ruling!(
        "Bronzebeak Foragers",
        "If Bronzebeak Foragers leaves the battlefield before its first ability resolves, none of the target permanents will be exiled."
    );
    ruling!(
        "Battle at the Helvault",
        "If Battle at the Helvault leaves the battlefield before its first or second chapter ability resolves, the target permanents won't be exiled."
    );
    ruling!(
        "Vault 13: Dweller's Journey",
        "If Vault 13 leaves the battlefield before its first chapter ability resolves, the target permanents won't be exiled at all."
    );
    supported("Battle at the Helvault");
    // Bronzebeak Foragers: one target per opponent; the Foragers leave first.
    let mut t = TestGame::new(3);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(b1)]);
    t.answer_targets(P0, &[Entity::Object(b2)]);
    let foragers = crate::r_s05_common::enter(&mut t, P0, "Bronzebeak Foragers");
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(foragers, None);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(b1) && t.on_battlefield(b2));
    // And when it stays, they're exiled.
    let mut t = TestGame::new(3);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(b1)]);
    t.answer_targets(P0, &[Entity::Object(b2)]);
    crate::r_s05_common::enter(&mut t, P0, "Bronzebeak Foragers");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));

    // Battle at the Helvault and Vault 13: a target for each player (P0's included).
    for saga in ["Battle at the Helvault", "Vault 13: Dweller's Journey"] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[Entity::Object(mine)]);
        t.answer_targets(P0, &[Entity::Object(theirs)]);
        let s = crate::r_s05_common::enter(&mut t, P0, saga);
        assert_eq!(t.stack_len(), 1, "{saga}");
        t.g.destroy(s, None);
        t.settle();
        t.resolve_all();
        assert!(t.on_battlefield(mine) && t.on_battlefield(theirs), "{saga}");
    }
}

// ---------------------------------------------------------------------------
// Each mode must target a different player
// ---------------------------------------------------------------------------

#[test]
fn shadrix_silverquill_zero_or_two_modes_each_a_different_player() {
    cr!("700.2", "700.2d", "603.3c");
    ruling!(
        "Shadrix Silverquill",
        "You may choose exactly zero modes or two modes. You can't choose only one mode. If you choose two modes, you choose which two and the target players as you put the triggered ability on the stack."
    );
    ruling!(
        "Shadrix Silverquill",
        "You can target a player with the third mode even if they control no creatures."
    );
    supported("Shadrix Silverquill");
    let combat = |t: &mut TestGame| {
        t.battlefield(P0, "Shadrix Silverquill");
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        // The modes were asked for.
        assert!(t.asked().iter().any(|(_, d)| matches!(
            d,
            mtg_engine::decision::Decision::ChooseModes { .. }
        )));
    };
    // Zero modes: the ability does nothing (it isn't put on the stack with modes).
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![]));
    let hand = t.hand_size(P1);
    combat(&mut t);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
    assert!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Inkling").is_empty());

    // One mode isn't allowed: it isn't a mode set of the right size.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    combat(&mut t);
    let chosen = t
        .g
        .stack
        .last()
        .map_or(0, |s| t.obj(*s).stack.as_ref().unwrap().chosen.len());
    assert_ne!(chosen, 1);
    t.resolve_all();

    // Two modes, chosen with their target players as the ability is put on the stack: P0
    // creates an Inkling, and P1 (who controls no creatures) is the third mode's target.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 2]));
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    combat(&mut t);
    let top = *t.g.stack.last().unwrap();
    let players: Vec<Entity> = t
        .obj(top)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect();
    assert_eq!(players, vec![Entity::Player(P0), Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Inkling").len(),
        1
    );
}

#[test]
fn phoenix_returns_targets_with_total_mana_value_6_counting_x_as_0() {
    cr!("202.3e", "115.1", "601.2c");
    ruling!(
        "Joshua, Phoenix's Dominant // Phoenix, Warden of Fire",
        "If a card in your graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    use crate::r_s19_common::add_lore;
    supported("Joshua, Phoenix's Dominant // Phoenix, Warden of Fire");
    // Phoenix, Warden of Fire's chapter III: "Return any number of target creature cards
    // with total mana value 6 or less from your graveyard to the battlefield."
    let mut t = TestGame::new(2);
    let walker = t.graveyard(P0, "Hangarback Walker");
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let phoenix = t.battlefield(P0, "Joshua, Phoenix's Dominant // Phoenix, Warden of Fire");
    assert!(mtg_engine::dfc::transform(&mut t.g, phoenix));
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(phoenix).face, FaceState::Back);
    t.g.objects[phoenix.0 as usize]
        .counters
        .insert("lore".into(), 2);
    t.recompute();
    // Hangarback Walker ({X}{X}: 0), Hill Giant (4) and Grizzly Bears (2) total 6; the
    // Llanowar Elves (1 more) don't fit.
    let from = t.asked().len();
    t.answer_targets(
        P0,
        &[walker, giant, bears, elves].map(Entity::Object),
    );
    add_lore(&mut t, phoenix, 1);
    let top = *t.g.stack.last().unwrap();
    let mut chosen: Vec<Entity> = t
        .obj(top)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect();
    chosen.sort();
    let mut expect = [walker, giant, bears].map(Entity::Object).to_vec();
    expect.sort();
    assert_eq!(chosen, expect, "{:?}", asked_since(&t, from));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

// ---------------------------------------------------------------------------
// The other cards of the gap: behavior tests
// ---------------------------------------------------------------------------

fn chosen_targets(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect()
}

#[test]
fn unlicensed_hearse_exiles_up_to_two_cards_from_a_single_graveyard() {
    cr!("115.1", "601.2c", "607.2a");
    supported("Unlicensed Hearse");
    let mut t = TestGame::new(2);
    let hearse = t.battlefield(P0, "Unlicensed Hearse");
    let mine = t.graveyard(P0, "Grizzly Bears");
    let a = t.graveyard(P1, "Hill Giant");
    let b = t.graveyard(P1, "Shock");
    // Answering a card from each graveyard: the second isn't from the same graveyard, so
    // only the first is chosen.
    t.answer_targets(P0, &[Entity::Object(mine), Entity::Object(a)]);
    let id = t.activate(P0, hearse, 0, &[]).unwrap().unwrap();
    assert_eq!(chosen_targets(&t, id), vec![Entity::Object(mine)]);
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    // As a creature (crewed), its power and toughness count the cards exiled with it.
    run(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![hearse])),
            mods: vec![Modification::AddTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.pt(hearse), (1, 1));
    // Two from P1's graveyard.
    t.g.objects[hearse.0 as usize].tapped = false;
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    let id = t.activate(P0, hearse, 0, &[]).unwrap().unwrap();
    assert_eq!(chosen_targets(&t, id).len(), 2);
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.pt(hearse), (3, 3));
}

#[test]
fn super_hero_civil_war_two_creatures_with_total_mana_value_6_or_less() {
    cr!("115.1", "601.2c", "611.2b");
    supported("The Super Hero Civil War");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // The Wurm (6) and the Bears (2) total 8: only the Wurm is taken.
    t.answer_targets(P0, &[Entity::Object(wurm), Entity::Object(bears)]);
    let saga = crate::r_s05_common::enter(&mut t, P0, "The Super Hero Civil War");
    let top = *t.g.stack.last().unwrap();
    assert_eq!(chosen_targets(&t, top), vec![Entity::Object(wurm)]);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).controller, P0);
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.obj_now(elves).controller, P1);
    // For as long as the Saga remains: it leaves, control returns.
    t.g.destroy(saga, None);
    t.settle();
    assert_eq!(t.obj_now(wurm).controller, P1);
}

#[test]
fn tocasia_returns_artifacts_with_total_mana_value_10_or_less() {
    cr!("115.1", "601.2c", "602.2b");
    supported("Tocasia, Dig Site Mentor");
    let setup = |t: &mut TestGame| -> ObjectId {
        let tocasia = t.graveyard(P0, "Tocasia, Dig Site Mentor");
        for c in [ManaType::G, ManaType::W, ManaType::U] {
            add_mana(t, P0, c, 3);
        }
        tocasia
    };
    // Steel Hellkite (6), Solemn Simulacrum (4) and Ornithopter (0): 10, all return.
    let mut t = TestGame::new(2);
    let tocasia = setup(&mut t);
    let v = [
        t.graveyard(P0, "Steel Hellkite"),
        t.graveyard(P0, "Solemn Simulacrum"),
        t.graveyard(P0, "Ornithopter"),
    ];
    t.answer_targets(P0, &v.map(Entity::Object));
    t.activate(P0, tocasia, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        names_on_battlefield(&t, P0),
        sorted(&["Ornithopter", "Solemn Simulacrum", "Steel Hellkite"])
    );
    // Myr Battlesphere (7) and Solemn Simulacrum (4): 11, only the first returns.
    let mut t = TestGame::new(2);
    let tocasia = setup(&mut t);
    let v = [
        t.graveyard(P0, "Myr Battlesphere"),
        t.graveyard(P0, "Solemn Simulacrum"),
    ];
    t.answer_targets(P0, &v.map(Entity::Object));
    t.activate(P0, tocasia, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Solemn Simulacrum"));
    assert_eq!(t.named_on_battlefield("Myr Battlesphere").len(), 1);
}

#[test]
fn balor_each_mode_targets_a_different_opponent() {
    cr!("700.2", "700.2d", "603.3c");
    supported("Balor");
    let balor_dies = |t: &mut TestGame| {
        let balor = t.battlefield(P0, "Balor");
        t.g.destroy(balor, None);
        t.g.flush_events();
        t.settle();
    };
    // One opponent: only one of the modes can be chosen.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 2]));
    balor_dies(&mut t);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).stack.as_ref().unwrap().chosen.len(), 1);
    t.resolve_all();

    // Two opponents, two modes: answering P1 for both, the second targets P2.
    let mut t = TestGame::new(3);
    for _ in 0..4 {
        t.hand(P2, "Grizzly Bears");
    }
    let ring = t.battlefield(P1, "Sol Ring");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 2]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    balor_dies(&mut t);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(
        chosen_targets(&t, top),
        vec![Entity::Player(P1), Entity::Player(P2)]
    );
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert_eq!(t.life(P2), 16);
}

#[test]
fn splinter_and_leo_each_mode_a_different_player() {
    cr!("700.2", "700.2d", "603.3c");
    supported("Splinter & Leo, Father & Son");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    crate::r_s05_common::enter(&mut t, P0, "Splinter & Leo, Father & Son");
    let top = *t.g.stack.last().unwrap();
    assert_eq!(
        chosen_targets(&t, top),
        vec![Entity::Player(P0), Entity::Player(P1)]
    );
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Mutant").len(),
        1
    );
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn age_of_ultron_destroys_up_to_one_creature_of_each_opponent() {
    cr!("115.1", "601.2c", "608.2b");
    // Chapter I: "For each opponent, destroy up to one target nonartifact creature that
    // player controls." (The card's chapter II isn't supported yet.)
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P2, "Ornithopter");
    let giant = t.battlefield(P2, "Hill Giant");
    let mine = t.battlefield(P0, "Llanowar Elves");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    crate::r_s05_common::enter(&mut t, P0, "Age of Ultron");
    let choices: Vec<Vec<Entity>> = asked_since(&t, from)
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0], vec![Entity::Object(bears)]);
    assert_eq!(choices[1], vec![Entity::Object(giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(thopter) && t.on_battlefield(mine));
}

#[test]
fn martyr_of_bones_exiles_up_to_x_cards_from_a_single_graveyard() {
    cr!("115.1", "601.2c", "601.2b");
    supported("Martyr of Bones");
    // "{1}, Reveal X black cards from your hand, Sacrifice this creature: Exile up to X
    // target cards from a single graveyard." X = 2: two of P1's cards; answering one of
    // P0's among them leaves only P1's.
    let mut t = TestGame::new(2);
    let martyr = t.battlefield(P0, "Martyr of Bones");
    let r1 = t.hand(P0, "Dark Ritual");
    let r2 = t.hand(P0, "Dark Ritual");
    let mine = t.graveyard(P0, "Grizzly Bears");
    let a = t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Shock");
    add_mana(&mut t, P0, ManaType::B, 1);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(r1), Entity::Object(r2)]);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(mine)]);
    let id = t.activate(P0, martyr, 0, &[]).unwrap().unwrap();
    assert_eq!(chosen_targets(&t, id), vec![Entity::Object(a)]);
    t.resolve();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn nils_a_target_for_each_player_is_optional() {
    cr!("115.1", "601.2c", "603.3d");
    ruling!(
        "Nils, Discipline Enforcer",
        "For each player, choosing a target creature that player controls is optional."
    );
    // "At the beginning of your end step, for each player, put a +1/+1 counter on up to
    // one target creature that player controls." (Nils's other ability isn't supported
    // yet.) A target for P1 only; none for P0 or P2.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Nils, Discipline Enforcer");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let p1 = t.battlefield(P1, "Hill Giant");
    let p2 = t.battlefield(P2, "Llanowar Elves");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[Entity::Object(p1)]);
    t.answer_targets(P0, &[]);
    t.advance_to(P0, Step::End);
    t.settle();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(chosen_targets(&t, top), vec![Entity::Object(p1)]);
    t.resolve_all();
    assert_eq!(t.counters(p1, "+1/+1"), 1);
    assert_eq!(t.counters(mine, "+1/+1"), 0);
    assert_eq!(t.counters(p2, "+1/+1"), 0);
}
