//! Rulings batch P216 — Food (CR 111.10b, 205.3g) and forage (CR 701.61): Food-making
//! triggers and their intervening "if" clauses, "whenever you sacrifice a Food",
//! replacement effects on creating Foods, and Food-making spells whose targets become
//! illegal.

use crate::r_p216_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s09_common::*;
use crate::r_s17_common::become_copy;
use crate::r_s25_common::*;
use crate::r_s29_common::put_counters;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------
// Intervening "if" clauses: Butterbur, Elanor Gardner.
// ---------------------------------------------------------------------------

/// Advances to `p`'s end step and puts its "at the beginning of" triggers on the stack.
fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

#[test]
fn butterbur_checks_for_a_food_as_the_end_step_begins_and_as_it_resolves() {
    cr!("603.4", "513.1a");
    ruling!(
        "Butterbur, Bree Innkeeper",
        "Butterbur, Bree Innkeeper's ability checks if you control a Food at the beginning of your end step. If you do, the ability won't trigger."
    );
    ruling!(
        "Butterbur, Bree Innkeeper",
        "If the ability does trigger, but you control a Food when it tries to resolve, the ability will do nothing. No Food will be created."
    );
    supported("Butterbur, Bree Innkeeper");
    // With a Food: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Butterbur, Bree Innkeeper");
    create_tokens(&mut t, P0, "Food", 1);
    to_end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(foods(&t, P0), 1);
    // Without one, it triggers; a Food created in response stops it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Butterbur, Bree Innkeeper");
    to_end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    create_tokens(&mut t, P0, "Food", 1);
    t.resolve_all();
    assert_eq!(foods(&t, P0), 1);
    // Unanswered, it makes one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Butterbur, Bree Innkeeper");
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(foods(&t, P0), 1);
}

#[test]
fn elanor_counts_foods_sacrificed_before_it_entered_and_triggers_once() {
    cr!("603.4", "513.1a", "701.21a");
    ruling!(
        "Elanor Gardner",
        "Elanor Gardner doesn't need to have been on the battlefield when you sacrificed a Food this turn."
    );
    ruling!(
        "Elanor Gardner",
        "Elanor Gardner's last ability will trigger only once during your end step, no matter how many Foods you sacrificed this turn. However, if you haven't sacrificed any Foods this turn as your end step begins, the ability won't trigger at all."
    );
    supported("Elanor Gardner");
    // Two Foods sacrificed in the first main phase; Elanor arrives in the second.
    let mut t = TestGame::new(2);
    create_tokens(&mut t, P0, "Food", 2);
    for f in with_subtype(&t, P0, "Food") {
        sacrifice(&mut t, P0, f);
    }
    t.advance_to(P0, Step::PostcombatMain);
    t.battlefield(P0, "Elanor Gardner");
    let forest = t.library_top(P0, "Forest");
    to_end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.resolve_all();
    let forest = t.g.current(forest);
    assert!(t.on_battlefield(forest));
    assert!(t.obj(forest).tapped);
    // No Food sacrificed as the end step begins: no trigger, and sacrificing one during
    // the end step is too late.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elanor Gardner");
    create_tokens(&mut t, P0, "Food", 1);
    to_end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    let food = with_subtype(&t, P0, "Food")[0];
    sacrifice(&mut t, P0, food);
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(tapped_lands(&t, P0), 0);
}

// ---------------------------------------------------------------------------
// "Whenever you sacrifice a Food": any Food, for any reason.
// ---------------------------------------------------------------------------

#[test]
fn whenever_you_sacrifice_a_food_counts_nontoken_foods_and_any_reason() {
    cr!("205.3g", "701.21a", "603.10a");
    ruling!(
        "Nuka-Cola Vending Machine",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token. For example, if you sacrifice a nontoken Food artifact, such as Candy Trail from the Wilds of Eldraine set, Nuka-Cola Vending Machine's last ability will trigger."
    );
    ruling!(
        "Nuka-Cola Vending Machine",
        "Nuka-Cola Vending Machine's last ability triggers whenever you sacrifice a Food for any reason, not just to activate a Food's activated ability."
    );
    ruling!(
        "Sierra, Nuka's Biggest Fan",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token. For example, if you sacrifice a nontoken Food artifact, such as Candy Trail from the Wilds of Eldraine set, Sierra's last ability will trigger."
    );
    ruling!(
        "Sierra, Nuka's Biggest Fan",
        "Sierra's last ability triggers whenever you sacrifice a Food for any reason, not just to activate a Food's activated ability."
    );
    ruling!(
        "Rapacious Guest",
        "Rapacious Guest's third ability triggers whenever you sacrifice a Food for any reason, not just to activate a Food's activated ability."
    );
    supported("Candy Trail");
    for name in [
        "Nuka-Cola Vending Machine",
        "Sierra, Nuka's Biggest Fan",
        "Rapacious Guest",
    ] {
        supported(name);
        // Candy Trail (a nontoken Food) sacrificed to its own ability, then a Food token
        // sacrificed by an effect (no ability of its own involved).
        for own_ability in [true, false] {
            let mut t = TestGame::new(2);
            let src = t.battlefield(P0, name);
            put_counters(&mut t, src, "quest", 2);
            let food = if own_ability {
                let trail = t.battlefield(P0, "Candy Trail");
                t.lands(P0, "Wastes", 2);
                t.activate(P0, trail, 0, &[]).unwrap();
                trail
            } else {
                create_tokens(&mut t, P0, "Food", 1);
                let f = with_subtype(&t, P0, "Food")[0];
                sacrifice(&mut t, P0, f);
                f
            };
            t.settle();
            assert!(!t.g.is_live(food), "{name}");
            assert!(triggered_from(&t, src) >= 1, "{name}: {own_ability}");
            t.answer_targets(P0, &[obj(src)]);
            t.resolve_all();
            match name {
                "Nuka-Cola Vending Machine" => {
                    let tr = with_subtype(&t, P0, "Treasure");
                    assert_eq!(tr.len(), 1);
                    assert!(t.obj(tr[0]).tapped);
                }
                // Two quest counters: +2/+2 to the target.
                "Sierra, Nuka's Biggest Fan" => assert_eq!(t.pt(src), (5, 6)),
                _ => assert_eq!(t.counters(src, "+1/+1"), 1),
            }
        }
    }
}

fn triggered_from(t: &TestGame, source: ObjectId) -> usize {
    crate::r_s11_common::triggered_from(t, source)
}

#[test]
fn the_cabbage_merchants_sacrifice_gains_no_life() {
    cr!("701.21a", "111.10b");
    ruling!(
        "The Cabbage Merchant",
        "Food sacrificed due to The Cabbage Merchant's second triggered ability is not being sacrificed to the Food's own ability, so you will not gain any life."
    );
    supported("The Cabbage Merchant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Cabbage Merchant");
    create_tokens(&mut t, P0, "Food", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P1);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(foods(&t, P0), 0);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn a_nontoken_food_pays_campsite_cuisines_sacrifice_but_not_two_costs() {
    cr!("205.3g", "603.12", "602.2b");
    ruling!(
        "Campsite Cuisine",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token. For example, you can sacrifice Instant Ramen (an Artifact – Food from the Magic: The Gathering—FINAL FANTASY main set) to pay the cost of Campsite Cuisine's last ability."
    );
    ruling!(
        "Campsite Cuisine",
        "You can't sacrifice a Food to pay multiple costs. For example, you can't sacrifice a Food token to activate its own ability and also to pay the cost of Campsite Cuisine's last ability."
    );
    supported("Campsite Cuisine");
    supported("Instant Ramen");
    // Instant Ramen (a nontoken Food) is sacrificed for X = 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Campsite Cuisine");
    let ramen = t.battlefield(P0, "Instant Ramen");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(ramen)]);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!t.g.is_live(ramen));
    assert_eq!(t.pt(bears), (5, 5));
    // A Food token sacrificed to activate its own ability in response is gone: nothing is
    // left for Campsite Cuisine's sacrifice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Campsite Cuisine");
    create_tokens(&mut t, P0, "Food", 1);
    let food = with_subtype(&t, P0, "Food")[0];
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    t.activate(P0, food, 0, &[]).unwrap();
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.pt(bears), (2, 2));
}

// ---------------------------------------------------------------------------
// Replacement effects on creating Foods: Bilbo, Peregrin Took.
// ---------------------------------------------------------------------------

#[test]
fn bilbo_adds_that_many_treasures_and_each_bilbo_applies_once() {
    cr!("614.1a", "614.5", "616.1");
    ruling!(
        "Bilbo, Fellow Conspirator",
        "If you control Bilbo and would create some number of Food tokens, you will instead create that many Food tokens and that many Treasure tokens."
    );
    ruling!(
        "Bilbo, Fellow Conspirator",
        "If you somehow control two Bilbo, Fellow Conspirators and would create some number of Food tokens, you will instead create that many Food tokens and twice that many Treasure tokens. Three Bilbos means that many Food tokens and three times that many Treasure tokens, and so on."
    );
    supported("Bilbo, Fellow Conspirator");
    for bilbos in 1..=3 {
        let mut t = TestGame::new(2);
        // "Somehow": the legend rule isn't checked while the tokens are created.
        for _ in 0..bilbos {
            t.battlefield(P0, "Bilbo, Fellow Conspirator");
        }
        create_tokens(&mut t, P0, "Food", 3);
        assert_eq!(foods(&t, P0), 3, "{bilbos}");
        assert_eq!(treasures(&t, P0), 3 * bilbos, "{bilbos}");
        // Other tokens aren't affected, nor an opponent's Foods.
        create_tokens(&mut t, P0, "Clue", 1);
        create_tokens(&mut t, P1, "Food", 1);
        assert_eq!(treasures(&t, P0), 3 * bilbos);
        assert_eq!(treasures(&t, P1), 0);
    }
}

#[test]
fn peregrin_took_adds_a_food_to_any_tokens_with_the_creating_effects_other_terms() {
    cr!("614.1a", "111.1");
    ruling!(
        "Peregrin Took",
        "Peregrin Took's first ability applies to all kinds of tokens that its controller creates, not just Food tokens."
    );
    ruling!(
        "Peregrin Took",
        "The additional Food tokens won't have any abilities the other tokens were created with. Anything else specified in the effect creating the token (such as tapped or \"Exile that token at end of combat\") applies to both the original tokens and the Food."
    );
    supported("Peregrin Took");
    supported("Frost Fair Lure Fish");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Peregrin Took");
    create_tokens(&mut t, P0, "Clue", 1);
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 1);
    assert_eq!(foods(&t, P0), 1);
    // Frost Fair Lure Fish: "create two 1/1 blue Fish creature tokens and create two
    // tapped Treasure tokens": a Food with the Fish (untapped) and one with the Treasures
    // (tapped).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Peregrin Took");
    t.enter(P0, "Frost Fair Lure Fish");
    t.resolve_all();
    let foods_now = with_subtype(&t, P0, "Food");
    assert_eq!(foods_now.len(), 2);
    assert_eq!(
        foods_now.iter().filter(|f| t.obj(**f).tapped).count(),
        1
    );
    for f in foods_now {
        let o = t.obj(f);
        assert!(!o.is(CardType::Creature));
        assert_eq!(o.chars.abilities.len(), 1);
    }
    // The two Fish tokens and Frost Fair Lure Fish itself.
    assert_eq!(with_subtype(&t, P0, "Fish").len(), 3);
    assert_eq!(treasures(&t, P0), 2);
}

// ---------------------------------------------------------------------------
// Food-making spells and abilities whose targets become illegal.
// ---------------------------------------------------------------------------

#[test]
fn food_makers_with_an_illegal_target_make_no_food() {
    cr!("608.2b", "111.10b");
    ruling!(
        "Savor",
        "If the target creature is an illegal target as Savor tries to resolve, it won't resolve and none of its effects will happen. You won't create a Food token."
    );
    ruling!(
        "Late to Dinner",
        "If the target is illegal as the spell tries to resolve, perhaps because it has been removed from the graveyard, you will not create a Food token."
    );
    ruling!(
        "Turn into a Pumpkin",
        "If the target nonland permanent is an illegal target by the time Turn into a Pumpkin tries to resolve, the spell won't resolve. You won't draw a card, and you won't create a Food if you spent three blue mana to cast the spell."
    );
    ruling!(
        "Spider Food",
        "You can cast Spider Food without a target just to create a Food token. However, if you do choose a target, and that target is illegal at the time Spider Food tries to resolve, the spell won't resolve and none of its effects will happen. You won't create a Food token."
    );
    // (spell, target card, its owner, its zone)
    let cases: &[(&str, &str, PlayerId, Zone)] = &[
        ("Savor", "Grizzly Bears", P1, Zone::Battlefield),
        ("Late to Dinner", "Grizzly Bears", P0, Zone::Graveyard(P0)),
        ("Turn into a Pumpkin", "Grizzly Bears", P1, Zone::Battlefield),
        ("Spider Food", "Ornithopter", P1, Zone::Battlefield),
    ];
    for (spell, target, owner, zone) in cases {
        supported(spell);
        for illegal in [false, true] {
            let mut t = TestGame::new(2);
            let tgt = place(&mut t, *owner, target, *zone);
            if *spell == "Turn into a Pumpkin" {
                // Three blue mana spent: adamant.
                t.lands(P0, "Island", 4);
            } else {
                lands_for_cost(&mut t, P0, spell);
            }
            let card = t.hand(P0, spell);
            let hand = t.hand_size(P0) - 1;
            t.cast_with(P0, card, &[obj(tgt)]).unwrap();
            if illegal {
                move_to(&mut t, tgt, Zone::Exile);
            }
            t.resolve_all();
            assert_eq!(foods(&t, P0), usize::from(!illegal), "{spell} {illegal}");
            if *spell == "Turn into a Pumpkin" {
                assert_eq!(t.hand_size(P0), hand + usize::from(!illegal));
            }
        }
    }
    // Spider Food with no target at all still makes a Food.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Spider Food", &[]);
    t.resolve_all();
    assert_eq!(foods(&t, P0), 1);
}

#[test]
fn food_making_abilities_with_an_illegal_target_make_no_food() {
    cr!("608.2b", "603.3d", "714.3b");
    ruling!(
        "Ignis Scientia",
        "If the target card is an illegal target as Ignis Scientia's last ability resolves, it won't resolve and none of its effects will happen. You won't create a Food token even if the target card was a creature card."
    );
    ruling!(
        "Hazel's Brewmaster",
        "You don't have to choose a target for Hazel's Brewmaster's second ability. However, if you do, and that target is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't create a Food token."
    );
    ruling!(
        "Of Herbs and Stewed Rabbit",
        "When Of Herbs and Stewed Rabbit's first chapter ability triggers, you can choose not to target a creature just to create a Food token. However, if you do choose a target, and that target is illegal at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. You won't create a Food token."
    );
    supported("Ignis Scientia");
    supported("Hazel's Brewmaster");
    supported("Of Herbs and Stewed Rabbit");
    // Ignis Scientia: "{1}{G}{U}, {T}: Exile target card from a graveyard. If a creature
    // card was exiled this way, create a Food token."
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let ignis = t.battlefield(P0, "Ignis Scientia");
        let bears = t.graveyard(P1, "Grizzly Bears");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 1);
        t.activate(P0, ignis, 0, &[obj(bears)]).unwrap();
        if illegal {
            move_to(&mut t, bears, Zone::Hand(P1));
        }
        t.resolve_all();
        assert_eq!(foods(&t, P0), usize::from(!illegal));
    }
    // Hazel's Brewmaster: "Whenever this creature enters or attacks, exile up to one
    // target card from a graveyard and create a Food token."
    for (target, illegal) in [(false, false), (true, false), (true, true)] {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P1, "Grizzly Bears");
        if target {
            t.answer_targets(P0, &[obj(bears)]);
        } else {
            t.answer_targets(P0, &[]);
        }
        t.enter(P0, "Hazel's Brewmaster");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if illegal {
            move_to(&mut t, bears, Zone::Hand(P1));
        }
        t.resolve_all();
        assert_eq!(foods(&t, P0), usize::from(!illegal), "{target} {illegal}");
    }
    // Of Herbs and Stewed Rabbit, chapter I: "Put a +1/+1 counter on up to one target
    // creature. Create a Food token."
    for (target, illegal) in [(false, false), (true, false), (true, true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if target {
            t.answer_targets(P0, &[obj(bears)]);
        } else {
            t.answer_targets(P0, &[]);
        }
        t.enter(P0, "Of Herbs and Stewed Rabbit");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if illegal {
            move_to(&mut t, bears, Zone::Graveyard(P0));
        }
        t.resolve_all();
        assert_eq!(foods(&t, P0), usize::from(!illegal), "{target} {illegal}");
        if target && !illegal {
            assert_eq!(t.counters(bears, "+1/+1"), 1);
        }
    }
}

#[test]
fn valley_rally_with_a_promised_gift_and_an_illegal_target_does_nothing() {
    cr!("608.2b", "702.174a", "702.174j");
    ruling!(
        "Valley Rally",
        "If the gift was promised and the target is illegal as Valley Rally tries to resolve, it won't resolve and none of its effects will happen. The opponent won't create a Food token and creatures you control won't get +2/+0."
    );
    supported("Valley Rally");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        cast_new(&mut t, P0, "Valley Rally", &[obj(bears)]);
        if illegal {
            move_to(&mut t, bears, Zone::Graveyard(P0));
        }
        t.resolve_all();
        assert_eq!(foods(&t, P1), usize::from(!illegal));
        assert_eq!(t.pt(giant), if illegal { (3, 3) } else { (5, 3) });
    }
}

#[test]
fn foreboding_fruit_makes_a_food_for_its_caster_whoever_draws() {
    cr!("207.2c", "106.4");
    ruling!(
        "Foreboding Fruit",
        "If at least three black mana was spent to cast Foreboding Fruit, you'll create a Food, even if you have another player draw two cards and lose 2 life."
    );
    supported("Foreboding Fruit");
    for black in [true, false] {
        let mut t = TestGame::new(2);
        if black {
            t.lands(P0, "Swamp", 3);
        } else {
            t.lands(P0, "Swamp", 1);
            t.lands(P0, "Wastes", 2);
        }
        let fruit = t.hand(P0, "Foreboding Fruit");
        let hand = t.hand_size(P1);
        t.cast(P0, fruit).target(P1).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P1), hand + 2);
        assert_eq!(t.life(P1), 18);
        assert_eq!(foods(&t, P0), usize::from(black));
        assert_eq!(foods(&t, P1), 0);
    }
}

// ---------------------------------------------------------------------------
// Copies: Brenard, Ginger Sculptor.
// ---------------------------------------------------------------------------

#[test]
fn a_copy_of_brenards_token_copies_the_exceptions() {
    cr!("707.9b", "707.2");
    ruling!(
        "Brenard, Ginger Sculptor",
        "If something becomes a copy of the token, the copy also has base power and toughness 1/1, is a Food Golem artifact creature in addition to its other types, and has \"{2}, {T}, Sacrifice this artifact: You gain 3 life.\""
    );
    supported("Brenard, Ginger Sculptor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brenard, Ginger Sculptor");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    destroy(&mut t, giant);
    t.resolve_all();
    let token = named_of(&t, P0, "Hill Giant");
    assert_eq!(token.len(), 1);
    let token = token[0];
    // Something else becomes a copy of the token: Wall of Wood (0/3, not a Food).
    let wall = t.battlefield(P1, "Wall of Wood");
    become_copy(&mut t, wall, token);
    let o = t.obj_now(wall);
    assert_eq!(o.chars.name.as_str(), "Hill Giant");
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Food") && o.chars.has_subtype("Golem"));
    assert!(o.chars.has_subtype("Giant"));
    assert_eq!((o.chars.power, o.chars.toughness), (Some(1), Some(1)));
    assert_eq!(abilities_with(&t, wall, "gain 3 life"), 1);
    // P1 controls no Brenard: no +2/+2.
    assert_eq!(t.pt(wall), (1, 1));
    // P0's token gets Brenard's bonus.
    assert_eq!(t.pt(token), (3, 3));
}

fn named_of(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    crate::r_s07_common::named_of(t, p, name)
}

// ---------------------------------------------------------------------------
// Counting and choosing Foods as an effect resolves.
// ---------------------------------------------------------------------------

#[test]
fn night_of_the_sweets_revenge_locks_in_x_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Night of the Sweets' Revenge",
        "The value of X is determined only as Night of the Sweets' Revenge's last ability resolves. It won't change later in the turn if the number of Foods you control changes."
    );
    supported("Night of the Sweets' Revenge");
    let mut t = TestGame::new(2);
    let night = t.battlefield(P0, "Night of the Sweets' Revenge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    create_tokens(&mut t, P0, "Food", 2);
    t.lands(P0, "Forest", 7);
    activate_containing(&mut t, P0, night, "{5}{G}{G}");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    create_tokens(&mut t, P0, "Food", 2);
    let f = with_subtype(&t, P0, "Food")[0];
    sacrifice(&mut t, P0, f);
    assert_eq!(t.pt(bears), (4, 4));
}

fn activate_containing(t: &mut TestGame, p: PlayerId, src: ObjectId, needle: &str) {
    crate::r_s06_common::activate_containing(t, p, src, needle).unwrap();
}

#[test]
fn gluttonous_troll_can_sacrifice_its_own_food() {
    cr!("602.2b", "111.10b");
    ruling!(
        "Gluttonous Troll",
        "You can sacrifice a Food token created by Gluttonous Troll's triggered ability to pay for the cost of its activated ability."
    );
    supported("Gluttonous Troll");
    let mut t = TestGame::new(2);
    let troll = t.enter(P0, "Gluttonous Troll");
    t.resolve_all();
    let food = with_subtype(&t, P0, "Food");
    assert_eq!(food.len(), 1);
    t.lands(P0, "Forest", 2);
    t.answer_choose(P0, &[obj(food[0])]);
    t.activate(P0, troll, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(foods(&t, P0), 0);
    assert_eq!(t.pt(troll), (5, 5));
}

#[test]
fn giant_opportunity_chooses_on_resolution_and_makes_at_most_one_giant() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Giant Opportunity",
        "You choose whether to sacrifice two Foods as Giant Opportunity resolves, not as you cast it."
    );
    ruling!(
        "Giant Opportunity",
        "You can't sacrifice more than two Foods to create more than one Giant token."
    );
    supported("Giant Opportunity");
    let mut t = TestGame::new(2);
    create_tokens(&mut t, P0, "Food", 5);
    lands_for_cost(&mut t, P0, "Giant Opportunity");
    let card = t.hand(P0, "Giant Opportunity");
    let from = t.asked().len();
    t.cast(P0, card).go();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. })));
    t.answer_yes(P0, true);
    let fs = with_subtype(&t, P0, "Food");
    t.answer_choose(P0, &fs.iter().map(|f| obj(*f)).collect::<Vec<_>>());
    t.resolve_all();
    assert_eq!(foods(&t, P0), 3);
    assert_eq!(with_subtype(&t, P0, "Giant").len(), 1);
    // Declining: three Foods.
    let mut t = TestGame::new(2);
    create_tokens(&mut t, P0, "Food", 2);
    cast_new(&mut t, P0, "Giant Opportunity", &[]);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(foods(&t, P0), 5);
    assert_eq!(with_subtype(&t, P0, "Giant").len(), 0);
}

#[test]
fn taste_of_death_makes_three_foods_whatever_was_sacrificed() {
    cr!("101.4", "701.21a");
    ruling!(
        "Taste of Death",
        "You create three Food tokens, no matter how many creatures were actually sacrificed, even if some or all players sacrificed no creatures."
    );
    supported("Taste of Death");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Taste of Death", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(foods(&t, P0), 3);
    assert_eq!(foods(&t, P1), 0);
}

#[test]
fn meriadoc_makes_one_food_per_player_attacked_by_halflings() {
    cr!("508.1", "603.2c");
    ruling!(
        "Meriadoc Brandybuck",
        "Meriadoc Brandybuck's ability has you create just one Food token per player you attack with a Halfling, no matter how many Halflings you attack them with beyond the first."
    );
    supported("Meriadoc Brandybuck");
    let mut t = TestGame::new(3);
    let merry = t.battlefield(P0, "Meriadoc Brandybuck");
    let a = t.battlefield(P0, "Eastfarthing Farmer");
    let b = t.battlefield(P0, "Eastfarthing Farmer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[
            (merry, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P2)),
            (bears, Entity::Player(P2)),
        ],
    );
    t.resolve_all();
    assert_eq!(foods(&t, P0), 2);
}

// ---------------------------------------------------------------------------
// Reflexive triggers and resolution-time choices: Eastfarthing Farmer, Curious
// Forager, Bushy Bodyguard, Insatiable Frugivore.
// ---------------------------------------------------------------------------

#[test]
fn eastfarthing_farmers_target_is_chosen_for_a_reflexive_trigger() {
    cr!("603.12", "603.3d");
    ruling!(
        "Eastfarthing Farmer",
        "You don't choose a target for Eastfarthing Farmer's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you create a Food token this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Eastfarthing Farmer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Eastfarthing Farmer");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(target_candidates(&t, P0, from).is_empty());
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    // The Food exists; the reflexive trigger is on the stack with its target.
    assert_eq!(foods(&t, P0), 1);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(target_candidates(&t, P0, from).len(), 1);
    // In response, another Food: counted as it resolves.
    create_tokens(&mut t, P1, "Food", 1);
    create_tokens(&mut t, P0, "Food", 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn curious_foragers_target_is_chosen_after_foraging() {
    cr!("603.12", "701.61a");
    ruling!(
        "Curious Forager",
        "You don't choose a target for Curious Forager's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you forage this way. You choose a target for this ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Curious Forager");
    let mut t = TestGame::new(2);
    create_tokens(&mut t, P0, "Food", 1);
    let card = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Curious Forager");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(target_candidates(&t, P0, from).is_empty());
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(card)]);
    t.resolve();
    assert_eq!(foods(&t, P0), 0);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(target_candidates(&t, P0, from), vec![vec![obj(card)]]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn bushy_bodyguard_forages_as_its_ability_resolves() {
    cr!("608.2c", "701.61a");
    ruling!(
        "Bushy Bodyguard",
        "You decide whether to forage or not as Bushy Bodyguard's last ability resolves. Once that ability starts resolving, it's too late for any player to respond before Bushy Bodyguard gets the +1/+1 counters if you foraged."
    );
    supported("Bushy Bodyguard");
    let mut t = TestGame::new(2);
    create_tokens(&mut t, P0, "Food", 1);
    let from = t.asked().len();
    let bb = t.enter(P0, "Bushy Bodyguard");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(count_yes_no(&t, from), 0);
    assert_eq!(foods(&t, P0), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(count_yes_no(&t, from), 1);
    assert_eq!(foods(&t, P0), 0);
    assert_eq!(t.counters(bb, "+1/+1"), 2);
}

fn count_yes_no(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count()
}

#[test]
fn insatiable_frugivores_triggers_wait_until_it_has_finished_resolving() {
    cr!("603.3", "608.2");
    ruling!(
        "Insatiable Frugivore",
        "You repeat the process as part of the resolution of Insatiable Frugivore's first ability. If the Food token you created or the cards you exiled from your graveyard caused any abilities to trigger, those abilities won't go on the stack until after Insatiable Frugivore's first ability has finished resolving."
    );
    supported("Insatiable Frugivore");
    supported("Reckless Fireweaver");
    // Reckless Fireweaver: "Whenever an artifact you control enters, this creature deals
    // 1 damage to each opponent."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reckless Fireweaver");
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.enter(P0, "Insatiable Frugivore");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.g.resolve_top();
    // Two Foods and an empty graveyard; no trigger was put on the stack yet.
    assert_eq!(foods(&t, P0), 2);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
