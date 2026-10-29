//! Rulings batch P098 — casting a card while an ability resolves (CR 608.2g): Goblin
//! Dark-Dwellers, Daring Waverider, Chandra, Acolyte of Flame, Shiko, Paragon of the Way,
//! Seifer Almasy, Sword of Once and Future and Treasure Keeper: costs that can and can't
//! be paid, timing permissions, and that the card can't be cast later.

use crate::r_s01_common::*;
use crate::r_s06_common::attach_new;
use crate::r_s07_common::cast_methods;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The options of each choice of how to cast a spell asked since `from`.
fn method_choices(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseCastingMethod { options, .. } => Some(options),
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.contains("how to cast") => Some(options),
            _ => None,
        })
        .collect()
}

/// `name` (Goblin Dark-Dwellers or Daring Waverider) enters under P0's control and its
/// trigger targets `card` in P0's graveyard; P0 answers "cast it" with `yes`.
fn cast_from_trigger(t: &mut TestGame, name: &str, card: ObjectId, yes: bool, then: &[Entity]) {
    supported(name);
    t.answer_targets(P0, &[obj(card)]);
    t.answer_yes(P0, yes);
    for e in then {
        t.answer_targets(P0, &[*e]);
    }
    t.enter(P0, name);
    t.settle();
}

#[test]
fn free_cast_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9", "118.9a", "601.2b", "601.2f");
    ruling!(
        "Goblin Dark-Dwellers",
        "If you cast a card \"without paying its mana cost,\" you can't pay alternative costs such as surge costs. You can pay additional costs. If the card has mandatory additional costs, you must pay those."
    );
    ruling!(
        "Daring Waverider",
        "If you cast a spell “without paying its mana cost”, you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast the spell."
    );
    for name in ["Goblin Dark-Dwellers", "Daring Waverider"] {
        // Burst Lightning's kicker {4} (optional additional cost) may be paid.
        let mut t = TestGame::new(2);
        let burst = t.graveyard(P0, "Burst Lightning");
        t.lands(P0, "Mountain", 4);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        cast_from_trigger(&mut t, name, burst, true, &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), 16, "{name}");
        assert_eq!(tapped_lands(&t, P0), 4, "{name}");
        assert!(t.in_exile("Burst Lightning"), "{name}");
        // Village Rites' mandatory sacrifice is paid.
        let mut t = TestGame::new(2);
        let rites = t.graveyard(P0, "Village Rites");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[obj(bears)]);
        cast_from_trigger(&mut t, name, rites, true, &[]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        assert_eq!(t.hand_size(P0), 2, "{name}");
        // Invigorate's alternative cost (an opponent gains 3 life) isn't offered or paid.
        let mut t = TestGame::new(2);
        let inv = t.graveyard(P0, "Invigorate");
        t.battlefield(P0, "Forest");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        cast_from_trigger(&mut t, name, inv, true, &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.pt(bears), (6, 6), "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
        assert!(method_choices(&t, from).is_empty(), "{name}");
    }
}

#[test]
fn goblin_dark_dwellers_card_stays_if_it_cant_or_isnt_cast() {
    cr!("608.2g", "601.2c");
    ruling!(
        "Goblin Dark-Dwellers",
        "If you can't cast the target instant or sorcery card, perhaps because there are no legal targets available, or if you choose not to cast it, it will remain in your graveyard."
    );
    // Shatter with no artifact on the battlefield.
    let mut t = TestGame::new(2);
    let shatter = t.graveyard(P0, "Shatter");
    cast_from_trigger(&mut t, "Goblin Dark-Dwellers", shatter, true, &[]);
    t.resolve_all();
    assert_eq!(t.zone(shatter), Zone::Graveyard(P0));
    // Choosing not to cast Lightning Bolt.
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    cast_from_trigger(&mut t, "Goblin Dark-Dwellers", bolt, false, &[]);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn cast_during_resolution_ignores_type_timing_but_not_other_timing() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Goblin Dark-Dwellers",
        "You cast the card during the resolution of the enters-the-battlefield ability, not later in the turn. If it's a sorcery card, ignore the timing restrictions based on it being a sorcery card. Other timing restrictions, such as \"Cast [this card] only during combat,\" must be followed."
    );
    // A sorcery on P1's turn, with Goblin Dark-Dwellers entering at instant speed.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.graveyard(P0, "Divination");
    cast_from_trigger(&mut t, "Goblin Dark-Dwellers", div, true, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_exile("Divination"));
    // Gorilla War Cry ("Cast this spell only during combat before blockers are
    // declared") in a main phase can't be cast.
    let mut t = TestGame::new(2);
    let cry = t.graveyard(P0, "Gorilla War Cry");
    cast_from_trigger(&mut t, "Goblin Dark-Dwellers", cry, true, &[]);
    t.resolve_all();
    assert_eq!(t.zone(cry), Zone::Graveyard(P0));
}

#[test]
fn combat_damage_triggers_cast_a_sorcery_mid_combat() {
    cr!("608.2g", "307.1");
    ruling!(
        "Seifer Almasy",
        "You cast the instant or sorcery while Seifer Almasy's last ability is resolving and still on the stack. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    ruling!(
        "Sword of Once and Future",
        "You choose whether or not to cast the instant or sorcery card as the triggered ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Seifer Almasy");
    supported("Sword of Once and Future");
    // Seifer Almasy: "Whenever Seifer Almasy deals combat damage to a player, you may cast
    // target instant or sorcery card with mana value 3 or less from your graveyard
    // without paying its mana cost."
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let seifer = t.battlefield(P0, "Seifer Almasy");
        let div = t.graveyard(P0, "Divination");
        t.answer_targets(P0, &[obj(div)]);
        // Attacking alone, Seifer has double strike: it deals combat damage twice.
        t.answer_yes(P0, yes);
        t.answer_yes(P0, false);
        attack_with(&mut t, &[(seifer, Entity::Player(P1))]);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.hand_size(P0), if yes { 2 } else { 0 }, "{yes}");
        if !yes {
            // Not later in the turn.
            t.advance_to(P0, Step::PostcombatMain);
            assert!(cast_methods(&mut t, P0, div).is_empty());
            assert_eq!(t.zone(div), Zone::Graveyard(P0));
        }
    }
    // Sword of Once and Future: "... surveil 2. Then you may cast an instant or sorcery
    // spell with mana value 2 or less from your graveyard without paying its mana cost."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Once and Future", bears);
    let ponder = t.graveyard(P0, "Ponder");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(ponder)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
    assert!(t.in_exile("Ponder"), "{}", t.dump_log());
}

#[test]
fn declining_to_cast_on_resolution_leaves_the_card_uncastable_later() {
    cr!("608.2g", "707.12");
    ruling!(
        "Daring Waverider",
        "You cast the spell while Daring Waverider’s ability is resolving and still on the stack. You can’t wait to cast it later in the turn."
    );
    ruling!(
        "Shiko, Paragon of the Way",
        "You cast the copy while Shiko’s last ability is resolving and still on the stack. You can’t wait to cast it later in the turn."
    );
    ruling!(
        "Chandra, Acolyte of Flame",
        "If you wish to cast the target card, you must cast it while Chandra’s last ability is resolving. You can’t cast it later in the turn."
    );
    // Daring Waverider: declined, Divination stays in the graveyard, not castable.
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    t.lands(P0, "Island", 3);
    cast_from_trigger(&mut t, "Daring Waverider", div, false, &[]);
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P0));
    assert!(cast_methods(&mut t, P0, div).is_empty());
    // Shiko: the card is exiled and copied; declining the copy leaves nothing to cast.
    supported("Shiko, Paragon of the Way");
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    t.answer_targets(P0, &[obj(div)]);
    t.answer_yes(P0, false);
    t.enter(P0, "Shiko, Paragon of the Way");
    t.resolve_all();
    assert!(t.in_exile("Divination"));
    assert_eq!(t.hand_size(P0), 0);
    let exiled = t.g.current(div);
    assert!(cast_methods(&mut t, P0, exiled).is_empty());
    // Chandra: declined, not castable later this turn (even with the mana).
    supported("Chandra, Acolyte of Flame");
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Acolyte of Flame");
    let div = t.graveyard(P0, "Divination");
    t.lands(P0, "Island", 3);
    t.answer_yes(P0, false);
    t.activate(P0, chandra, 2, &[obj(div)])
        .expect("activate -2");
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P0));
    assert!(cast_methods(&mut t, P0, div).is_empty());
}

#[test]
fn chandra_acolyte_of_flame_pays_the_cost_or_an_alternative_cost() {
    cr!("601.2b", "601.2h", "118.9");
    ruling!(
        "Chandra, Acolyte of Flame",
        "You must pay the costs for the spell you wish to cast with Chandra’s last ability. If that spell has any alternative costs, you may pay those."
    );
    // Divination with no mana: it can't be cast.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Acolyte of Flame");
    let div = t.graveyard(P0, "Divination");
    t.answer_yes(P0, true);
    t.activate(P0, chandra, 2, &[obj(div)])
        .expect("activate -2");
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P0));
    // With three Islands, it's cast for its mana cost.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Acolyte of Flame");
    let div = t.graveyard(P0, "Divination");
    t.lands(P0, "Island", 3);
    t.answer_yes(P0, true);
    t.activate(P0, chandra, 2, &[obj(div)])
        .expect("activate -2");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(tapped_lands(&t, P0), 3);
    assert!(t.in_exile("Divination"));
    // Invigorate for its alternative cost ("If you control a Forest, rather than pay this
    // spell's mana cost, you may have an opponent gain 3 life").
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Acolyte of Flame");
    let inv = t.graveyard(P0, "Invigorate");
    t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.activate(P0, chandra, 2, &[obj(inv)])
        .expect("activate -2");
    t.resolve_all();
    let methods = method_choices(&t, from);
    assert_eq!(methods.len(), 1, "{methods:?}");
    assert_eq!(methods[0].len(), 2, "{methods:?}");
    assert_eq!(t.pt(bears), (6, 6));
    assert_eq!(t.life(P1), 23);
    assert_eq!(tapped_lands(&t, P0), 0);
}

#[test]
fn treasure_keeper_uncast_and_all_lands_go_back_in_a_random_order() {
    cr!("608.2g");
    ruling!(
        "Treasure Keeper",
        "If you don't cast the nonland card with mana value 3 or less, it'll be put on the bottom of your library in a random order with the other cards."
    );
    ruling!(
        "Treasure Keeper",
        "If your library has no nonland cards in it with mana value 3 or less, you'll reveal all the cards in your library, then put them back in a random order."
    );
    supported("Treasure Keeper");
    // Not cast: Divination and the lands above it go to the bottom.
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Treasure Keeper");
    let cards = stack_library(&mut t, P0, &["Forest", "Island", "Divination"]);
    t.answer_yes(P0, false);
    crate::r_s02_common::destroy(&mut t, keeper);
    t.resolve_all();
    let lib = &t.g.players[P0.idx()].library;
    let n = lib.len();
    let bottom: Vec<ObjectId> = lib[..3].to_vec();
    for c in &cards {
        assert!(bottom.contains(&t.g.current(*c)), "{n}");
    }
    assert!(t.in_graveyard(P0, "Treasure Keeper"));
    // Only lands (and an expensive spell): all revealed and put back.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].library.clear();
    let keeper = t.battlefield(P0, "Treasure Keeper");
    stack_library(&mut t, P0, &["Forest", "Island", "Air Elemental", "Plains"]);
    crate::r_s02_common::destroy(&mut t, keeper);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 4);
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.named_on_battlefield("Air Elemental").is_empty());
}
