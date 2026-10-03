//! Rulings on Class cards (CR 716): levels and level bars, the abilities of each level,
//! and several Class permanents at once.

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s05_common::move_to;
use crate::r_s19_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Gains `p` 2 life as an effect would (replacement effects apply), and settles.
fn gain_2(t: &mut TestGame, p: PlayerId) {
    t.g.gain_life(p, 2);
    t.g.flush_events();
    t.settle();
}

/// The levels the activated level abilities (class level bars) of `id` advance it to.
fn level_abilities(t: &mut TestGame, id: ObjectId) -> Vec<u32> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(act) => match act.body.effect {
                Effect::SetClassLevel { level } => Some(level),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn a_class_starts_with_its_first_ability_and_gains_the_next_as_a_level_ability_resolves() {
    cr!("716.2a", "716.3");
    ruling!(
        "Cleric Class",
        "Each Class starts with only the first of three class abilities. As the first level ability resolves, the Class becomes level 2 and gains the second class ability."
    );
    supported("Cleric Class");
    // Cleric Class: "If you would gain life, you gain that much life plus 1 instead."
    // Level 2: "Whenever you gain life, put a +1/+1 counter on target creature you
    // control." Level 3: "When this Class becomes level 3, return target creature card
    // from your graveyard to the battlefield. You gain life equal to that creature's
    // toughness."
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Cleric Class");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 9);
    // Level 1: only the first ability.
    gain_2(&mut t, P0);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.stack_len(), 0, "no level 2 trigger at level 1");
    // The first level ability on the stack: the Class is still level 1.
    gain_level(&mut t, P0, class, 2).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(level(&t, class), 1);
    gain_2(&mut t, P0);
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.stack_len(), 1, "still no level 2 trigger");
    // As it resolves, the Class becomes level 2 and has the second ability.
    t.resolve();
    assert_eq!(level(&t, class), 2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    gain_2(&mut t, P0);
    assert_eq!(t.life(P0), 29);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // The second level ability: level 3 and its ability ("When this Class becomes level
    // 3, ...") only as it resolves.
    let giant = t.graveyard(P0, "Hill Giant");
    gain_level(&mut t, P0, class, 3).unwrap();
    assert_eq!(level(&t, class), 2);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(level(&t, class), 3);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn gaining_a_level_keeps_the_abilities_of_the_previous_levels() {
    cr!("716.2a");
    ruling!(
        "Cleric Class",
        "Gaining a level won't remove abilities that a Class had at a previous level."
    );
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Cleric Class");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Plains", 9);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve();
    gain_level(&mut t, P0, class, 3).unwrap();
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(level(&t, class), 3);
    // Level 3's ability returned Hill Giant (toughness 3); the life gain was increased by
    // level 1's ability (3 + 1), and level 2's ability triggered from it.
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn a_level_ability_can_be_activated_only_from_the_previous_level() {
    cr!("716.2a");
    ruling!(
        "Wizard Class",
        "You can't activate the first level ability of a Class unless that Class is level 1. Similarly, you can't activate the second level ability of a Class unless that Class is level 2."
    );
    supported("Wizard Class");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Wizard Class");
    t.lands(P0, "Island", 20);
    // Level 1: not the second level ability.
    assert!(gain_level(&mut t, P0, class, 3).is_err());
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    // Level 2: not the first level ability.
    assert_eq!(level(&t, class), 2);
    assert!(gain_level(&mut t, P0, class, 2).is_err());
    gain_level(&mut t, P0, class, 3).unwrap();
    t.resolve_all();
    // Level 3: neither.
    assert_eq!(level(&t, class), 3);
    assert!(gain_level(&mut t, P0, class, 2).is_err());
    assert!(gain_level(&mut t, P0, class, 3).is_err());
}

#[test]
fn several_class_permanents_each_track_their_own_level() {
    cr!("716.2a", "716.2b");
    ruling!(
        "Caretaker's Talent",
        "There's no restriction on how many Class permanents you can control, whether they're the same or different classes. Each Class permanent tracks its own level separately."
    );
    supported("Caretaker's Talent");
    supported("Hunter's Talent");
    // Caretaker's Talent level 3: "Creature tokens you control get +2/+2."
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Caretaker's Talent");
    let second = t.battlefield(P0, "Caretaker's Talent");
    let hunter = t.battlefield(P0, "Hunter's Talent");
    let soldier = create_token(&mut t, P0, "Soldier");
    t.resolve_all();
    t.lands(P0, "Plains", 10);
    gain_level(&mut t, P0, first, 2).unwrap();
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    t.resolve_all();
    gain_level(&mut t, P0, first, 3).unwrap();
    t.resolve_all();
    assert_eq!(level(&t, first), 3);
    assert_eq!(level(&t, second), 1);
    assert_eq!(level(&t, hunter), 1);
    // Only the first one's level 3 ability applies.
    assert_eq!(t.pt(soldier), (3, 3));
    // The second one is still level 1: it can't gain level 3, only its first level.
    assert!(gain_level(&mut t, P0, second, 3).is_err());
    assert!(gain_level(&mut t, P0, second, 2).is_ok());
    t.resolve_all();
    assert_eq!((level(&t, first), level(&t, second)), (3, 2));
    assert_eq!(t.pt(soldier), (3, 3));
}

#[test]
fn a_player_can_multiclass_and_control_the_same_class_twice() {
    cr!("716.2a", "716.2b");
    ruling!(
        "Wizard Class",
        "You can multiclass or even control multiple Class enchantments of the same class. Each Class permanent tracks its own level separately."
    );
    // Wizard Class level 2: "When this Class becomes level 2, draw two cards."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Wizard Class");
    let b = t.battlefield(P0, "Wizard Class");
    let cleric = t.battlefield(P0, "Cleric Class");
    t.lands(P0, "Island", 6);
    let hand = t.hand_size(P0);
    gain_level(&mut t, P0, a, 2).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!((level(&t, a), level(&t, b), level(&t, cleric)), (2, 1, 1));
    // The other Wizard Class becomes level 2 on its own, and its ability triggers too.
    gain_level(&mut t, P0, b, 2).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
    assert_eq!((level(&t, a), level(&t, b), level(&t, cleric)), (2, 2, 1));
}

#[test]
fn each_talent_starts_with_its_first_ability_and_gains_the_next_as_it_levels_up() {
    cr!("716.2a", "716.3");
    ruling!(
        "Caretaker's Talent",
        "Each Class starts with only the first of its three class abilities. As the first level ability resolves, the Class becomes level 2 and gains the second class ability."
    );
    // Caretaker's Talent: "Whenever one or more tokens you control enter, draw a card.
    // This ability triggers only once each turn." Level 2: "When this Class becomes level
    // 2, create a token that's a copy of target token you control." Level 3: "Creature
    // tokens you control get +2/+2."
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Caretaker's Talent");
    let hand = t.hand_size(P0);
    let soldier = create_token(&mut t, P0, "Soldier");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(soldier), (1, 1));
    t.lands(P0, "Plains", 5);
    gain_level(&mut t, P0, class, 2).unwrap();
    // On the stack: still level 1, nothing copied yet.
    assert_eq!(level(&t, class), 1);
    assert_eq!(tokens(&t, P0).len(), 1);
    t.answer_targets(P0, &[Entity::Object(soldier)]);
    t.resolve();
    // It resolved: level 2, and "When this Class becomes level 2" triggered.
    assert_eq!(level(&t, class), 2);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(t.pt(soldier), (1, 1));
    gain_level(&mut t, P0, class, 3).unwrap();
    assert_eq!(t.pt(soldier), (1, 1));
    t.resolve_all();
    assert_eq!(level(&t, class), 3);
    for tok in tokens(&t, P0) {
        assert_eq!(t.pt(tok), (3, 3));
    }
}

#[test]
fn a_talent_has_three_class_abilities_and_two_level_abilities() {
    cr!("716.1", "716.2a");
    ruling!(
        "Artist's Talent",
        "Each Class has five abilities. The three in the major sections of its text box are class abilities. Class abilities can be static, activated, or triggered abilities. The other two are level abilities, one activated ability to advance the Class to level 2 and another to advance the Class to level 3."
    );
    supported("Artist's Talent");
    supported("Searing Spear");
    // Artist's Talent: "Whenever you cast a noncreature spell, you may discard a card. If
    // you do, draw a card." (triggered) Level 2: "Noncreature spells you cast cost {1}
    // less to cast." (static) Level 3: "If a source you control would deal noncombat
    // damage to an opponent or a permanent an opponent controls, it deals that much damage
    // plus 2 instead." (static)
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Artist's Talent");
    // The two level abilities are activated abilities advancing it to levels 2 and 3.
    assert_eq!(level_abilities(&mut t, class), vec![2, 3]);
    t.lands(P0, "Mountain", 6);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    gain_level(&mut t, P0, class, 3).unwrap();
    t.resolve_all();
    assert_eq!(level(&t, class), 3);
    assert_eq!(tapped_lands(&t, P0), 6);
    // At level 3 all three class abilities work: Searing Spear ({1}{R}) costs {R}, its
    // cast triggers the first ability, and it deals 3 + 2 damage.
    t.lands(P0, "Mountain", 1);
    let spear = t.hand(P0, "Searing Spear");
    t.cast(P0, spear).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the spell and the class trigger");
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Still only its two level abilities.
    assert_eq!(level_abilities(&mut t, class), vec![2, 3]);
}

#[test]
fn a_class_has_three_class_abilities_and_two_level_abilities() {
    cr!("716.1", "716.2a");
    ruling!(
        "Wizard Class",
        "Each Class has five abilities. The three in the major sections of its text box are class abilities. Class abilities can be static, activated, or triggered abilities. The other two are level abilities, one activated ability to advance the Class to level 2 and another to advance the Class to level 3."
    );
    // Wizard Class: "You have no maximum hand size." (static) Level 2: "When this Class
    // becomes level 2, draw two cards." (triggered) Level 3: "Whenever you draw a card,
    // put a +1/+1 counter on target creature you control." (triggered)
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Wizard Class");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(level_abilities(&mut t, class), vec![2, 3]);
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
    t.lands(P0, "Island", 8);
    let hand = t.hand_size(P0);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    gain_level(&mut t, P0, class, 3).unwrap();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.g.player(P0).max_hand_size, None);
    assert_eq!(level_abilities(&mut t, class), vec![2, 3]);
}

#[test]
fn two_barbarian_classes_roll_two_additional_dice_and_ignore_the_two_lowest() {
    cr!("706.6");
    ruling!(
        "Barbarian Class",
        "if you have multiple Barbarian Class cards, you roll that many additional dice and ignore that many of the lowest rolls"
    );
    supported("Barbarian Class");
    supported("Recruitment Drive");
    // Barbarian Class: "If you would roll one or more dice, instead roll that many dice
    // plus one and ignore the lowest roll." Recruitment Drive: "Roll a d20. 1—9 | Create
    // two 1/1 white Soldier creature tokens. 10—19 | Create two 2/2 white Knight creature
    // tokens. 20 | Create three 2/2 white Knight creature tokens."
    let drive = |classes: usize, loaded: &[u32]| {
        let mut t = TestGame::new(2);
        for _ in 0..classes {
            t.battlefield(P0, "Barbarian Class");
        }
        t.g.dice.loaded.extend(loaded.iter().copied());
        let spell = in_hand_with_mana(&mut t, P0, "Recruitment Drive");
        t.cast(P0, spell).go();
        t.resolve_all();
        let unused = t.g.dice.loaded.len();
        (
            with_subtype(&t, P0, "Soldier").len(),
            with_subtype(&t, P0, "Knight").len(),
            unused,
        )
    };
    // One Barbarian Class: two dice (3 and 5), the 3 is ignored: 5 counts. The third
    // loaded result isn't rolled.
    assert_eq!(drive(1, &[3, 5, 20]), (2, 0, 1));
    // Two: three dice, the two lowest ignored: the 20 counts.
    assert_eq!(drive(2, &[3, 5, 20]), (0, 3, 0));
    // Two, with the highest roll first: still the highest of the three.
    assert_eq!(drive(2, &[12, 5, 3]), (0, 2, 0));
}

#[test]
fn advancing_a_class_is_an_activated_ability_that_can_be_responded_to() {
    cr!("716.2a", "602.2");
    ruling!(
        "Leader's Talent",
        "An ability that advances a Class to a higher level is a normal activated ability. It uses the stack and can be responded to."
    );
    supported("Leader's Talent");
    // Leader's Talent level 2: "Whenever a creature you control leaves the battlefield,
    // if it had a counter on it, you gain 2 life."
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Leader's Talent");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
    t.lands(P0, "Plains", 3);
    gain_level(&mut t, P0, class, 2).unwrap();
    assert_eq!(t.stack_len(), 1);
    // In response, P1 bolts the Bears: the Class is still level 1, so no life.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(level(&t, class), 2);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn each_level_bar_is_an_activated_ability_with_the_listed_cost() {
    cr!("716.2a", "107.16");
    ruling!(
        "Leader's Talent",
        "Each represents an activated ability with the listed cost that advances the Class to the level listed in that bar."
    );
    // Leader's Talent: "{2}{W}: Level 2", "{3}{W}: Level 3".
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Leader's Talent");
    assert_eq!(level_abilities(&mut t, class), vec![2, 3]);
    t.lands(P0, "Plains", 2);
    assert!(gain_level(&mut t, P0, class, 2).is_err(), "{{2}}{{W}} needs three mana");
    t.lands(P0, "Plains", 1);
    gain_level(&mut t, P0, class, 2).unwrap();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(level(&t, class), 2);
    t.lands(P0, "Plains", 3);
    assert!(gain_level(&mut t, P0, class, 3).is_err(), "{{3}}{{W}} needs four mana");
    t.lands(P0, "Plains", 1);
    gain_level(&mut t, P0, class, 3).unwrap();
    assert_eq!(tapped_lands(&t, P0), 7);
    t.resolve_all();
    assert_eq!(level(&t, class), 3);
}

#[test]
fn a_class_has_its_top_section_and_each_section_up_to_its_level() {
    cr!("716.2a", "716.3", "603.10a");
    ruling!(
        "Leader's Talent",
        "Each Class starts at level 1 and has the abilities in the top section of its text box. Once a Class advances to level 2 or level 3, it has all of the abilities listed in the associated section of its text box as well."
    );
    // Leader's Talent: "Whenever you attack, put a +1/+1 counter on target attacking
    // creature." Level 2: "Whenever a creature you control leaves the battlefield, if it
    // had a counter on it, you gain 2 life." Level 3: "Whenever you cast a spell, put a
    // +1/+1 counter on each creature you control."
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Leader's Talent");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 7);
    // Level 1: attacking puts a counter on the target attacking creature.
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
    // A creature with a counter leaving: nothing at level 1.
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(other), counters::PLUS1, 1, None);
    destroy(&mut t, other);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Level 2: a creature with a counter leaving the battlefield (dying or not) gains 2
    // life; one without counters doesn't.
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    move_to(&mut t, bears, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Level 3: casting a spell puts a counter on each creature P0 controls.
    gain_level(&mut t, P0, class, 3).unwrap();
    t.resolve_all();
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    // The level 2 ability still works at level 3.
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn a_level_ability_cant_advance_a_class_to_its_level_or_lower() {
    cr!("716.2a");
    ruling!(
        "Cool but Rude",
        "You can't activate an ability that advances a Class to a particular level if that Class is already that level or higher."
    );
    supported("Cool but Rude");
    // Cool but Rude: "{1}{R}: Level 2" and "{1}{R}: Level 3 — When this Class becomes
    // level 3, search your library for a card, put it into your hand, shuffle, then
    // discard a card at random."
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Cool but Rude");
    t.lands(P0, "Mountain", 10);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    assert!(gain_level(&mut t, P0, class, 2).is_err());
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.hand(P0, "Shock");
    let hand = t.hand_size(P0);
    let library = t.library_size(P0);
    gain_level(&mut t, P0, class, 3).unwrap();
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.resolve_all();
    assert_eq!(level(&t, class), 3);
    // The found card went to the hand, then a card was discarded at random.
    assert_eq!(t.library_size(P0), library - 1);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(t.in_hand(P0, "Lightning Bolt") != t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_hand(P0, "Shock") != t.in_graveyard(P0, "Shock"));
    // Level 3: neither level ability can be activated.
    assert!(gain_level(&mut t, P0, class, 2).is_err());
    assert!(gain_level(&mut t, P0, class, 3).is_err());
}
