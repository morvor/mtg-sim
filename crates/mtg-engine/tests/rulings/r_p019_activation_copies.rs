//! Rulings batch P019 — abilities that copy an ability as it's activated (CR 707.10,
//! 602.2, 603.2): Battlemage's Bracers ("Whenever an ability of equipped creature is
//! activated, if it isn't a mana ability, you may pay {1}. If you do, copy that ability."),
//! Kurkesh, Onakke Ancient ("Whenever you activate an ability of an artifact, if it isn't
//! a mana ability, you may pay {R}."), Rings of Brighthearth ("Whenever you activate an
//! ability, if it isn't a mana ability, you may pay {2}."), Verrak, Warped Sengir ("if life
//! was paid to activate it, you may pay that much life again") and Ertha Jo ("Whenever
//! you activate an ability that targets a creature or player, copy that ability."). A copy
//! keeps the original's X, modes and cost choices (CR 707.10), may get new targets (CR
//! 707.10c), and isn't activated (so it isn't copied again, and isn't a spell for
//! magecraft).

use crate::r_p019_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use crate::r_s25_common::{abilities_from, change_copy_targets, targets_of, x_of};
use crate::r_s26_common::modes_on_stack;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn battlemages_bracers_copy_keeps_x_and_the_mode() {
    cr!("707.10", "107.3", "700.2g", "603.2");
    ruling!(
        "Battlemage's Bracers",
        "If the ability has {X} in its activation cost, the copy has the same value of X."
    );
    ruling!(
        "Battlemage's Bracers",
        "If the ability is modal (that is, it has a bulleted list of modes), the copy will have the same mode(s). You can’t choose new ones."
    );
    supported("Battlemage's Bracers");
    supported("Atalya, Samite Master");
    // Atalya: "{X}, {T}: Choose one — • Prevent the next X damage that would be dealt to
    // target creature this turn. Spend only white mana on X. • You gain X life. Spend only
    // white mana on X."
    let mut t = TestGame::new(2);
    let atalya = t.battlefield(P0, "Atalya, Samite Master");
    attach_new(&mut t, P0, "Battlemage's Bracers", atalya);
    t.lands(P0, "Plains", 3);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let ability = activate_containing(&mut t, P0, atalya, "Choose one")
        .unwrap()
        .unwrap();
    assert_eq!(x_of(&t, ability), Some(2));
    t.settle();
    // Pay {1} and copy it.
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.resolve();
    let both = abilities_from(&t, atalya);
    assert_eq!(both.len(), 2);
    for a in &both {
        assert_eq!(modes_on_stack(&t, *a), vec![1]);
        assert_eq!(x_of(&t, *a), Some(2));
    }
    assert!(!asked_modes_or_x(&t, from));
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn battlemages_bracers_copy_may_change_targets_or_keep_one_with_no_new_legal_choice() {
    cr!("707.10c", "115.7d");
    ruling!(
        "Battlemage's Bracers",
        "The copy will have the same targets as the ability it’s copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can’t choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Prodigal Pyromancer");
    supported("Royal Assassin");
    // Prodigal Pyromancer: "{T}: This creature deals 1 damage to any target." The copy's
    // target is changed to P1.
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    attach_new(&mut t, P0, "Battlemage's Bracers", pyro);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    let ping = activate_containing(&mut t, P0, pyro, "damage")
        .unwrap()
        .unwrap();
    t.settle();
    t.answer_yes(P0, true);
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, ping);
    assert_eq!(targets_of(&t, copy), vec![Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!t.on_battlefield(elves));

    // Royal Assassin: "{T}: Destroy target tapped creature." Its target untaps before the
    // copy is made, and there's no other tapped creature: the copy keeps that (now
    // illegal) target.
    let mut t = TestGame::new(2);
    let assassin = t.battlefield(P0, "Royal Assassin");
    attach_new(&mut t, P0, "Battlemage's Bracers", assassin);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, assassin, "Destroy")
        .unwrap()
        .unwrap();
    t.settle();
    t.g.untap(bears);
    t.answer_yes(P0, true);
    change_copy_targets(&mut t, P0, &[None]);
    t.resolve();
    let both = abilities_from(&t, assassin);
    assert_eq!(both.len(), 2);
    assert_eq!(targets_of(&t, both[1]), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn battlemages_bracers_copy_doesnt_trigger_magecraft() {
    cr!("707.10");
    ruling!(
        "Battlemage's Bracers",
        "Copying abilities doesn’t cause magecraft abilities to trigger."
    );
    supported("Archmage Emeritus");
    // Archmage Emeritus: "Magecraft — Whenever you cast or copy an instant or sorcery
    // spell, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archmage Emeritus");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    attach_new(&mut t, P0, "Battlemage's Bracers", pyro);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, pyro, "damage").unwrap();
    t.settle();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(abilities_from(&t, pyro).len(), 2);
    assert_eq!(triggers_on_stack(&t, "Magecraft"), 0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn battlemages_bracers_copies_keyword_activated_abilities_but_not_mana_abilities() {
    cr!("602.1", "605.1a", "702.87a", "603.2");
    ruling!(
        "Battlemage's Bracers",
        "Activated abilities contain a colon. They’re generally written “[Cost]: [Effect].” Some keyword abilities (such as boast) are activated abilities and will have colons in their reminder text. An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    supported("Student of Warfare");
    // Student of Warfare: "Level up {W}" — an activated keyword ability: copied.
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Student of Warfare");
    attach_new(&mut t, P0, "Battlemage's Bracers", student);
    t.lands(P0, "Plains", 2);
    t.activate(P0, student, 0, &[]).unwrap();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "copy that ability"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(student, "level"), 2);
    // Llanowar Elves: "{T}: Add {G}." — a mana ability: Bracers doesn't trigger.
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Battlemage's Bracers", elves);
    activate_containing(&mut t, P0, elves, "Add").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn kurkesh_copies_an_artifacts_activated_keyword_ability() {
    cr!("602.1", "702.6a", "707.10c");
    ruling!(
        "Kurkesh, Onakke Ancient",
        "Activated abilities are written in the form \"Cost: Effect.\" Some keywords are activated abilities and will have colons in their reminder texts."
    );
    supported("Bonesplitter");
    // Bonesplitter: "Equip {1}" — the copy, with a new target, attaches it to the Bears;
    // then the original to the Elves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kurkesh, Onakke Ancient");
    let splitter = t.battlefield(P0, "Bonesplitter");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    activate_containing(&mut t, P0, splitter, "Equip").unwrap();
    t.settle();
    t.answer_yes(P0, true);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(bears))]);
    t.resolve();
    assert_eq!(abilities_from(&t, splitter).len(), 2);
    t.resolve();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
    t.resolve();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(elves)));
}

#[test]
fn kurkesh_doesnt_copy_mana_abilities() {
    cr!("605.1a", "603.2");
    ruling!(
        "Kurkesh, Onakke Ancient",
        "A mana ability is an ability that (1) could add mana when it resolves, (2) isn't a loyalty ability, and (3) doesn't require a target."
    );
    supported("Ratchet Bomb");
    // Sol Ring's "{T}: Add {C}{C}." is a mana ability; Ratchet Bomb's "{T}: Put a charge
    // counter on this artifact." isn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kurkesh, Onakke Ancient");
    let ring = t.battlefield(P0, "Sol Ring");
    activate_containing(&mut t, P0, ring, "Add {C}{C}").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    mtg_engine::mana_abilities::empty_pool(&mut t.g, P0);
    let bomb = t.battlefield(P0, "Ratchet Bomb");
    t.lands(P0, "Mountain", 1);
    activate_containing(&mut t, P0, bomb, "charge counter").unwrap();
    t.settle();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(bomb, "charge"), 2);
}

#[test]
fn kurkesh_sacrificed_to_pay_a_cost_doesnt_copy_that_ability() {
    cr!("602.2b", "601.2i", "603.2");
    ruling!(
        "Kurkesh, Onakke Ancient",
        "If paying the activation cost of the ability includes sacrificing Kurkesh, the ability won't be copied. At the time the ability is considered activated (after all costs are paid), Kurkesh is no longer on the battlefield."
    );
    supported("Blasting Station");
    // Blasting Station: "{T}, Sacrifice a creature: This artifact deals 1 damage to any
    // target."
    let mut t = TestGame::new(2);
    let kurkesh = t.battlefield(P0, "Kurkesh, Onakke Ancient");
    let station = t.battlefield(P0, "Blasting Station");
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(kurkesh)]);
    activate_containing(&mut t, P0, station, "Sacrifice a creature")
        .unwrap()
        .unwrap();
    t.settle();
    assert!(!t.on_battlefield(kurkesh));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn rings_of_brighthearth_pays_once_per_trigger() {
    cr!("603.2", "707.10");
    ruling!(
        "Rings of Brighthearth",
        "You can't pay {2} more than once for each time the triggered ability of Rings of Brighthearth resolves."
    );
    supported("Rings of Brighthearth");
    // Plenty of mana: still one payment and one copy (the copy isn't activated, so it
    // doesn't trigger the Rings again).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rings of Brighthearth");
    let bomb = t.battlefield(P0, "Ratchet Bomb");
    t.lands(P0, "Wastes", 8);
    activate_containing(&mut t, P0, bomb, "charge counter").unwrap();
    t.settle();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.resolve();
    assert_eq!(yes_nos(&t, from), 1);
    assert_eq!(abilities_from(&t, bomb).len(), 2);
    assert_eq!(triggers_on_stack(&t, "pay {2}"), 0);
    t.resolve_all();
    assert_eq!(t.counters(bomb, "charge"), 2);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
}

#[test]
fn verraks_copy_makes_its_resolution_choices_separately() {
    cr!("707.10", "608.2");
    ruling!(
        "Verrak, Warped Sengir",
        "Any choices made when the ability resolves won’t have been made yet when it’s copied. Any such choices will be made separately when the copy resolves."
    );
    supported("Verrak, Warped Sengir");
    supported("Arid Mesa");
    // Arid Mesa: "{T}, Pay 1 life, Sacrifice this land: Search your library for a Mountain
    // or Plains card, put it onto the battlefield, then shuffle." Verrak: pay 1 life again
    // and copy it. The copy finds the Mountain; the original (its source gone) the Plains.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verrak, Warped Sengir");
    let mesa = t.battlefield(P0, "Arid Mesa");
    let mountain = t.library_top(P0, "Mountain");
    let plains = t.library_top(P0, "Plains");
    activate_containing(&mut t, P0, mesa, "Search").unwrap();
    t.settle();
    assert_eq!(t.life(P0), 19);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(mountain)]);
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.named_on_battlefield("Mountain").len(), 1);
    assert_eq!(t.named_on_battlefield("Plains").len(), 1);
}

#[test]
fn ertha_jos_copy_of_a_modal_ability_keeps_its_mode() {
    cr!("707.10", "700.2g", "707.10c");
    ruling!(
        "Ertha Jo, Frontier Mentor",
        "If the ability is modal (that is, if it says, \"Choose one —\" or similar), the mode is copied and can't be changed."
    );
    supported("Ertha Jo, Frontier Mentor");
    supported("Kargan Intimidator");
    // Kargan Intimidator: "{1}: Choose one that hasn't been chosen this turn — • This
    // creature gets +1/+1 until end of turn. • Target creature becomes a Coward until end
    // of turn. • Target Warrior gains trample until end of turn." Ertha Jo copies the
    // Coward mode; the copy targets the Hill Giant.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ertha Jo, Frontier Mentor");
    let kargan = t.battlefield(P0, "Kargan Intimidator");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let ability = activate_containing(&mut t, P0, kargan, "Choose one")
        .unwrap()
        .unwrap();
    t.settle();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(giant))]);
    let from = t.asked().len();
    t.resolve();
    let both = abilities_from(&t, kargan);
    assert_eq!(both.len(), 2);
    assert!(both.iter().all(|a| modes_on_stack(&t, *a) == vec![1]));
    assert_eq!(modes_on_stack(&t, ability), vec![1]);
    assert!(!asked_modes_or_x(&t, from));
    t.resolve_all();
    assert!(t.obj_now(bears).chars.has_subtype("Coward"));
    assert!(t.obj_now(giant).chars.has_subtype("Coward"));
    assert_eq!(t.pt(kargan), (3, 1));
}

#[test]
fn verraks_copy_of_a_modal_ability_keeps_its_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Verrak, Warped Sengir",
        "If the ability is modal (that is, if it says, “Choose one —” or similar), the mode is copied and can’t be changed."
    );
    // No real card has a modal activated ability with a life cost, so a custom artifact:
    // "{1}, Pay 2 life: Choose one — • You gain 5 life. • Draw a card."
    let relic = crate::r_s01_common::custom_card(
        "Modal Relic",
        "Artifact",
        "{1}",
        None,
        "{1}, Pay 2 life: Choose one —\n• You gain 5 life.\n• Draw a card.",
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verrak, Warped Sengir");
    let relic = t.custom(P0, relic, mtg_engine::object::Zone::Battlefield);
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    activate_containing(&mut t, P0, relic, "Choose one")
        .unwrap()
        .unwrap();
    t.settle();
    assert_eq!(t.life(P0), 18);
    // Pay 2 life again and copy it.
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.resolve();
    assert_eq!(t.life(P0), 16);
    let both = abilities_from(&t, relic);
    assert_eq!(both.len(), 2);
    assert!(both.iter().all(|a| modes_on_stack(&t, *a) == vec![0]));
    assert!(!asked_modes_or_x(&t, from));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.hand_size(P0), hand);
}
