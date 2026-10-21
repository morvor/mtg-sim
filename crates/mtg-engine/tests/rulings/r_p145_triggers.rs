//! Rulings batch P145 — when the looting engines' triggered abilities trigger and resolve:
//! cast triggers resolve before the spell, intervening "if" clauses, "whenever you attack",
//! simultaneous events and look-back.

use crate::r_p145_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The cast trigger of `trigger_source` resolves before the spell, even though the spell
/// is countered with the trigger on the stack.
#[test]
fn cast_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Academy Wall",
        "An ability that triggers when a player casts a spell resolves before the spell that caused it to trigger. The ability resolves even if that spell is countered."
    );
    supported("Academy Wall");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Academy Wall");
    t.library_top(P0, "Memnite");
    let ritual = {
        yes(&mut t, P0);
        cast_new(&mut t, P0, "Dark Ritual", &[])
    };
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you may draw a card"), 1);
    counter(&mut t, ritual);
    t.resolve_all();
    assert!(t.in_hand(P0, "Memnite") || t.in_graveyard(P0, "Memnite"));
    assert_eq!(t.library_size(P0), 30, "drew Memnite");
    assert!(t.in_graveyard(P0, "Dark Ritual"));

    ruling!(
        "Mirrodin Besieged",
        "If you support the Mirrans, Mirrodin Besieged’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Mirrodin Besieged");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    enter(&mut t, P0, "Mirrodin Besieged");
    t.resolve_all();
    let thopter = cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    counter(&mut t, thopter);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Myr"), 1);
    assert!(t.in_graveyard(P0, "Ornithopter"));

    ruling!(
        "Kaito, Cunning Infiltrator",
        "The ability of the emblem created by Kaito's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    supported("Kaito, Cunning Infiltrator");
    let mut t = TestGame::new(2);
    let kaito = t.battlefield(P0, "Kaito, Cunning Infiltrator");
    t.g.objects[kaito.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 9);
    activate_containing(&mut t, P0, kaito, "emblem").expect("-9");
    t.resolve_all();
    // An opponent's spell triggers the emblem too ("a player casts").
    t.set_step(P1, Step::PrecombatMain);
    let ritual = cast_new(&mut t, P1, "Dark Ritual", &[]);
    t.settle();
    counter(&mut t, ritual);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Ninja"), 1);
}

fn tokens_with_subtype(t: &TestGame, p: PlayerId, sub: &str) -> usize {
    tokens(t, p)
        .into_iter()
        .filter(|id| t.g.obj(*id).chars.has_subtype(sub))
        .count()
}

#[test]
fn mirrodin_besieged_without_a_choice_has_neither_ability() {
    cr!("614.12", "607.2d");
    ruling!(
        "Mirrodin Besieged",
        "If you somehow control Mirrodin Besieged and no choice was made for it, it has neither of the two triggered abilities."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirrodin Besieged");
    t.library_top(P0, "Memnite");
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(stacked_triggers(&t), 0);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(stacked_triggers(&t), 0);
    assert_eq!(tokens(&t, P0).len(), 0);
}

/// P0 supports the Phyrexians, with `artifacts` artifact cards in the graveyard and
/// Ornithopter (an artifact card) as the only card in hand. Advances to P0's end step
/// (the trigger is put on the stack, targeting P1 unless P1 can't be targeted).
fn phyrexian(t: &mut TestGame, artifacts: usize) {
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    enter(t, P0, "Mirrodin Besieged");
    t.resolve_all();
    for _ in 0..artifacts {
        t.graveyard(P0, "Memnite");
    }
    t.hand(P0, "Ornithopter");
    t.library_top(P0, "Island");
    t.advance_to(P0, Step::End);
    t.settle();
}

#[test]
fn mirrodin_besieged_phyrexian_checks_on_resolution() {
    cr!("603.3d", "608.2b", "104.3h");
    ruling!(
        "Mirrodin Besieged",
        "If you support the Phyrexians, you must choose a target opponent as your end step begins. Whether that player loses the game is determined only as the ability resolves."
    );
    // Fourteen artifact cards: discarding Ornithopter (the fifteenth) makes P1 lose.
    let mut t = TestGame::new(2);
    phyrexian(&mut t, 14);
    assert_eq!(triggers_on_stack(&t, "fifteen"), 1);
    let thopter = t.g.player(P0).hand[0];
    discard_these(&mut t, P0, &[thopter]);
    t.resolve();
    assert!(t.has_lost(P1));
    // Thirteen: discarding the fourteenth isn't enough.
    let mut t = TestGame::new(2);
    phyrexian(&mut t, 13);
    let thopter = t.g.player(P0).hand[0];
    discard_these(&mut t, P0, &[thopter]);
    t.resolve();
    assert!(!t.has_lost(P1));
    assert_eq!(t.graveyard_size(P0), 14);
    // With no legal target opponent, the ability isn't put on the stack: no draw, no
    // discard.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Sanctity");
    phyrexian(&mut t, 14);
    assert_eq!(triggers_on_stack(&t, "fifteen"), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 14);
}

#[test]
fn mirrodin_besieged_two_headed_giant_team_loses() {
    cr!("810.8a", "104.3h");
    ruling!(
        "Mirrodin Besieged",
        "In a Two-Headed Giant game, if one player on a team loses the game, that team loses the game."
    );
    let mut t = crate::r_p108_common::two_headed_giant();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    enter(&mut t, P0, "Mirrodin Besieged");
    t.resolve_all();
    for _ in 0..15 {
        t.graveyard(P0, "Memnite");
    }
    t.library_top(P0, "Island");
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.has_lost(P2));
    assert!(t.has_lost(P3));
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
}

#[test]
fn canyon_crab_checks_hand_casts_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Canyon Crab",
        "At the beginning of your end step, Canyon Crab's ability will check to see if you've cast a spell from your hand this turn."
    );
    supported("Canyon Crab");
    // No spell cast: it triggers and loots.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Canyon Crab");
    t.library_top(P0, "Memnite");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card, then discard"), 1);
    // Casting a spell from hand in response: the ability does nothing.
    lands_for_cost(&mut t, P0, "Dark Ritual");
    let ritual = t.hand(P0, "Dark Ritual");
    t.cast_with(P0, ritual, &[]).expect("cast");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 31, "no draw");
    // A spell cast from hand earlier this turn: no trigger at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Canyon Crab");
    cast_new(&mut t, P0, "Dark Ritual", &[]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card, then discard"), 0);
    // A spell cast from somewhere else (the graveyard, with flashback) doesn't count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Canyon Crab");
    let twice = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 3);
    t.cast(P0, twice)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card, then discard"), 1);
}

const EMET: &str = "Emet-Selch, Unsundered // Hades, Sorcerer of Eld";

#[test]
fn emet_selch_needs_fourteen_cards_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Emet-Selch, Unsundered // Hades, Sorcerer of Eld",
        "Emet-Selch's last ability checks your graveyard at the moment it would trigger to see if you have fourteen or more cards in your graveyard."
    );
    supported(EMET);
    // Thirteen cards: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, EMET);
    fill_graveyard(&mut t, P0, 13);
    into_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "fourteen"), 0);
    // Fourteen: it triggers; it transforms if there are still fourteen on resolution.
    let mut t = TestGame::new(2);
    let emet = t.battlefield(P0, EMET);
    fill_graveyard(&mut t, P0, 14);
    into_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "fourteen"), 1);
    yes(&mut t, P0);
    t.resolve();
    assert_eq!(name_of(&t, emet), "Hades, Sorcerer of Eld");
    // Fourteen when it triggers, thirteen on resolution: nothing happens.
    let mut t = TestGame::new(2);
    let emet = t.battlefield(P0, EMET);
    fill_graveyard(&mut t, P0, 14);
    into_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "fourteen"), 1);
    let one = t.g.player(P0).graveyard[0];
    move_to(&mut t, one, Zone::Exile);
    yes(&mut t, P0);
    t.resolve();
    assert_eq!(name_of(&t, emet), "Emet-Selch, Unsundered");
}

#[test]
fn chakra_meditation_resolves_while_the_lesson_is_on_the_stack() {
    cr!("603.3", "608.2c");
    ruling!(
        "Chakra Meditation",
        "Notably, if you cast a Lesson spell while you have two Lesson cards in your graveyard, you will still have to discard a card to Chakra Meditation's ability because the Lesson you cast is still on the stack."
    );
    supported("Chakra Meditation");
    for (in_graveyard, discards) in [(2, 1), (3, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Chakra Meditation");
        for _ in 0..in_graveyard {
            t.graveyard(P0, "Octopus Form");
        }
        t.library_top(P0, "Memnite");
        let bears = t.battlefield(P0, "Grizzly Bears");
        cast_new(&mut t, P0, "Octopus Form", &[obj(bears)]);
        t.resolve();
        assert_eq!(t.stack_len(), 1, "the Lesson is still on the stack");
        assert_eq!(
            t.graveyard_size(P0),
            in_graveyard + discards,
            "{in_graveyard} Lessons"
        );
    }
}

#[test]
fn galadriel_checks_as_beginning_of_combat_starts() {
    cr!("603.4", "603.2");
    ruling!(
        "Galadriel, Elven-Queen",
        "If another Elf hasn't entered the battlefield under your control this turn as your beginning of combat step starts, Galadriel, Elven-Queen's ability won't trigger."
    );
    supported("Galadriel, Elven-Queen");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galadriel, Elven-Queen");
    into_beginning_of_combat(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "votes"), 0);
    // An Elf entering during the beginning of combat step doesn't trigger it.
    enter(&mut t, P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(triggers_on_stack(&t, "votes"), 0);
    // An Elf that entered in the main phase: it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galadriel, Elven-Queen");
    enter(&mut t, P0, "Llanowar Elves");
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "votes"), 1);
}

#[test]
fn murder_of_crows_sees_creatures_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Murder of Crows",
        "If another creature dies at the same time as Murder of Crows, its last ability triggers."
    );
    supported("Murder of Crows");
    let mut t = TestGame::new(2);
    let crows = t.battlefield(P0, "Murder of Crows");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    destroy_together(&mut t, &[crows, bears, elves]);
    assert_eq!(triggers_on_stack(&t, "you may draw a card"), 2);
}

#[test]
fn sage_of_the_falls_triggers_per_creature_and_resolves_one_at_a_time() {
    cr!("603.2c", "603.3b", "608.2");
    ruling!(
        "Sage of the Falls",
        "If more than one non-Human creature enters the battlefield at the same time, Sage of the Falls's ability triggers that many times. You'll resolve the abilities one at a time"
    );
    supported("Sage of the Falls");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sage of the Falls");
    t.library_top(P0, "Island");
    t.library_top(P0, "Swamp");
    // Two non-Humans and a Human (Elite Vanguard) enter at once: two triggers.
    enter_together(
        &mut t,
        P0,
        &["Grizzly Bears", "Llanowar Elves", "Elite Vanguard"],
    );
    assert_eq!(triggers_on_stack(&t, "you may draw a card"), 2);
    let hands = watch(&mut t, P0, is_discard, |g| g.player(P0).hand.len());
    yes(&mut t, P0);
    yes(&mut t, P0);
    t.resolve_all();
    // Each discard came after exactly one draw.
    assert_eq!(*hands.lock().unwrap(), vec![1, 1]);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn queen_of_dale_counts_spells_cast_before_she_entered() {
    cr!("603.2");
    ruling!(
        "The Queen of Dale",
        "If a noncreature spell was already cast by an opponent the turn The Queen of Dale enters, that opponent already cast their first noncreature spell this turn"
    );
    supported("The Queen of Dale");
    for cast_before in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        if cast_before {
            cast_new(&mut t, P1, "Dark Ritual", &[]);
            t.resolve_all();
        }
        t.battlefield(P0, "The Queen of Dale");
        cast_new(&mut t, P1, "Dark Ritual", &[]);
        t.settle();
        assert_eq!(
            triggers_on_stack(&t, "recruit"),
            usize::from(!cast_before),
            "cast before: {cast_before}"
        );
    }
}

#[test]
fn ledger_shredder_counts_spells_cast_before_it_entered() {
    cr!("603.2", "702.163a");
    ruling!(
        "Ledger Shredder",
        "Spells that were cast before Ledger Shredder entered the battlefield count. If Ledger Shredder was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Ledger Shredder");
    // A spell, then Ledger Shredder put onto the battlefield, then a second spell.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Dark Ritual", &[]);
    t.resolve_all();
    t.battlefield(P0, "Ledger Shredder");
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "connives"), 1);
    // Ledger Shredder as the first spell: the next one is the second.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Ledger Shredder", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "connives"), 1);
    t.resolve_all();
    // A third spell doesn't trigger it.
    cast_new(&mut t, P0, "Memnite", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "connives"), 0);
}

#[test]
fn whenever_you_attack_vehicles_need_not_attack() {
    cr!("508.3a", "603.2");
    ruling!(
        "Boosted Sloop",
        "Attacking with any creature will cause the second ability to trigger, even if this Vehicle isn’t attacking."
    );
    ruling!(
        "Great Gilded Boat",
        "Attacking with any creatures will cause Great Gilded Boat's ability to trigger. Great Gilded Boat doesn't have to be among them."
    );
    for (vehicle, text) in [
        ("Boosted Sloop", "draw a card, then discard"),
        ("Great Gilded Boat", "recruit"),
    ] {
        supported(vehicle);
        let mut t = TestGame::new(2);
        t.battlefield(P0, vehicle);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        assert_eq!(triggers_on_stack(&t, text), 1, "{vehicle}");
    }
}

/// A creature card that is both a Pirate and a Vehicle (no real card is).
fn pirate_vehicle() -> mtg_engine::card::CardDef {
    custom_card(
        "Pirate Skiff",
        "Artifact Creature — Pirate Vehicle",
        "{1}",
        Some((2, 2)),
        "",
    )
}

#[test]
fn fearless_swashbuckler_one_pirate_vehicle_is_enough() {
    cr!("603.4", "508.3a");
    ruling!(
        "Fearless Swashbuckler",
        "If you attack with a single creature that is both a Pirate and a Vehicle, the last ability triggers."
    );
    supported("Fearless Swashbuckler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fearless Swashbuckler");
    let skiff = t.custom(P0, pirate_vehicle(), Zone::Battlefield);
    attack_with(&mut t, &[(skiff, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "draw three cards"), 1);
    // A Pirate alone isn't enough.
    let mut t = TestGame::new(2);
    let swash = t.battlefield(P0, "Fearless Swashbuckler");
    attack_with(&mut t, &[(swash, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "draw three cards"), 0);
}

#[test]
fn mary_read_and_anne_bonny_trigger_once_per_card() {
    cr!("603.2c");
    ruling!(
        "Mary Read and Anne Bonny",
        "the ability will still trigger only once for that discarded card."
    );
    supported("Mary Read and Anne Bonny");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mary Read and Anne Bonny");
    let skiff = t.custom(P0, pirate_vehicle(), Zone::Hand(P0));
    discard(&mut t, P0, skiff);
    assert_eq!(triggers_on_stack(&t, "Treasure"), 1);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn varina_counts_the_zombies_attacked_with() {
    cr!("603.10", "508.3a", "506.4");
    ruling!(
        "Varina, Lich Queen",
        "Varina's first ability counts the number of Zombies you attacked with, even if some of those Zombies leave combat before the triggered ability resolves."
    );
    supported("Varina, Lich Queen");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Varina, Lich Queen");
    let a = t.battlefield(P0, "Walking Corpse");
    let b = t.battlefield(P0, "Walking Corpse");
    t.hand(P0, "Island");
    t.hand(P0, "Swamp");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "draw that many"), 1);
    destroy(&mut t, b);
    let life = t.life(P0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), life + 2);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), 28);
}

#[test]
fn wharf_infiltrator_any_creature_discard_and_one_payment() {
    cr!("603.2", "118.3");
    ruling!(
        "Wharf Infiltrator",
        "Wharf Infiltrator's last ability triggers anytime you discard a creature card"
    );
    ruling!(
        "Wharf Infiltrator",
        "While resolving Wharf Infiltrator's last ability, you can't pay {2} multiple times to get multiple Eldrazi Horror tokens."
    );
    supported("Wharf Infiltrator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wharf Infiltrator");
    t.lands(P0, "Wastes", 4);
    let bears = t.hand(P0, "Grizzly Bears");
    discard(&mut t, P0, bears);
    assert_eq!(triggers_on_stack(&t, "Eldrazi Horror"), 1);
    yes(&mut t, P0);
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Horror"), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
}

#[test]
fn the_locust_god_only_sees_draws() {
    cr!("121.1", "603.2");
    ruling!(
        "The Locust God",
        "If a spell or ability causes you to put cards into your hand without specifically using the word “draw,” The Locust God's first triggered ability won't trigger."
    );
    supported("The Locust God");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Locust God");
    cast_new(&mut t, P0, "Impulse", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(tokens(&t, P0).len(), 0);
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn katara_experience_counters_are_the_players() {
    cr!("122.1", "122.1d");
    ruling!(
        "Katara, Waterbending Master",
        "All experience counters are identical, no matter how you got them."
    );
    ruling!(
        "Katara, Waterbending Master",
        "The experience counter goes on you, the player, not on Katara. You will keep that counter even if Katara, Waterbending Master dies."
    );
    supported("Katara, Waterbending Master");
    let mut t = TestGame::new(2);
    let katara = t.battlefield(P0, "Katara, Waterbending Master");
    // A spell during an opponent's turn: an experience counter for P0.
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P0, "Dark Ritual", &[]);
    t.resolve_all();
    assert_eq!(counters_on_player(&t, P0, counters::EXPERIENCE), 1);
    assert_eq!(t.counters(katara, counters::EXPERIENCE), 0);
    // Two more from elsewhere; Katara counts all three as she attacks.
    player_counters(&mut t, P0, counters::EXPERIENCE, 2);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(katara, Entity::Player(P1))]);
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 27);
    assert_eq!(t.hand_size(P0), 2);
    // Katara dies: the counters stay.
    destroy(&mut t, katara);
    assert_eq!(counters_on_player(&t, P0, counters::EXPERIENCE), 3);
}

#[test]
fn coralhelm_chronicler_multikicker_triggers_once() {
    cr!("702.33c", "702.33d", "603.2");
    ruling!(
        "Coralhelm Chronicler",
        "If you cast a spell with multikicker, the first ability will trigger only once, no matter how many times you kicked the spell."
    );
    supported("Coralhelm Chronicler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Coralhelm Chronicler");
    t.lands(P0, "Wastes", 6);
    let chalice = t.hand(P0, "Everflowing Chalice");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.cast(P0, chalice).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "kicked spell"), 1);
    t.resolve_all();
    let chalice = t.named_on_battlefield("Everflowing Chalice")[0];
    assert_eq!(t.counters(chalice, "charge"), 2, "kicked twice");
}
