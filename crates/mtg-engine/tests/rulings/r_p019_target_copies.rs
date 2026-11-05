//! Rulings batch P019 — "Copy target activated or triggered ability you control" (CR
//! 707.10): Strionic Resonator (triggered abilities only), Tawnos, Urza's Apprentice
//! ("from an artifact source"), Abstruse Archaic ("from a colorless source"), Peter
//! Parker's Camera, The Peregrine Dynamo ("from another legendary source that's not a
//! commander") and Vantress Visions. A copy keeps the original's modes, division and cost
//! choices, may get new targets (CR 707.10c), has the same source (CR 707.10b), is linked
//! like the original (CR 607.2a), and makes its resolution choices separately (CR 608.2).

use crate::r_p019_common::*;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack, with_subtype};
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, attached_to};
use crate::r_s18_common::ADVENTURE;
use crate::r_s25_common::{abilities_from, change_copy_targets, targets_of};
use crate::r_s26_common::modes_on_stack;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `copier`'s "Copy target ..." ability (whose text contains `needle`) targets the topmost
/// object on the stack.
fn copy_top(t: &mut TestGame, copier: ObjectId, needle: &str) {
    let top = *t.g.stack.last().expect("nothing to copy");
    t.answer_targets(P0, &[Entity::Object(top)]);
    activate_containing(t, P0, copier, needle)
        .expect("copy ability")
        .expect("on the stack");
}

/// P0 casts Vantress Visions ("Copy target activated or triggered ability you control. You
/// may choose new targets for the copy.") targeting the topmost object on the stack.
fn vantress_top(t: &mut TestGame) {
    let top = *t.g.stack.last().expect("nothing to copy");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Virtue of Knowledge // Vantress Visions");
    t.cast(P0, card)
        .method(ADVENTURE)
        .target(Entity::Object(top))
        .go();
}

/// The division of the stack object's first target slot.
fn division(t: &TestGame, id: ObjectId) -> Vec<u32> {
    t.g.obj(id)
        .stack
        .as_deref()
        .and_then(|si| si.chosen.first())
        .and_then(|c| c.divided.first().cloned())
        .unwrap_or_default()
}

/// Number of activated and triggered abilities the permanent has.
fn ability_count(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id).chars.abilities.len()
}

#[test]
fn strionic_resonator_copy_of_a_trigger_with_a_cost_pays_it_separately() {
    cr!("707.10", "608.2", "707.10c");
    ruling!(
        "Strionic Resonator",
        "Any choices made when the triggered ability resolves won't have been made yet when it's copied. Any such choices will be made separately when the copy resolves. If the triggered ability asks you to pay a cost (such as that of Frenzied Goblin), you pay that cost for the copy."
    );
    supported("Frenzied Goblin");
    // Frenzied Goblin: "Whenever this creature attacks, you may pay {R}. If you do, target
    // creature can't block this turn." The copy (targeting the Giant) is paid for; the
    // original (targeting the Bears) isn't.
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    let goblin = t.battlefield(P0, "Frenzied Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(goblin, Entity::Player(P1))]);
    assert_eq!(abilities_from(&t, goblin).len(), 1);
    t.lands(P0, "Wastes", 2);
    copy_top(&mut t, resonator, "Copy target");
    t.lands(P0, "Mountain", 2);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(giant))]);
    t.resolve();
    assert_eq!(abilities_from(&t, goblin).len(), 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(yes_nos(&t, from), 2);
    assert!(!t.g.can_block_at_all(giant));
    assert!(t.g.can_block_at_all(bears));
}

#[test]
fn strionic_resonator_copy_of_a_linked_trigger_is_linked_too() {
    cr!("707.10", "607.2a", "607.3");
    ruling!(
        "Strionic Resonator",
        "If a triggered ability is linked to a second ability, copies of that triggered ability are also linked to that second ability. If the second ability refers to \"the exiled card,\" it refers to all cards exiled by the triggered ability and the copy. For example, if Fiend Hunter's enters-the-battlefield ability is copied and two creatures are exiled, they both return when Fiend Hunter leaves the battlefield."
    );
    supported("Fiend Hunter");
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    let hunter = t.enter(P0, "Fiend Hunter");
    t.settle();
    t.lands(P0, "Wastes", 2);
    copy_top(&mut t, resonator, "Copy target");
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    destroy(&mut t, hunter);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn strionic_resonator_copy_keeps_the_division_and_number_of_targets() {
    cr!("707.10c", "601.2d", "115.7f");
    ruling!(
        "Strionic Resonator",
        "If the triggered ability divides damage or distributes counters among a number of targets (for example, the ability of Bogardan Hellkite), the division and number of targets can't be changed. If you choose new targets, you must choose the same number of targets."
    );
    supported("Bogardan Hellkite");
    // Bogardan Hellkite: "When this creature enters, it deals 5 damage divided as you
    // choose among any number of targets." 3 to P1, 2 to the Bears; the copy hits the
    // Giant instead of the Bears.
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1), Entity::Object(bears)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 2]));
    let hellkite = t.enter(P0, "Bogardan Hellkite");
    t.settle();
    let original = *t.g.stack.last().unwrap();
    assert_eq!(division(&t, original), vec![3, 2]);
    t.lands(P0, "Wastes", 2);
    copy_top(&mut t, resonator, "Copy target");
    change_copy_targets(&mut t, P0, &[None, Some(Entity::Object(giant))]);
    let from = t.asked().len();
    t.resolve();
    let both = abilities_from(&t, hellkite);
    assert_eq!(both.len(), 2);
    let copy = both[1];
    assert_eq!(
        targets_of(&t, copy),
        vec![Entity::Player(P1), Entity::Object(giant)]
    );
    assert_eq!(division(&t, copy), vec![3, 2]);
    assert_eq!(
        asked_count(&t, from, |d| matches!(
            d,
            mtg_engine::decision::Decision::Divide { .. }
        )),
        0
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.obj_now(giant).damage, 2);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn strionic_resonator_copy_of_a_modal_trigger_keeps_its_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Strionic Resonator",
        "If the triggered ability is modal (that is, if it says, \"Choose one —\" or similar), the mode is copied and can't be changed."
    );
    supported("Charming Prince");
    // Charming Prince: "When this creature enters, choose one — • Scry 2. • You gain 3
    // life. • Exile another target creature you own. ..."
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    let prince = t.enter(P0, "Charming Prince");
    t.settle();
    t.lands(P0, "Wastes", 2);
    copy_top(&mut t, resonator, "Copy target");
    let from = t.asked().len();
    t.resolve();
    let both = abilities_from(&t, prince);
    assert_eq!(both.len(), 2);
    assert!(both.iter().all(|a| modes_on_stack(&t, *a) == vec![1]));
    assert!(!asked_modes_or_x(&t, from));
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn strionic_resonator_creates_an_instance_of_the_ability_without_granting_one() {
    cr!("707.10", "707.10b");
    ruling!(
        "Strionic Resonator",
        "Strionic Resonator targets a triggered ability that has triggered and is on the stack and creates another instance of that ability on the stack. It doesn't cause any object to gain an ability."
    );
    // Ajani's Pridemate: "Whenever you gain life, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    let before = ability_count(&mut t, pridemate);
    let resonator_abilities = ability_count(&mut t, resonator);
    crate::r_s25_common::cast_new(&mut t, P0, "Angel's Mercy", &[]);
    t.resolve();
    t.lands(P0, "Wastes", 2);
    copy_top(&mut t, resonator, "Copy target");
    t.resolve();
    assert_eq!(abilities_from(&t, pridemate).len(), 2);
    assert_eq!(ability_count(&mut t, pridemate), before);
    assert_eq!(ability_count(&mut t, resonator), resonator_abilities);
    t.resolve_all();
    assert_eq!(t.counters(pridemate, counters::PLUS1), 2);
    // Another life gain later triggers the Pridemate once.
    crate::r_s25_common::cast_new(&mut t, P0, "Angel's Mercy", &[]);
    t.resolve();
    assert_eq!(abilities_from(&t, pridemate).len(), 1);
}

#[test]
fn strionic_resonator_copies_a_keyword_triggered_ability() {
    cr!("603.1", "702.108a", "707.10");
    ruling!(
        "Strionic Resonator",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\""
    );
    supported("Monastery Swiftspear");
    // Monastery Swiftspear's prowess ("Whenever you cast a noncreature spell, this
    // creature gets +1/+1 until end of turn") is a triggered ability.
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    let swiftspear = t.battlefield(P0, "Monastery Swiftspear");
    crate::r_s25_common::cast_new(&mut t, P0, "Angel's Mercy", &[]);
    t.settle();
    assert_eq!(abilities_from(&t, swiftspear).len(), 1);
    t.lands(P0, "Wastes", 2);
    copy_top(&mut t, resonator, "Copy target");
    t.resolve();
    assert_eq!(abilities_from(&t, swiftspear).len(), 2);
    t.resolve();
    t.resolve();
    assert_eq!(t.pt(swiftspear), (3, 4));
}

#[test]
fn defabricate_counters_a_keyword_triggered_ability() {
    cr!("603.1", "702.108a", "701.6a");
    ruling!(
        "Defabricate",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\""
    );
    supported("Defabricate");
    // Defabricate: "Choose one — • Counter target artifact or enchantment spell. ... •
    // Counter target activated or triggered ability." It counters P1's prowess trigger.
    let mut t = TestGame::new(2);
    let swiftspear = t.battlefield(P1, "Monastery Swiftspear");
    crate::r_s25_common::cast_new(&mut t, P1, "Angel's Mercy", &[]);
    t.settle();
    let prowess = *t.g.stack.last().unwrap();
    assert_eq!(abilities_from(&t, swiftspear), vec![prowess]);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let defabricate = t.hand(P0, "Defabricate");
    t.cast(P0, defabricate)
        .modes(&[1])
        .target(Entity::Object(prowess))
        .go();
    t.resolve_all();
    assert_eq!(t.pt(swiftspear), (1, 2));
    assert_eq!(t.life(P1), 27);
}

#[test]
fn tawnos_copies_the_cycling_ability_of_an_artifact_card() {
    cr!("707.10", "702.29a", "113.7");
    ruling!(
        "Tawnos, Urza's Apprentice",
        "An \"artifact source\" is a permanent, spell, or card in any zone with the card type \"artifact.\" For example, Tawnos's ability can target a cycling ability you've activated if the discarded card is an artifact card."
    );
    supported("Runaway Boulder");
    // Runaway Boulder (an artifact card): "Cycling {2}". The copy draws a card too.
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Urza's Apprentice");
    let boulder = t.hand(P0, "Runaway Boulder");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, boulder, "Cycling")
        .unwrap()
        .unwrap();
    assert!(t.in_graveyard(P0, "Runaway Boulder"));
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    copy_top(&mut t, tawnos, "Copy target");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn tawnos_copy_of_mirrorworks_trigger_pays_its_cost_separately() {
    cr!("707.10", "608.2", "707.10b");
    ruling!(
        "Tawnos, Urza's Apprentice",
        "Any choices made when the ability resolves won't have been made yet when it's copied. Any such choices will be made separately when the copy resolves. If a triggered ability asks you to pay a cost (such as that of Mirrorworks), you pay that cost for the copy separately."
    );
    ruling!(
        "Tawnos, Urza's Apprentice",
        "Tawnos's ability targets an ability that is on the stack and creates another instance of that ability on the stack. It doesn't cause any object to gain an ability."
    );
    supported("Mirrorworks");
    // Mirrorworks: "Whenever another nontoken artifact you control enters, you may pay
    // {2}. If you do, create a token that's a copy of that artifact."
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Urza's Apprentice");
    let mirrorworks = t.battlefield(P0, "Mirrorworks");
    let before = ability_count(&mut t, mirrorworks);
    t.enter(P0, "Ornithopter");
    t.settle();
    assert_eq!(abilities_from(&t, mirrorworks).len(), 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    copy_top(&mut t, tawnos, "Copy target");
    t.resolve();
    assert_eq!(abilities_from(&t, mirrorworks).len(), 2);
    assert_eq!(ability_count(&mut t, mirrorworks), before);
    t.lands(P0, "Wastes", 4);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_nos(&t, from), 2);
    let thopters = t.named_on_battlefield("Ornithopter");
    assert_eq!(thopters.len(), 3);
    assert_eq!(
        thopters
            .iter()
            .filter(|o| t.obj_now(**o).is_token())
            .count(),
        2
    );
}

#[test]
fn tawnos_copy_keeps_the_distribution_of_counters() {
    cr!("707.10c", "601.2d", "115.7f");
    ruling!(
        "Tawnos, Urza's Apprentice",
        "If the ability divides damage or distributes counters among a number of targets (for example, the ability of Verdurous Gearhulk), the division and number of targets can't be changed. If you choose new targets, you must choose the same number of targets."
    );
    supported("Verdurous Gearhulk");
    // Verdurous Gearhulk (an artifact): "When this creature enters, distribute four +1/+1
    // counters among any number of target creatures you control." 3 to the Bears and 1 to
    // the Elves; the copy puts the 3 on the Giant instead.
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Urza's Apprentice");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(elves)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 1]));
    t.enter(P0, "Verdurous Gearhulk");
    t.settle();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    copy_top(&mut t, tawnos, "Copy target");
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(giant)), None]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(
        targets_of(&t, copy),
        vec![Entity::Object(giant), Entity::Object(elves)]
    );
    assert_eq!(division(&t, copy), vec![3, 1]);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 3);
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
    assert_eq!(t.counters(elves, counters::PLUS1), 2);
}

#[test]
fn tawnos_copy_of_an_imprint_trigger_makes_the_linked_ability_use_both_cards() {
    cr!("707.10", "607.2a", "607.3", "107.3");
    ruling!(
        "Tawnos, Urza's Apprentice",
        "In some cases involving linked abilities, an ability requires information about \"the exiled card.\" When this happens, the ability gets multiple answers. If these answers are being used to determine the value of a variable, the sum is used. For example, if Soul Foundry's enters-the-battlefield ability is copied, two cards are exiled. The value of X in the activation cost of Soul Foundry's other ability is the sum of the two cards' mana values. As the ability resolves, you create a token copy of each card."
    );
    supported("Soul Foundry");
    // Soul Foundry: "Imprint — When this artifact enters, you may exile a creature card
    // from your hand. {X}, {T}: Create a token that's a copy of the exiled card. X is the
    // mana value of that card." Grizzly Bears (2) and Llanowar Elves (1): X is 3.
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Urza's Apprentice");
    let bears = t.hand(P0, "Grizzly Bears");
    let elves = t.hand(P0, "Llanowar Elves");
    let foundry = t.enter(P0, "Soul Foundry");
    t.settle();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    copy_top(&mut t, tawnos, "Copy target");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(elves), Zone::Exile);
    t.lands(P0, "Wastes", 2);
    assert!(activate_containing(&mut t, P0, foundry, "token").is_err());
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, foundry, "token")
        .unwrap()
        .unwrap();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 5);
    t.resolve_all();
    let tokens = crate::r_s01_common::tokens(&t, P0);
    let mut names: Vec<String> = tokens
        .iter()
        .map(|o| t.obj_now(*o).chars.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Llanowar Elves"]);
}

#[test]
fn tawnos_copies_a_living_weapon_trigger() {
    cr!("603.1", "702.92a", "707.10");
    ruling!(
        "Tawnos, Urza's Apprentice",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    supported("Flayer Husk");
    // Flayer Husk: "Living weapon (When this Equipment enters, create a 0/0 black
    // Phyrexian Germ creature token, then attach this to it.)" The copy makes a Germ and
    // attaches the Husk to it; then the original makes another and moves the Husk.
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Urza's Apprentice");
    let husk = t.enter(P0, "Flayer Husk");
    t.settle();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    copy_top(&mut t, tawnos, "Copy target");
    t.resolve();
    assert_eq!(abilities_from(&t, husk).len(), 2);
    t.resolve();
    let first = with_subtype(&t, P0, "Germ");
    assert_eq!(first.len(), 1);
    assert_eq!(attached_to(&t, husk), Some(Entity::Object(first[0])));
    t.resolve_all();
    let second = with_subtype(&t, P0, "Germ");
    assert_eq!(second.len(), 1);
    assert_ne!(second, first);
    assert_eq!(attached_to(&t, husk), Some(Entity::Object(second[0])));
}

/// P0's `copier` copies Bow of Nylea's "{1}{G}, {T}: Choose one — ... • You gain 3 life.
/// ..." ability (a legendary artifact's): the copy keeps the mode and P0 gains 6.
fn copy_bows_life_mode(copier: &str, needle: &str, copier_lands: &[(&str, usize)]) {
    let mut t = TestGame::new(2);
    let copier = t.enter(P0, copier);
    t.settle();
    t.resolve_all();
    let bow = t.battlefield(P0, "Bow of Nylea");
    t.lands(P0, "Forest", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    let ability = activate_containing(&mut t, P0, bow, "Choose one")
        .unwrap()
        .unwrap();
    for (land, n) in copier_lands {
        t.lands(P0, land, *n);
    }
    copy_top(&mut t, copier, needle);
    let from = t.asked().len();
    t.resolve();
    let both = abilities_from(&t, bow);
    assert_eq!(both.len(), 2);
    assert!(both
        .iter()
        .all(|a| modes_on_stack(&t, *a) == modes_on_stack(&t, ability)));
    assert!(!asked_modes_or_x(&t, from));
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn copies_of_a_modal_ability_keep_the_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Peter Parker's Camera",
        "If the ability is modal (that is, if it says, \"Choose one —\" or similar), the mode is copied and can't be changed."
    );
    ruling!(
        "Tawnos, Urza's Apprentice",
        "If the ability is modal (that is, if it says, \"Choose one —\" or similar), the mode is copied and can't be changed."
    );
    ruling!(
        "The Peregrine Dynamo",
        "If the ability is modal (that is, if it says, “Choose one —” or similar), the mode is copied and can’t be changed."
    );
    supported("Bow of Nylea");
    // Bow of Nylea is a legendary enchantment artifact: an artifact source for Tawnos and
    // another legendary source for The Peregrine Dynamo.
    copy_bows_life_mode("Peter Parker's Camera", "Copy target", &[("Wastes", 2)]);
    copy_bows_life_mode(
        "Tawnos, Urza's Apprentice",
        "Copy target",
        &[("Island", 1), ("Mountain", 1)],
    );
    copy_bows_life_mode("The Peregrine Dynamo", "Copy target", &[("Wastes", 1)]);
}

#[test]
fn abstruse_archaic_copy_of_a_modal_ability_from_a_sacrificed_colorless_source() {
    cr!("707.10", "700.2g", "113.7a");
    ruling!(
        "Abstruse Archaic",
        "If the ability is modal (that is, if it says, \"Choose one —\" or similar), the mode is copied and can't be changed."
    );
    supported("Balm of Restoration");
    // Balm of Restoration: "{1}, {T}, Sacrifice this artifact: Choose one — • You gain 2
    // life. • Prevent the next 2 damage that would be dealt to any target this turn." Its
    // source (colorless, as it last existed) is gone by the time it's copied.
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    let balm = t.battlefield(P0, "Balm of Restoration");
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    activate_containing(&mut t, P0, balm, "Choose one")
        .unwrap()
        .unwrap();
    assert!(!t.on_battlefield(balm));
    t.lands(P0, "Wastes", 1);
    copy_top(&mut t, archaic, "Copy target");
    let from = t.asked().len();
    t.resolve();
    assert_eq!(abilities_from(&t, balm).len(), 2);
    assert!(abilities_from(&t, balm)
        .iter()
        .all(|a| modes_on_stack(&t, *a) == vec![0]));
    assert!(!asked_modes_or_x(&t, from));
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn vantress_visions_copy_makes_resolution_choices_separately() {
    cr!("707.10", "608.2", "603.5");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "Any choices made when the ability resolves won't have been made yet when it's copied by Vantress Visions. Any such choices will be made separately when the copy resolves. If a triggered ability asks you to pay a cost, you pay that cost for the copy separately."
    );
    supported("Keldon Raider");
    // Keldon Raider: "When this creature enters, you may discard a card. If you do, draw a
    // card." The copy: discard and draw. The original: don't.
    let mut t = TestGame::new(2);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.enter(P0, "Keldon Raider");
    t.settle();
    vantress_top(&mut t);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(yes_nos(&t, from), 2);
    // Vantress Visions (in exile, on an adventure) and one discarded Island.
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn vantress_visions_copy_keeps_the_division_and_mode() {
    cr!("707.10c", "601.2d", "700.2g");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "If the ability copied by Vantress Visions divides damage or distributes counters among a number of targets, the division and number of targets can't be changed. If you choose new targets, you must choose the same number of targets."
    );
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "If the ability copied by Vantress Visions is modal (that is, if it says, \"Choose one —\" or similar), the mode is copied and can't be changed."
    );
    // Bogardan Hellkite: 3 to P1 and 2 to the Bears; the copy hits the Giant instead.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1), Entity::Object(bears)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 2]));
    let hellkite = t.enter(P0, "Bogardan Hellkite");
    t.settle();
    vantress_top(&mut t);
    change_copy_targets(&mut t, P0, &[None, Some(Entity::Object(giant))]);
    t.resolve();
    let copy = abilities_from(&t, hellkite)[1];
    assert_eq!(division(&t, copy), vec![3, 2]);
    assert_eq!(targets_of(&t, copy).len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.obj_now(giant).damage, 2);
    // Charming Prince: the "gain 3 life" mode is copied.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    let prince = t.enter(P0, "Charming Prince");
    t.settle();
    vantress_top(&mut t);
    let from = t.asked().len();
    t.resolve();
    assert!(abilities_from(&t, prince)
        .iter()
        .all(|a| modes_on_stack(&t, *a) == vec![1]));
    assert!(!asked_modes_or_x(&t, from));
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn vantress_visions_copy_uses_the_cost_choices_of_the_original() {
    cr!("707.10", "601.2h");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "If the activated ability's cost contains a choice, such as a creature to sacrifice or a number of counters to remove, the copy created by Vantress Visions uses that same information. You can't pay the cost again, even if you want to."
    );
    supported("Disciple of Griselbrand");
    // Disciple of Griselbrand: "{1}, Sacrifice a creature: You gain life equal to the
    // sacrificed creature's toughness." Sacrificing the Hill Giant (3): each gains 3.
    let mut t = TestGame::new(2);
    let disciple = t.battlefield(P0, "Disciple of Griselbrand");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, disciple, "Sacrifice")
        .unwrap()
        .unwrap();
    vantress_top(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(disciple));
}

#[test]
fn vantress_visions_copy_has_the_same_source() {
    cr!("707.10b");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "The source of the copy created by Vantress Visions is the same as the source of the original ability."
    );
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    crate::r_s25_common::cast_new(&mut t, P0, "Angel's Mercy", &[]);
    t.resolve();
    vantress_top(&mut t);
    t.resolve();
    assert_eq!(abilities_from(&t, pridemate).len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(pridemate, counters::PLUS1), 2);
}
