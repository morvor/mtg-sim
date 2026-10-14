//! Rulings batch P133 — individual bounce cards: costs and mana value (Coastal Breach,
//! Johann's Stopgap, Monk Class, Gust of Wind, Crashing Tide), Jin-Gitaxias // The Great
//! Synthesis, Fumble, copies (Mystic Confluence, Divide by Zero, String of
//! Disappearances), Time of Ice, Banishing Knack, Galecaster Colossus, Bounce Chamber.

use crate::r_p133_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_activate, can_cast};
use crate::r_s17_common::{enter_transformed, name_of, transform};
use crate::r_s19_common::add_lore;
use crate::r_s22_common::{check_free_cast_costs, FreeCaster};
use crate::r_s26_common::{modes_on_stack, spell_copies};
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn cost_reductions_dont_change_mana_value() {
    cr!("202.3", "601.2f", "702.125a");
    ruling!(
        "Coastal Breach",
        "Effects that reduce what you pay to cast a spell don’t affect its mana value."
    );
    ruling!(
        "Johann's Stopgap",
        "The cost-reduction ability of Johann's Stopgap doesn't change its mana cost or mana value, only the total cost you pay. Specifically, the mana value of Johann's Stopgap is always 4."
    );
    supported("Coastal Breach");
    supported("Johann's Stopgap");
    // Coastal Breach with undaunted in a two-player game: {5}{U} paid, mana value 7.
    let mut t = TestGame::new(2);
    let breach = t.hand(P0, "Coastal Breach");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 5);
    let s = t.cast(P0, breach).go();
    assert_eq!(t.g.mana_value_of(s), 7);
    // Johann's Stopgap bargained: {1}{U} paid, mana value 4.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let stopgap = t.hand(P0, "Johann's Stopgap");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    let s = t.cast(P0, stopgap).target(bears).go();
    assert!(!t.on_battlefield(thopter));
    assert_eq!(t.g.mana_value_of(s), 4);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
}

#[test]
fn gust_of_wind_reduction_locked_in_once_cast() {
    cr!("601.2f", "601.2h", "601.2i");
    ruling!(
        "Gust of Wind",
        "Gust of Wind is unaffected if you no longer control a creature with flying after you've cast it."
    );
    ruling!(
        "Gust of Wind",
        "Once you announce that you're casting Gust of Wind, no player may take actions until the spell has been paid for. Notably, opponents can't try to change whether you control a creature with flying."
    );
    supported("Gust of Wind");
    let mut t = TestGame::new(2);
    let crow = t.battlefield(P0, "Storm Crow");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gust = t.hand(P0, "Gust of Wind");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let from = t.asked().len();
    t.cast(P0, gust).target(bears).go();
    // P1 wasn't given priority while it was cast.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, decision::Decision::Priority { .. })));
    kill(&mut t, crow);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn crashing_tide_loses_flash_after_being_cast() {
    cr!("601.2", "702.8a", "608.2");
    ruling!(
        "Crashing Tide",
        "Once you announce that you’re casting Crashing Tide, players can’t try to remove your Merfolk to make it lose flash until you’re done casting it. If it loses flash after it’s been cast, it will still resolve if able."
    );
    supported("Crashing Tide");
    let mut t = TestGame::new(2);
    let merfolk = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let tide = spell(&mut t, P0, "Crashing Tide");
    t.cast(P0, tide).target(bears).go();
    kill(&mut t, merfolk);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn monk_class_second_spell_reduction() {
    cr!("601.2f", "716.2a");
    ruling!(
        "Monk Class",
        "If Monk Class is the first spell you cast, the next spell is the second spell and costs {1} less. If Monk Class is the second (or later) spell you cast, its bonus won't apply until the next turn."
    );
    ruling!(
        "Monk Class",
        "If you have more than one Monk Class on the battlefield, their abilities are cumulative. For example, if you control two Monk Classes, the second spell you cast each turn costs {2} less to cast."
    );
    // Monk Class as the first spell: Grizzly Bears ({1}{G}) costs {G}.
    let mut t = TestGame::new(2);
    let monk = spell(&mut t, P0, "Monk Class");
    t.cast(P0, monk).go();
    t.resolve_all();
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Monk Class as the second spell: the third spell isn't reduced.
    let mut t = TestGame::new(2);
    let shock = spell(&mut t, P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    let monk = spell(&mut t, P0, "Monk Class");
    t.cast(P0, monk).go();
    t.resolve_all();
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Two Monk Classes: the second spell, Hill Giant ({3}{R}), costs {1}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Monk Class");
    t.battlefield(P0, "Monk Class");
    let shock = spell(&mut t, P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn mystic_confluence_copy_keeps_modes() {
    cr!("707.10", "707.10c", "700.2g");
    ruling!(
        "Mystic Confluence",
        "If Mystic Confluence is copied, the effect that creates the copy will usually allow you to choose new targets, but you can't choose new modes."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let conf = spell(&mut t, P0, "Mystic Confluence");
    let s = t.cast(P0, conf).modes(&[1, 2, 2]).target(a).go();
    let twin = spell(&mut t, P0, "Twincast");
    t.cast(P0, twin).target(s).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(modes_on_stack(&t, copies[0]), vec![1, 2, 2]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(in_hand_of(&t, a, P1) && in_hand_of(&t, b, P1));
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn mystic_confluence_triggers_wait_until_it_resolved() {
    cr!("608.2", "603.3");
    ruling!(
        "Mystic Confluence",
        "No player can cast spells or activate abilities in between the modes of a resolving spell. Any abilities that trigger won't be put onto the stack until Mystic Confluence is done resolving."
    );
    // Psychosis Crawler: "Whenever you draw a card, each opponent loses 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychosis Crawler");
    let conf = spell(&mut t, P0, "Mystic Confluence");
    t.cast(P0, conf).modes(&[2, 2, 2]).go();
    let from = t.asked().len();
    t.g.resolve_top();
    // Nothing was asked while it resolved, and its triggers wait.
    assert!(t.asked()[from..].is_empty());
    assert!(t.in_graveyard(P0, "Mystic Confluence"));
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn divide_by_zero_copy_returned_to_hand_ceases_to_exist() {
    cr!("707.10a", "704.5e");
    ruling!(
        "Divide by Zero",
        "If an effect returns a copy of a spell to its owner’s hand, the copy ceases to exist."
    );
    supported("Divide by Zero");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let shock = spell(&mut t, P1, "Shock");
    let s = t.cast(P1, shock).target(Entity::Player(P0)).go();
    let twin = spell(&mut t, P1, "Twincast");
    t.cast(P1, twin).target(s).go();
    t.answer_yes(P1, false);
    t.resolve();
    let copy = spell_copies(&t)[0];
    let hand = t.hand_size(P1);
    let dbz = spell(&mut t, P0, "Divide by Zero");
    t.cast(P0, dbz).target(copy).go();
    t.resolve();
    assert!(spell_copies(&t).is_empty());
    assert_eq!(t.hand_size(P1), hand);
    t.resolve_all();
    // Only the original Shock dealt damage.
    assert_eq!(t.life(P0), 18);
}

#[test]
fn string_of_disappearances_copies() {
    cr!("707.10", "707.12", "603.2");
    ruling!(
        "String of Disappearances",
        "The copies String of Disappearances creates are created on the stack, so they’re not cast. Abilities that trigger when a player casts a spell won’t trigger."
    );
    ruling!(
        "String of Disappearances",
        "The copy of String of Disappearances has the ability to copy itself. The string can go on or not, as long as players are able and willing to keep paying {U}{U}."
    );
    supported("String of Disappearances");
    let mut t = TestGame::new(2);
    // Scalding Viper: "Whenever an opponent casts a spell with mana value 3 or less, this
    // creature deals 1 damage to that player."
    t.battlefield(P0, "Scalding Viper // Steam Clean");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Island", 2);
    let string = spell(&mut t, P0, "String of Disappearances");
    t.lands(P0, "Island", 2);
    t.cast(P0, string).target(a).go();
    // P1 pays and copies it at P0's Bears; P0 pays and copies the copy at P1's Giant;
    // P1 can't pay again.
    for (p, target) in [(P1, b), (P0, c)] {
        // Pay {U}{U}, copy it, choose a new target.
        t.answer_yes(p, true);
        t.answer_yes(p, true);
        t.answer_yes(p, true);
        t.answer_targets(p, &[Entity::Object(target)]);
    }
    t.resolve_all();
    assert!(in_hand_of(&t, a, P1));
    assert!(in_hand_of(&t, b, P0));
    assert!(in_hand_of(&t, c, P1));
    assert_eq!(crate::r_s11_common::spells_copied(&t), 2);
    // The copies weren't cast: Scalding Viper never triggered.
    assert_eq!(t.life(P1), 20);
}

#[test]
fn great_synthesis_chapter_one() {
    cr!("714.2c", "402.2", "608.2h");
    ruling!(
        "Jin-Gitaxias // The Great Synthesis",
        "For chapter I of The Great Synthesis, the number of cards you draw is determined as the ability resolves. If you no longer control The Great Synthesis at that time, your maximum hand size remains unchanged."
    );
    supported("Jin-Gitaxias // The Great Synthesis");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    let saga = enter_transformed(&mut t, P0, "Jin-Gitaxias // The Great Synthesis");
    assert_eq!(t.stack_len(), 1);
    // Two more cards before it resolves, and P1 gains control of the Saga.
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    crate::r_s06_common::give_control(&mut t, saga, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 10);
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, Some(7));
    // Still controlling it: no maximum hand size.
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, "Jin-Gitaxias // The Great Synthesis");
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
}

#[test]
fn jin_gitaxias_transform_ability_resolves_with_fewer_cards() {
    cr!("602.5b", "608.2");
    ruling!(
        "Jin-Gitaxias // The Great Synthesis",
        "Once you’ve activated Jin-Gitaxias’s last ability, it doesn’t matter what happens to the number of cards in your hand. The ability will resolve even if you have six or fewer cards in your hand by that time."
    );
    let mut t = TestGame::new(2);
    let jin = t.battlefield(P0, "Jin-Gitaxias // The Great Synthesis");
    let cards: Vec<ObjectId> = (0..7).map(|_| t.hand(P0, "Island")).collect();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    t.activate(P0, jin, 0, &[]).unwrap();
    for c in &cards[..3] {
        t.g.discard(P0, *c, None);
    }
    t.settle();
    assert_eq!(t.hand_size(P0), 4);
    t.resolve();
    let now = t.named_on_battlefield("The Great Synthesis");
    assert_eq!(now.len(), 1);
    assert_eq!(name_of(&t, now[0]), "The Great Synthesis");
}

/// P0's The Great Synthesis reaches chapter III (no chapter I or II this time), casting
/// `card` from P0's hand.
fn synthesis_chapter_three(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let jin = t.battlefield(P0, "Jin-Gitaxias // The Great Synthesis");
    transform(t, jin);
    let saga = t.g.current(jin);
    t.g.objects[saga.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::LORE.into(), 2);
    t.answer_choose(P0, &[Entity::Object(card)]);
    answers(t);
    add_lore(t, saga, 1);
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn great_synthesis_free_casts_pay_additional_costs() {
    cr!("118.9", "118.9a", "714.2c");
    ruling!(
        "Jin-Gitaxias // The Great Synthesis",
        "If you cast a spell “without paying its mana cost,” you can’t pay any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast it."
    );
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: |t, name| t.hand(P0, name),
        run: synthesis_chapter_three,
    });
}

#[test]
fn fumble_attaches_to_an_untargeted_creature() {
    cr!("701.3a", "303.4", "301.5");
    ruling!(
        "Fumble",
        "Fumble doesn’t target the creature that it attaches Auras and Equipment to. You can attach Auras and Equipment to creatures you can’t target, such as an opponent’s creature with hexproof."
    );
    ruling!(
        "Fumble",
        "The Auras and Equipment must be attached to a creature if possible, even if the only creature they can all be attached to is an opponent’s creature."
    );
    supported("Fumble");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let sword = crate::r_s06_common::attach_new(&mut t, P1, "Short Sword", bears);
    let aura = crate::r_s06_common::attach_new(&mut t, P1, "Holy Strength", bears);
    let fumble = spell(&mut t, P0, "Fumble");
    t.cast(P0, fumble).target(bears).go();
    t.answer_choose(P0, &[Entity::Object(scout)]);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
    for x in [sword, aura] {
        assert_eq!(t.obj_now(x).controller, P0);
        assert_eq!(t.obj_now(x).attached_to, Some(Entity::Object(scout)));
    }
}

#[test]
fn fumble_cant_attach_them_all_to_one_creature() {
    cr!("701.3b", "704.5m");
    ruling!(
        "Fumble",
        "The Auras and Equipment that you gain control of must all be attached to one creature that they can all legally be attached to. If they can’t all legally be attached to one creature, they all remain on the battlefield unattached (and Auras among them will be put into their owners’ graveyards)."
    );
    // The only other creature has protection from black: Unholy Strength can't enchant
    // it, so neither is attached.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let knight = t.battlefield(P1, "White Knight");
    let sword = crate::r_s06_common::attach_new(&mut t, P1, "Short Sword", bears);
    let aura = crate::r_s06_common::attach_new(&mut t, P1, "Unholy Strength", bears);
    let fumble = spell(&mut t, P0, "Fumble");
    t.cast(P0, fumble).target(bears).go();
    t.answer_choose(P0, &[Entity::Object(knight)]);
    t.resolve_all();
    assert!(t.on_battlefield(sword));
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert_eq!(t.obj_now(sword).controller, P0);
    assert_eq!(t.zone(aura), Zone::Graveyard(P1));
}

#[test]
fn time_of_ice_chapters() {
    cr!("714.2", "611.2b", "502.3");
    ruling!(
        "Time of Ice",
        "The effects of Time of Ice's first two chapter abilities expire if you lose control of it, even if you immediately regain control of it or cast another Time of Ice."
    );
    ruling!(
        "Time of Ice",
        "The effect of Time of Ice's final chapter ability returns creatures that are tapped for any reason, not just those tapped by Time of Ice."
    );
    supported("Time of Ice");
    // Losing and regaining control ends chapter I's effect.
    for lose_control in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        let saga = t.enter(P0, "Time of Ice");
        t.resolve_all();
        assert!(t.obj_now(bears).tapped);
        if lose_control {
            crate::r_s06_common::give_control(&mut t, saga, P1);
            crate::r_s06_common::give_control(&mut t, saga, P0);
        }
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.obj_now(bears).tapped, !lose_control);
    }
    // Chapter III: P0's own creature tapped by attacking is returned too.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    t.g.objects[mine.0 as usize].tapped = true;
    let untapped = t.battlefield(P1, "Grizzly Bears");
    let saga = t.battlefield(P0, "Time of Ice");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::LORE.into(), 2);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert!(in_hand_of(&t, mine, P0));
    assert!(t.on_battlefield(untapped));
}

#[test]
fn banishing_knack_new_ability_belongs_to_the_creatures_controller() {
    cr!("602.2", "113.8");
    ruling!(
        "Banishing Knack",
        "The targeted creature’s controller, not Banishing Knack’s controller, is the one who can activate the creature’s new ability."
    );
    supported("Banishing Knack");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P0, "Ornithopter");
    let knack = spell(&mut t, P0, "Banishing Knack");
    t.cast(P0, knack).target(bears).go();
    t.resolve_all();
    assert!(!can_activate(&mut t, P0, bears));
    assert!(can_activate(&mut t, P1, bears));
    t.activate(P1, bears, 0, &[Entity::Object(thopter)]).unwrap();
    t.resolve_all();
    assert!(in_hand_of(&t, thopter, P0));
}

#[test]
fn galecaster_colossus_taps_any_untapped_wizard() {
    cr!("302.6", "118.3");
    ruling!(
        "Galecaster Colossus",
        "To activate Galecaster Colossus’s ability, you may tap any untapped Wizard you control, including one you haven’t controlled continuously since the beginning of your most recent turn, such as itself. (Note that tapping the creature doesn’t use {T} [the tap symbol].)"
    );
    supported("Galecaster Colossus");
    let mut t = TestGame::new(2);
    let colossus = t.battlefield_sick(P0, "Galecaster Colossus");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(colossus)]);
    t.activate(P0, colossus, 0, &[Entity::Object(bears)])
        .unwrap();
    assert!(t.obj(colossus).tapped);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
}

#[test]
fn bounce_chamber_lowest_toughness_on_resolution() {
    cr!("717.5", "701.52a", "115.10");
    ruling!(
        "Bounce Chamber",
        "Bounce Chamber doesn’t target any creature. Whichever creature has the least toughness as the ability resolves is the one that gets bounced. If there’s a tie, you choose one of them at that time."
    );
    supported("Bounce Chamber");
    let mut t = TestGame::new(2);
    let chamber = t.battlefield(P0, "Bounce Chamber");
    let lit = t.obj(chamber).card.as_ref().unwrap().attraction_lights[0];
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    crate::r_s35_common::load_dice(&mut t, &[lit]);
    mtg_engine::variants::roll_to_visit(&mut t.g, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A Memnite enters before it resolves, tied with the hexproof Gladecover Scout as the
    // creature with the least toughness: P0 chooses between them as it resolves.
    let memnite = t.battlefield(P1, "Memnite");
    t.answer_choose(P0, &[Entity::Object(memnite)]);
    t.resolve_all();
    assert!(in_hand_of(&t, memnite, P1));
    assert!(t.on_battlefield(scout) && t.on_battlefield(bears));
}
