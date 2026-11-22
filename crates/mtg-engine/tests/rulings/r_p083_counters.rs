//! Rulings batch P083 — counterspell hate: spells that can't be countered (CR 101.2,
//! 701.6), Banefire's conditional "can't be countered" (CR 107.3), Glen Elendra's Answer
//! countering spells and abilities (CR 113.3, 701.6b), Gaea's Revenge's targeting
//! restriction (CR 115, 113.6), City of Solitude's activation restriction (CR 602.1),
//! ward against spells that can't be countered (CR 702.21), Defense Grid in Two-Headed
//! Giant (CR 805.4), and divided damage (CR 601.2d, 608.2b).

use crate::r_p083_common::*;
use crate::r_s01_common::{attack_with, tokens};
use crate::r_s04_common::{crew, is_creature, untapped_lands};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------
// Spells that can't be countered, against counterspells.
// ---------------------------------------------------------------------------------

/// P0 casts the uncounterable spell `name` (with `targets`); P1 answers with Dismiss
/// ("Counter target spell. Draw a card."): Dismiss resolves and P1 draws, but the spell
/// stays on the stack. Then the spell resolves.
fn dismiss_fails(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    let spell = if name == "Mistcutter Hydra" {
        // X = 2.
        t.lands(P0, "Forest", 3);
        let c = t.hand(P0, name);
        t.cast(P0, c).x(2).go()
    } else {
        cast_new(t, P0, name, targets)
    };
    let hand = t.hand_size(P1);
    counter_with(t, P1, "Dismiss", spell);
    assert!(on_stack(t, spell), "{name} was countered");
    assert!(t.in_graveyard(P1, "Dismiss"), "Dismiss resolved");
    assert_eq!(t.hand_size(P1), hand + 1, "Dismiss's draw still happens");
    t.resolve_all();
    spell
}

#[test]
fn uncounterable_spells_are_targeted_but_not_countered() {
    cr!("101.2", "608.2c", "701.6a");
    ruling!(
        "Kavu Chameleon",
        "Counterspells can be cast that target it, but when they resolve they simply don't counter it since it can't be countered."
    );
    ruling!(
        "Scragnoth",
        "Side-effects of spells that would counter this card will still happen, because they happen even if the spell they’re trying to counter can’t be countered."
    );
    ruling!(
        "Mistcutter Hydra",
        "Spells that can't be countered can still be chosen as the target of spells or abilities that try to counter them. Although the spell won't be countered, the spell or ability that tries to counter it will resolve, and any other effects of that spell or ability will happen."
    );
    ruling!(
        "Thought Distortion",
        "A spell or ability that counters spells can still target Thought Distortion. When that spell or ability resolves, Thought Distortion won’t be countered, but any additional effects of that spell or ability will still happen."
    );
    // Creature spells: they resolve and enter. Scragnoth and Mistcutter Hydra have
    // protection from blue, which doesn't work on the stack (CR 113.6), so the blue
    // Dismiss can target them.
    for name in ["Kavu Chameleon", "Scragnoth", "Mistcutter Hydra"] {
        let mut t = TestGame::new(2);
        dismiss_fails(&mut t, name, &[]);
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
    }
    // A sorcery: it resolves and does what it says.
    let mut t = TestGame::new(2);
    let ring = t.hand(P1, "Sol Ring");
    t.g.turn.active = P0;
    dismiss_fails(&mut t, "Thought Distortion", &[player(P1)]);
    assert!(t.in_exile("Sol Ring"), "Thought Distortion resolved");
    let _ = ring;
}

#[test]
fn terra_stomper_survives_cancel() {
    cr!("101.2", "701.6a");
    ruling!(
        "Terra Stomper",
        "Terra Stomper can be targeted by spells that try to counter it (such as Cancel). Those spells will resolve, but the part of their effect that would counter Terra Stomper won’t do anything. Any other effects those spells have will work as normal."
    );
    supported("Terra Stomper");
    let mut t = TestGame::new(2);
    let stomper = cast_new(&mut t, P0, "Terra Stomper", &[]);
    counter_with(&mut t, P1, "Cancel", stomper);
    assert!(t.in_graveyard(P1, "Cancel"));
    assert!(on_stack(&t, stomper));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Terra Stomper").len(), 1);
}

#[test]
fn scragnoth_remand_draws_but_doesnt_return_it() {
    cr!("101.2", "608.2c", "701.6a");
    ruling!(
        "Scragnoth",
        "Spells with a side effect that only happens if a spell is countered do not have their side effect happen when the countering fails."
    );
    supported("Scragnoth");
    supported("Remand");
    let mut t = TestGame::new(2);
    let scrag = cast_new(&mut t, P0, "Scragnoth", &[]);
    let hand0 = t.hand_size(P0);
    let hand1 = t.hand_size(P1);
    // Remand: "Counter target spell. If that spell is countered this way, put it into
    // its owner's hand instead of into that player's graveyard. Draw a card."
    counter_with(&mut t, P1, "Remand", scrag);
    assert!(on_stack(&t, scrag), "not countered");
    assert_eq!(t.hand_size(P0), hand0, "not returned to its owner's hand");
    assert_eq!(t.hand_size(P1), hand1 + 1, "the unconditional draw happens");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Scragnoth").len(), 1);
}

#[test]
fn scragnoth_spell_can_be_bounced_from_the_stack() {
    cr!("101.2", "113.6", "701.6a");
    ruling!(
        "Scragnoth",
        "Spells can target Scragnoth and affect it while it is on the stack. Only effects that counter spells are ignored."
    );
    supported("Unsubstantiate");
    // Unsubstantiate (blue, "Return target spell or creature to its owner's hand"):
    // protection from blue doesn't work on the stack, and returning a spell isn't
    // countering it.
    let mut t = TestGame::new(2);
    let scrag = cast_new(&mut t, P0, "Scragnoth", &[]);
    cast_new(&mut t, P1, "Unsubstantiate", &[obj(scrag)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Scragnoth"));
    assert!(t.named_on_battlefield("Scragnoth").is_empty());
    // On the battlefield, protection from blue stops Unsubstantiate targeting it.
    let s = t.battlefield(P0, "Scragnoth");
    let targets = spell_targets(&mut t, P1, "Unsubstantiate");
    assert!(!targets.contains(&obj(s)));
}

// ---------------------------------------------------------------------------------
// Banefire.
// ---------------------------------------------------------------------------------

/// P0 casts Banefire with X = `x` at P1 (paying with `lands` Mountains).
fn banefire(t: &mut TestGame, x: i64, lands: usize) -> ObjectId {
    supported("Banefire");
    t.lands(P0, "Mountain", lands);
    let bf = t.hand(P0, "Banefire");
    t.answer_targets(P0, &[player(P1)]);
    t.cast(P0, bf).x(x).go()
}

#[test]
fn banefire_with_x_five_is_targeted_but_not_countered() {
    cr!("101.2", "107.3", "701.6a");
    ruling!(
        "Banefire",
        "Banefire can be targeted by spells that try to counter it regardless of what the value of X is. If X is 5 or more, those spells will still resolve, but the part of their effect that would counter Banefire won’t do anything. Any other effects those spells have will work as normal."
    );
    for x in [4, 5, 6] {
        let mut t = TestGame::new(2);
        let bf = banefire(&mut t, x, x as usize + 1);
        let hand = t.hand_size(P1);
        counter_with(&mut t, P1, "Dismiss", bf);
        assert_eq!(t.hand_size(P1), hand + 1, "X = {x}: Dismiss draws");
        t.resolve_all();
        let dealt = if x >= 5 { x as i32 } else { 0 };
        assert_eq!(t.life(P1), 20 - dealt, "X = {x}");
    }
}

#[test]
fn banefire_checks_the_chosen_x() {
    cr!("107.3", "107.3a", "601.2f", "615.12");
    ruling!(
        "Banefire",
        "To determine whether it can be countered and its damage can be prevented, Banefire checks the number you chose for X, not the amount of mana you actually spent or the amount of damage it will actually deal."
    );
    supported("Goblin Electromancer");
    supported("Furnace of Rath");
    supported("Healing Salve");
    // X = 5 with Goblin Electromancer ("Instant and sorcery spells you cast cost {1}
    // less"): five mana spent, but X is 5, so Dismiss doesn't counter it and P1's Healing
    // Salve shield doesn't prevent its damage.
    let salve = |t: &mut TestGame| {
        t.lands(P1, "Plains", 1);
        let s = t.hand(P1, "Healing Salve");
        t.answer_targets(P1, &[player(P1)]);
        t.cast(P1, s).modes(&[1]).go();
        t.resolve_all();
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Electromancer");
    salve(&mut t);
    let bf = banefire(&mut t, 5, 5);
    assert!(
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == P0 && t.g.obj(**id).is(CardType::Land))
            .all(|id| t.g.obj(*id).tapped),
        "five lands paid for X = 5"
    );
    counter_with(&mut t, P1, "Dismiss", bf);
    t.resolve_all();
    assert_eq!(t.life(P1), 15, "not countered and not prevented");
    // X = 4 with Furnace of Rath (it would deal 8): it can be countered...
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furnace of Rath");
    let bf = banefire(&mut t, 4, 5);
    counter_with(&mut t, P1, "Dismiss", bf);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "countered");
    // ... and its damage can be prevented.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furnace of Rath");
    salve(&mut t);
    banefire(&mut t, 4, 5);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - (8 - 3), "3 of the 8 damage prevented");
}

#[test]
fn banefire_with_an_illegal_target_doesnt_resolve() {
    cr!("608.2b");
    ruling!(
        "Banefire",
        "If its target is illegal by the time Banefire tries to resolve, the spell isn’t countered, but it doesn’t resolve. The target won’t be dealt damage."
    );
    supported("Banefire");
    supported("Multani's Presence");
    for x in [3, 5] {
        let mut t = TestGame::new(2);
        // "Whenever a spell you've cast is countered, draw a card."
        t.battlefield(P0, "Multani's Presence");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Mountain", x as usize + 1);
        let bf = t.hand(P0, "Banefire");
        t.answer_targets(P0, &[obj(bears)]);
        t.cast(P0, bf).x(x).go();
        // In response, the Bears gain shroud... simplest: they leave and come back as a
        // new object (CR 400.7).
        cast_resolve(&mut t, P1, "Unsummon", &[obj(bears)]);
        let again = t.battlefield(P1, "Grizzly Bears");
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Banefire"), "X = {x}");
        assert_eq!(
            t.hand_size(P0),
            hand,
            "X = {x}: not countered, so no card drawn"
        );
        assert_eq!(damage_on(&t, again), 0);
        assert_eq!(t.life(P1), 20);
        assert_eq!(t.life(P0), 20);
    }
}

// ---------------------------------------------------------------------------------
// Glen Elendra's Answer.
// ---------------------------------------------------------------------------------

fn faeries(t: &TestGame, p: PlayerId) -> usize {
    tokens(t, p)
        .iter()
        .filter(|id| t.obj_now(**id).chars.has_subtype("Faerie"))
        .count()
}

#[test]
fn glen_elendras_answer_counters_spells_and_both_kinds_of_abilities_already_on_the_stack() {
    cr!("113.3b", "113.3c", "602.1", "603.1", "701.6a", "701.6b");
    ruling!(
        "Glen Elendra's Answer",
        "Activated abilities are written in the form \"[Cost]: [Effect].\" Some keyword abilities, such as equip and crew, are activated abilities and will have colons in their reminder texts."
    );
    ruling!(
        "Glen Elendra's Answer",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities, such as mobilize (from the Tarkir: Dragonstorm release), are triggered abilities and will have \"when,\" \"whenever,\" or \"at\" in their reminder text."
    );
    ruling!(
        "Glen Elendra's Answer",
        "Glen Elendra's Answer counters only spells, activated abilities, and triggered abilities your opponents control that are already on the stack. It doesn't prevent spells from being cast or abilities from being activated or triggering later in the turn, and it has no effect on static abilities."
    );
    supported("Glen Elendra's Answer");
    supported("Nightblade Brigade");
    supported("Smuggler's Copter");
    supported("Glorious Anthem");
    let mut t = TestGame::new(2);
    let brigade = t.battlefield(P1, "Nightblade Brigade");
    let copter = t.battlefield(P1, "Smuggler's Copter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P1, "Glorious Anthem");
    t.set_step(P1, Step::BeginningOfCombat);
    // Mobilize 1 triggers (a keyword triggered ability).
    attack_with(&mut t, &[(brigade, player(P0))]);
    assert_eq!(t.stack_len(), 1, "the mobilize trigger");
    // Crew 1 (a keyword activated ability).
    assert!(crew(&mut t, P1, copter, &[bears]));
    // A spell.
    cast_new(&mut t, P1, "Lightning Bolt", &[player(P0)]);
    assert_eq!(t.stack_len(), 3);
    let p1_tokens = tokens(&t, P1).len();
    cast_new(&mut t, P0, "Glen Elendra's Answer", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 0, "all three countered");
    assert_eq!(faeries(&t, P0), 3, "a Faerie for each");
    assert_eq!(t.life(P0), 20, "the Bolt was countered");
    assert_eq!(tokens(&t, P1).len(), p1_tokens, "no Warrior token");
    assert!(!is_creature(&t, copter), "not crewed");
    assert!(t.obj_now(bears).tapped, "the crew cost isn't refunded");
    // Static abilities aren't affected: Glorious Anthem still pumps P1's creatures.
    assert_eq!(t.pt(bears), (3, 3));
    // Later this turn, P1 can still activate abilities and cast spells.
    assert!(crew(&mut t, P1, copter, &[elves]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
    cast_resolve(&mut t, P1, "Shock", &[player(P0)]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn glen_elendras_answer_leaves_its_controllers_own_spells_and_abilities() {
    cr!("701.6a", "109.5");
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[player(P1)]).unwrap();
    cast_new(&mut t, P0, "Shock", &[player(P1)]);
    cast_new(&mut t, P1, "Lightning Bolt", &[player(P0)]);
    cast_new(&mut t, P0, "Glen Elendra's Answer", &[]);
    t.resolve_all();
    assert_eq!(faeries(&t, P0), 1);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17, "P0's Shock and Pyromancer ping resolved");
}

#[test]
fn glen_elendras_answer_doesnt_touch_mana_abilities() {
    cr!("605.1a", "605.1b", "605.3b", "605.4a", "701.6a");
    ruling!(
        "Glen Elendra's Answer",
        "Glen Elendra's Answer can't target mana abilities. An activated mana ability is one that could add mana as it resolves, doesn't have a target, and isn't a loyalty ability. A triggered mana ability is one that could add mana as it resolves and triggers based on an activated mana ability."
    );
    supported("Wild Growth");
    // (The Answer has no target: its "can't target" is the wording of the same ruling on
    // targeted counters such as Tale's End. It holds trivially, and what it means here,
    // that mana abilities, which never use the stack, aren't countered, is tested.)
    // P1 taps a Forest enchanted with Wild Growth ("Whenever enchanted land is tapped for
    // mana, its controller adds an additional {G}", a triggered mana ability) while Glen
    // Elendra's Answer is on the stack: neither ability uses the stack, so nothing is
    // countered and P1 keeps the mana.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    crate::r_s06_common::attach_new(&mut t, P1, "Wild Growth", forest);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let gea = cast_new(&mut t, P0, "Glen Elendra's Answer", &[]);
    assert!(tap_for_mana(&mut t, P1, forest, ""));
    assert!(tap_for_mana(&mut t, P1, elves, ""));
    assert_eq!(t.stack_len(), 1, "only the Answer is on the stack");
    assert!(on_stack(&t, gea));
    t.resolve();
    assert_eq!(faeries(&t, P0), 0, "nothing countered");
    assert_eq!(t.g.player(P1).mana_pool.total(), 3);
}

#[test]
fn spells_countered_this_way_are_counted_wherever_they_go() {
    cr!("701.6a", "614.1a", "707.10a");
    // "For each spell and ability countered this way" (Glen Elendra's Answer) and "for
    // each spell countered this way" (Swift Silence) count every spell the instruction
    // countered: a copy of a spell (which then ceases to exist, CR 707.10a), and Darksteel
    // Colossus, which its own replacement effect shuffles into its owner's library
    // instead of putting it into a graveyard.
    supported("Darksteel Colossus");
    supported("Twincast");
    supported("Swift Silence");
    let setup = |t: &mut TestGame| {
        t.set_step(P1, Step::PrecombatMain);
        cast_new(t, P1, "Darksteel Colossus", &[]);
        let bolt = cast_new(t, P1, "Lightning Bolt", &[player(P0)]);
        cast_new(t, P1, "Twincast", &[obj(bolt)]);
        t.resolve();
        assert_eq!(t.stack_len(), 3, "the Colossus, the Bolt and its copy");
    };
    let mut t = TestGame::new(2);
    setup(&mut t);
    let library = t.library_size(P1);
    cast_new(&mut t, P0, "Glen Elendra's Answer", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(
        t.library_size(P1),
        library + 1,
        "the Colossus was shuffled in"
    );
    assert_eq!(faeries(&t, P0), 3);
    let mut t = TestGame::new(2);
    setup(&mut t);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Swift Silence", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand + 3);
}

// ---------------------------------------------------------------------------------
// Gaea's Revenge.
// ---------------------------------------------------------------------------------

#[test]
fn gaeas_revenge_abilities_work_in_their_zones() {
    cr!("101.2", "113.6", "113.6g");
    ruling!(
        "Gaea's Revenge",
        "Gaea's Revenge's first ability works only while it's a spell on the stack. Gaea's Revenge's last ability works only while it's on the battlefield."
    );
    supported("Gaea's Revenge");
    // On the stack: it can't be countered, but its last ability doesn't work yet, so the
    // blue Dismiss and Unsubstantiate can target the spell: Dismiss doesn't counter it, and
    // Unsubstantiate returns it to its owner's hand.
    let mut t = TestGame::new(2);
    let gr = cast_new(&mut t, P0, "Gaea's Revenge", &[]);
    counter_with(&mut t, P1, "Dismiss", gr);
    assert!(on_stack(&t, gr), "not countered");
    cast_new(&mut t, P1, "Unsubstantiate", &[obj(gr)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Gaea's Revenge"));
    // On the battlefield: Unsubstantiate can't target it.
    let gr = t.battlefield(P0, "Gaea's Revenge");
    assert!(!spell_targets(&mut t, P1, "Unsubstantiate").contains(&obj(gr)));
}

#[test]
fn gaeas_revenge_stops_your_own_colorless_equip() {
    cr!("115.1e", "702.6a", "105.2c");
    ruling!(
        "Gaea's Revenge",
        "Gaea's Revenge's last ability applies to all nongreen spells and abilities from nongreen sources, including ones you control. For example, you can't target it with the equip ability of an Equipment you control (unless the Equipment is somehow green)."
    );
    supported("Bonesplitter");
    supported("Painter's Servant");
    let mut t = TestGame::new(2);
    let gr = t.battlefield(P0, "Gaea's Revenge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bone = t.battlefield(P0, "Bonesplitter");
    let targets = ability_targets(&mut t, bone, 0);
    assert!(targets.contains(&obj(bears)));
    assert!(
        !targets.contains(&obj(gr)),
        "a colorless source's equip can't target it"
    );
    // Your own nongreen spell can't target it either; your green one can.
    assert!(spell_targets(&mut t, P0, "Giant Growth").contains(&obj(gr)));
    assert!(!spell_targets(&mut t, P0, "Shock").contains(&obj(gr)));
    // With Painter's Servant naming green, the Equipment is green: equip can target it.
    crate::r_p022_common::choose_color(&mut t, P0, Color::Green);
    t.enter(P0, "Painter's Servant");
    t.settle();
    assert!(ability_targets(&mut t, bone, 0).contains(&obj(gr)));
    t.lands(P0, "Wastes", 1);
    t.activate(P0, bone, 0, &[obj(gr)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bone).attached_to, Some(obj(gr)));
}

#[test]
fn gaeas_revenge_can_be_targeted_by_a_spell_that_is_green_and_another_color() {
    cr!("105.2", "115.1a");
    ruling!(
        "Gaea's Revenge",
        "If a spell is one or more colors, and one of those colors is green, that spell can target Gaea's Revenge."
    );
    supported("Ground Assault");
    let mut t = TestGame::new(2);
    let gr = t.battlefield(P1, "Gaea's Revenge");
    // Ground Assault ({R}{G}): red and green.
    assert!(spell_targets(&mut t, P0, "Ground Assault").contains(&obj(gr)));
    // Red alone: no.
    assert!(!spell_targets(&mut t, P0, "Lightning Bolt").contains(&obj(gr)));
    t.lands(P0, "Mountain", 2);
    cast_resolve(&mut t, P0, "Ground Assault", &[obj(gr)]);
    assert_eq!(damage_on(&t, gr), 4, "four lands: 4 damage");
}

// ---------------------------------------------------------------------------------
// Ward and spells that can't be countered.
// ---------------------------------------------------------------------------------

#[test]
fn ward_cost_may_be_paid_but_the_uncounterable_spell_resolves_anyway() {
    cr!("101.2", "702.21a", "701.6a");
    ruling!(
        "Abrupt Decay",
        "If you target a creature with ward, you may still pay the ward cost, but Abrupt Decay won't be countered even if you don't."
    );
    ruling!(
        "Slice from the Shadows",
        "If you target a creature with ward, you may still pay the ward cost, but Slice from the Shadows won't be countered even if you don't."
    );
    ruling!(
        "Void Rend",
        "If you target a creature with ward, you may still pay the ward cost, but Void Rend won't be countered even if you don't."
    );
    supported("Tomakul Honor Guard");
    for name in ["Abrupt Decay", "Slice from the Shadows", "Void Rend"] {
        supported(name);
        for pay in [false, true] {
            let mut t = TestGame::new(2);
            let guard = t.battlefield(P0, "Tomakul Honor Guard");
            // Two spare lands for the ward cost {2}.
            t.lands(P1, "Wastes", 2);
            crate::r_s25_common::lands_for_cost(&mut t, P1, name);
            if name == "Slice from the Shadows" {
                t.lands(P1, "Swamp", 1);
            }
            let card = t.hand(P1, name);
            t.answer_targets(P1, &[obj(guard)]);
            let mut b = t.cast(P1, card);
            if name == "Slice from the Shadows" {
                b = b.x(1);
            }
            let spell = b.go();
            let _ = spell;
            t.settle();
            assert_eq!(t.stack_len(), 2, "{name}: ward triggered");
            assert_eq!(untapped_lands(&t, P1), 2);
            t.answer_yes(P1, pay);
            t.resolve();
            let paid = untapped_lands(&t, P1) == 0;
            assert_eq!(paid, pay, "{name}: paid the ward cost = {pay}");
            t.resolve_all();
            assert!(!t.on_battlefield(guard), "{name} resolved (pay = {pay})");
        }
    }
}

// ---------------------------------------------------------------------------------
// City of Solitude.
// ---------------------------------------------------------------------------------

#[test]
fn city_of_solitude_stops_activating_abilities_of_cards_in_hand() {
    cr!("602.1", "602.5", "702.29a");
    ruling!(
        "City of Solitude",
        "City of Solitude can affect abilities of cards that are not on the battlefield because it prevents players from activating those abilities."
    );
    supported("City of Solitude");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "City of Solitude");
    let card = t.hand(P1, "Deadshot Minotaur");
    t.lands(P1, "Mountain", 1);
    // P0's turn: P1 can't cycle.
    assert!(!crate::r_s04_common::can_cycle(&mut t, P1, card));
    assert!(crate::r_s04_common::cycle(&mut t, P1, card, 0).is_err());
    // P1's turn: P1 can.
    t.set_step(P1, Step::PrecombatMain);
    assert!(crate::r_s04_common::can_cycle(&mut t, P1, card));
}

#[test]
fn city_of_solitude_doesnt_stop_triggered_abilities() {
    cr!("602.1", "603.2", "603.3");
    ruling!(
        "City of Solitude",
        "City of Solitude does not stop triggered abilities from being put on the stack. They are never “activated”."
    );
    supported("Soul Warden");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "City of Solitude");
    t.battlefield(P1, "Soul Warden");
    // P0's turn: P0 casts a creature; P1's Soul Warden triggers.
    cast_resolve(&mut t, P0, "Grizzly Bears", &[]);
    assert_eq!(t.life(P1), 21);
}

#[test]
fn city_of_solitude_in_two_headed_giant_is_each_teams_own_turn() {
    cr!("805.4", "805.4a", "810.2", "602.5", "702.29a");
    // With shared team turns, the turn is the team's: on it both players of the active
    // team may cast spells and activate abilities (P0 represents the team's turn), and
    // neither player of the other team may.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P2, "City of Solitude");
    let mut minotaurs = vec![];
    for p in [P1, P2, P3] {
        t.lands(p, "Mountain", 2);
        minotaurs.push(t.hand(p, "Deadshot Minotaur"));
    }
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(crate::r_s04_common::can_cycle(&mut t, P1, minotaurs[0]));
    let life = t.life(P2);
    assert!(t.cast_with(P1, bolt, &[player(P2)]).is_ok());
    t.resolve_all();
    assert_eq!(t.life(P2), life - 3);
    for (i, p) in [P2, P3].into_iter().enumerate() {
        assert!(!crate::r_s04_common::can_cycle(&mut t, p, minotaurs[i + 1]));
        let bolt = t.hand(p, "Lightning Bolt");
        assert!(t.cast_with(p, bolt, &[player(P0)]).is_err(), "{p:?}");
    }
}

// ---------------------------------------------------------------------------------
// Defense Grid in Two-Headed Giant.
// ---------------------------------------------------------------------------------

#[test]
fn defense_grid_in_two_headed_giant() {
    cr!("805.4", "810.2", "601.2f");
    ruling!(
        "Defense Grid",
        "In Two-Headed Giant, spells don't cost extra on your own turn."
    );
    supported("Defense Grid");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P2, "Defense Grid");
    // The team of P0 and P1 takes the turn: P1 (not `turn.active`) pays no extra.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.cast_with(P1, bolt, &[player(P2)]).is_ok());
    t.resolve_all();
    // An opponent's spell costs {3} more.
    t.lands(P3, "Mountain", 1);
    let bolt = t.hand(P3, "Lightning Bolt");
    assert!(t.cast_with(P3, bolt, &[player(P0)]).is_err());
    t.lands(P3, "Wastes", 3);
    assert!(t.cast_with(P3, bolt, &[player(P0)]).is_ok());
}

// ---------------------------------------------------------------------------------
// Magic Missile: damage divided among one to three targets.
// ---------------------------------------------------------------------------------

#[test]
fn magic_missile_keeps_its_division_when_a_target_becomes_illegal() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Magic Missile",
        "If Magic Missile has one or more targets, and some (but not all) of them are illegal as Magic Missile tries to resolve, the amount of damage dealt to the remaining legal targets doesn't change. If all targets become illegal, Magic Missile won't resolve."
    );
    ruling!(
        "Magic Missile",
        "You divide the damage as you cast Magic Missile. Each target must be assigned at least 1 damage."
    );
    supported("Magic Missile");
    // Divided 1/1/1 among two creatures and a player; one creature leaves.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Craw Wurm");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Magic Missile");
    let mm = t.hand(P0, "Magic Missile");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1, 1]));
    t.cast(P0, mm).targets(&[obj(a), obj(b), player(P1)]).go();
    cast_resolve(&mut t, P1, "Unsummon", &[obj(a)]);
    t.resolve_all();
    assert_eq!(damage_on(&t, b), 1, "unchanged");
    assert_eq!(t.life(P1), 19, "unchanged");
    // A division assigning a target 0 isn't legal: the engine doesn't accept it and
    // divides the damage so that each target gets at least 1.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Craw Wurm");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Magic Missile");
    let mm = t.hand(P0, "Magic Missile");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 0]));
    let _ = t.cast(P0, mm).targets(&[obj(a), obj(b)]).try_go();
    t.resolve_all();
    assert!(damage_on(&t, a) >= 1 && damage_on(&t, b) >= 1);
    assert_eq!(damage_on(&t, a) + damage_on(&t, b), 3);
    // All targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let mm = cast_new(&mut t, P0, "Magic Missile", &[obj(a)]);
    cast_resolve(&mut t, P1, "Unsummon", &[obj(a)]);
    t.resolve_all();
    assert!(!on_stack(&t, mm));
    assert!(t.in_graveyard(P0, "Magic Missile"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn kavu_chameleon_new_color_replaces_its_colors() {
    cr!("105.3", "613.1e");
    ruling!(
        "Kavu Chameleon",
        "The new color replaces all previous colors."
    );
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Kavu Chameleon");
    assert!(t.obj_now(kavu).chars.colors.contains(Color::Green));
    t.lands(P0, "Forest", 1);
    crate::r_p022_common::choose_color(&mut t, P0, Color::Blue);
    t.activate(P0, kavu, 0, &[]).unwrap();
    t.resolve_all();
    let colors = t.obj_now(kavu).chars.colors;
    assert!(colors.contains(Color::Blue));
    assert!(!colors.contains(Color::Green), "green is replaced");
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    let colors = t.obj_now(kavu).chars.colors;
    assert!(colors.contains(Color::Green) && !colors.contains(Color::Blue));
}
