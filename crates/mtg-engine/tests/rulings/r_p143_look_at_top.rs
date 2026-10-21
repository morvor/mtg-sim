//! Rulings batch P143 — "look at the top N cards" abilities: Fecund Greenshell, Cream of
//! the Crop, Parcelbeast, Risen Reef, Vivien's Grizzly, Merchant's Dockhand, Search for
//! Azcanta, Geometer's Arthropod, Loot, Trail of Crumbs, Prophetic Titan, Icon of Ancestry
//! and Monumental Henge.

use crate::r_p107_common::{act, mana};
use crate::r_p116_common::{enter_together, play_forests};
use crate::r_p117_common::grow_then_exile;
use crate::r_p121_common::damage;
use crate::r_p143_common::*;
use crate::r_s02_common::{create_token, destroy};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn ten_lands(t: &mut TestGame) {
    t.lands(P0, "Forest", 10);
}

/// The sizes of the choices among library cards asked of P0 since `from`.
fn looks_since(t: &TestGame, from: usize) -> Vec<usize> {
    entity_choices(t, P0, from)
        .iter()
        .filter(|(_, _, c)| {
            c.iter().all(|e| match e {
                Entity::Object(o) => matches!(t.g.obj(*o).zone, Zone::Library(_)),
                _ => false,
            })
        })
        .map(|(_, _, c)| c.len())
        .collect()
}

/// The cards (followed across zone changes) as they are now.
fn now(t: &TestGame, ids: &[ObjectId]) -> Vec<ObjectId> {
    ids.iter().map(|i| t.g.current(*i)).collect()
}

#[test]
fn fecund_greenshell_toughness_bonus_and_damage() {
    cr!("704.5g", "611.3a");
    supported("Fecund Greenshell");
    ruling!(
        "Fecund Greenshell",
        "Damage remains marked on creatures until the turn ends. If Fecund Greenshell’s second ability stops applying (because Fecund Greenshell leaves the battlefield or loses its abilities or because you no longer control ten or more lands), then any creatures that needed the toughness bonus to stay alive will die."
    );
    let mut t = TestGame::new(2);
    ten_lands(&mut t);
    let f = t.battlefield(P0, "Fecund Greenshell");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    damage(&mut t, bears, 3);
    assert!(t.on_battlefield(bears));
    destroy(&mut t, f);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn fecund_greenshell_checks_power_and_toughness_as_the_creature_enters() {
    cr!("603.6a", "603.10");
    ruling!(
        "Fecund Greenshell",
        "Fecund Greenshell’s last ability checks the power and toughness of a creature only at the moment it enters."
    );
    // Merfolk Thaumaturgist (1/2) triggers it; with "Creatures you control get +2/+0." it
    // enters as a 3/2 and doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fecund Greenshell");
    let lib = set_library(&mut t, P0, &["Memnite", "Island"]);
    let m = t.enter(P0, "Merfolk Thaumaturgist");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // It dies before the ability resolves: the ability still resolves.
    destroy(&mut t, m);
    t.resolve_all();
    assert!(is_in(&t, lib[0], Zone::Hand(P0)));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fecund Greenshell");
    let anthem = custom_card(
        "Power Anthem",
        "Enchantment",
        "{1}",
        None,
        "Creatures you control get +2/+0.",
    );
    t.custom(P0, anthem, Zone::Battlefield);
    t.enter(P0, "Merfolk Thaumaturgist");
    t.settle();
    assert_eq!(t.stack_len(), 0);

    ruling!(
        "Fecund Greenshell",
        "If the top card of your library is a land card, you may choose to put it into your hand rather than onto the battlefield tapped."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fecund Greenshell");
    let lib = set_library(&mut t, P0, &["Island", "Memnite"]);
    t.answer_yes(P0, false);
    t.enter(P0, "Merfolk Thaumaturgist");
    t.resolve_all();
    assert!(is_in(&t, lib[0], Zone::Hand(P0)));
}

#[test]
fn cream_of_the_crop_uses_the_power_as_it_resolves() {
    cr!("608.2h", "113.7a");
    supported("Cream of the Crop");
    ruling!("Cream of the Crop", "If the creature has 0 power, this ability has no effect.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cream of the Crop");
    let lib = set_library(&mut t, P0, &["Island", "Swamp"]);
    t.answer_yes(P0, true);
    let from = n_asked(&t);
    t.enter(P0, "Ornithopter");
    t.resolve_all();
    assert!(looks_since(&t, from).is_empty());
    assert_eq!(library_top_first(&t, P0), lib);

    ruling!(
        "Cream of the Crop",
        "If the creature has 1 power, this ability lets you look at the top card of your library, then you'll have to leave it there."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cream of the Crop");
    let lib = set_library(&mut t, P0, &["Island", "Swamp"]);
    t.answer_yes(P0, true);
    t.enter(P0, "Memnite");
    t.resolve_all();
    assert_eq!(library_top_first(&t, P0), now(&t, &lib));

    ruling!(
        "Cream of the Crop",
        "The creature's power is checked as this ability resolves. If the creature has left the battlefield, use its last known information."
    );
    // Grizzly Bears enters; in response it gets two +1/+1 counters and is exiled: X is 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cream of the Crop");
    let lib = set_library(&mut t, P0, &["Island", "Swamp", "Plains", "Memnite", "Forest", "Wastes"]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[3])]);
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    grow_then_exile(&mut t, bears, 2);
    let from = n_asked(&t);
    t.resolve_all();
    assert_eq!(looks_since(&t, from), vec![4]);
    let top = library_top_first(&t, P0);
    assert_eq!((top[0], top[1]), (t.g.current(lib[3]), lib[4]));
}

#[test]
fn parcelbeast_and_risen_reef_may_take_a_land_into_hand() {
    cr!("608.2c", "305.2");
    supported("Parcelbeast");
    ruling!(
        "Parcelbeast",
        "If the top card of your library is a land card, you may just put it into your hand instead of putting it onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Parcelbeast");
    let lib = set_library(&mut t, P0, &["Island", "Memnite"]);
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, false);
    act(&mut t, P0, p, "Look at the top card", &[]).unwrap();
    t.resolve_all();
    assert!(is_in(&t, lib[0], Zone::Hand(P0)));

    ruling!(
        "Parcelbeast",
        "If you don’t put the top card of your library onto the battlefield, you put it into your hand without revealing it."
    );
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Parcelbeast");
    let lib = set_library(&mut t, P0, &["Memnite", "Island"]);
    mana(&mut t, P0, ManaType::C, 1);
    act(&mut t, P0, p, "Look at the top card", &[]).unwrap();
    t.resolve_all();
    assert!(is_in(&t, lib[0], Zone::Hand(P0)));

    ruling!(
        "Parcelbeast",
        "Parcelbeast’s effect doesn’t count as playing a land. It can put a land card onto the battlefield even if you’ve already played your land for the turn."
    );
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Parcelbeast");
    let lib = set_library(&mut t, P0, &["Island", "Memnite"]);
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    assert_eq!(play_forests(&mut t, P0, 1), 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, true);
    act(&mut t, P0, p, "Look at the top card", &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(lib[0]));
    // Nor does it use up a land play.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Parcelbeast");
    let lib = set_library(&mut t, P0, &["Island", "Memnite"]);
    t.hand(P0, "Forest");
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, true);
    act(&mut t, P0, p, "Look at the top card", &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(lib[0]));
    assert_eq!(play_forests(&mut t, P0, 1), 1);

    supported("Risen Reef");
    ruling!(
        "Risen Reef",
        "If you put the card into your hand, you don't have to reveal it."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Island", "Memnite"]);
    t.answer_yes(P0, false);
    t.enter(P0, "Risen Reef");
    t.resolve_all();
    assert!(is_in(&t, lib[0], Zone::Hand(P0)));
}

#[test]
fn viviens_grizzly_puts_anything_not_taken_on_the_bottom() {
    cr!("401.4");
    supported("Vivien's Grizzly");
    ruling!(
        "Vivien's Grizzly",
        "The ability of Vivien’s Grizzly has you put the card on the bottom of your library if you don’t put it into your hand for any reason"
    );
    for (top, take) in [("Island", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Vivien's Grizzly");
        let lib = set_library(&mut t, P0, &[top, "Memnite", "Swamp"]);
        mana(&mut t, P0, ManaType::C, 3);
        mana(&mut t, P0, ManaType::G, 1);
        t.answer_yes(P0, take);
        t.answer_choose(P0, if take { &[] } else { &[] });
        act(&mut t, P0, g, "Look at the top card", &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 0, "{top}");
        assert_eq!(library_top_first(&t, P0), now(&t, &[lib[1], lib[2], lib[0]]), "{top}");
    }
}

/// Activates Merchant's Dockhand with `x` (tapping that many of `arts`).
fn dockhand(t: &mut TestGame, d: ObjectId, x: i64) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    mana(t, P0, ManaType::C, 3);
    mana(t, P0, ManaType::U, 1);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    act(t, P0, d, "Look at the top X", &[])
}

#[test]
fn merchants_dockhand_x() {
    cr!("107.3", "602.2b");
    supported("Merchant's Dockhand");
    ruling!(
        "Merchant's Dockhand",
        "If the value of X is 0, you don’t look at or move any cards."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Merchant's Dockhand");
    let lib = set_library(&mut t, P0, &["Island", "Memnite"]);
    let from = n_asked(&t);
    dockhand(&mut t, d, 0).unwrap();
    t.resolve_all();
    assert!(looks_since(&t, from).is_empty());
    assert_eq!(library_top_first(&t, P0), lib);
    assert_eq!(t.hand_size(P0), 0);

    ruling!(
        "Merchant's Dockhand",
        "If the value of X is greater than or equal to the number of cards in your library, you can order your library as you wish."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Merchant's Dockhand");
    let arts: Vec<Entity> = (0..4).map(|_| obj(t.battlefield(P0, "Ornithopter"))).collect();
    let lib = set_library(&mut t, P0, &["Island", "Memnite", "Swamp", "Plains"]);
    t.answer_choose(P0, &arts);
    t.answer_choose(P0, &[obj(lib[1])]);
    // Bottom order (top first): Plains, Island, Swamp.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![2, 0, 1]));
    let from = n_asked(&t);
    dockhand(&mut t, d, 4).unwrap();
    t.resolve_all();
    assert_eq!(looks_since(&t, from), vec![4]);
    let order = asked_of(&t, P0, from)
        .into_iter()
        .find_map(|d| match d {
            Decision::Order { items, .. } => Some(items),
            _ => None,
        })
        .unwrap();
    assert_eq!(order, vec!["Island", "Swamp", "Plains"]);
    assert_eq!(library_top_first(&t, P0), now(&t, &[lib[3], lib[0], lib[2]]));

    ruling!(
        "Merchant's Dockhand",
        "Merchant’s Dockhand is tapped to pay the {T} cost. It can’t also be one of the X untapped artifacts you tap."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Merchant's Dockhand");
    t.battlefield(P0, "Ornithopter");
    set_library(&mut t, P0, &["Island", "Memnite", "Swamp"]);
    assert!(dockhand(&mut t, d, 2).is_err() || {
        t.resolve_all();
        t.hand_size(P0) == 1 && t.library_size(P0) == 2 && {
            // X was capped at 1: only the top card was looked at.
            false
        }
    });
}

#[test]
fn search_for_azcanta_leaves_the_card_on_top() {
    cr!("701.25a");
    supported("Search for Azcanta // Azcanta, the Sunken Ruin");
    ruling!(
        "Search for Azcanta // Azcanta, the Sunken Ruin",
        "If you don't put the top card of your library into your graveyard while resolving Search for Azcanta's triggered ability, you'll leave it on top of your library (and probably draw it during your draw step)."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Search for Azcanta");
    let lib = set_library(&mut t, P0, &["Memnite", "Island", "Swamp"]);
    t.set_step(P1, Step::End);
    t.answer(P0, DecisionKind::Surveil, Answer::Entities(vec![]));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(is_in(&t, lib[0], Zone::Hand(P0)));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn geometers_arthropod_uses_x_of_the_spell_as_it_last_existed() {
    cr!("608.2h", "107.3f", "603.3");
    supported("Geometer's Arthropod");
    ruling!(
        "Geometer's Arthropod",
        "Geometer's Arthropod's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving. In that case, use the value of X as that spell last existed on the stack to determine the value of X for Geometer's Arthropod's ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Geometer's Arthropod");
    let lib = set_library(&mut t, P0, &["Island", "Memnite", "Swamp", "Plains", "Forest"]);
    let one = t.hand(P0, "Endless One");
    t.lands(P0, "Wastes", 3);
    let spell = t.cast(P0, one).x(3).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    crate::r_p119_common::counter_spell(&mut t, spell);
    t.answer_choose(P0, &[obj(lib[1])]);
    let from = n_asked(&t);
    t.resolve_all();
    assert_eq!(looks_since(&t, from), vec![3]);
    assert!(is_in(&t, lib[1], Zone::Hand(P0)));
    assert_eq!(library_top_first(&t, P0)[0], lib[3]);

    ruling!(
        "Geometer's Arthropod",
        "In those cases, use the value of X as the spell exists on the stack at the time Geometer's Arthropod's ability resolves to determine the value of X for Geometer's Arthropod's ability."
    );
    // An effect changes X of the spell on the stack (as Unbound Flourishing would) after
    // the ability triggered: the ability uses the new value.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Geometer's Arthropod");
    set_library(&mut t, P0, &["Island", "Memnite", "Swamp", "Plains", "Forest"]);
    let one = t.hand(P0, "Endless One");
    t.lands(P0, "Wastes", 2);
    let spell = t.cast(P0, one).x(2).go();
    t.settle();
    t.g.obj_mut(spell).stack.as_mut().unwrap().x = Some(4);
    let from = n_asked(&t);
    t.resolve();
    assert_eq!(looks_since(&t, from), vec![4]);
}

#[test]
fn loot_and_x_costs_and_extra_land_plays() {
    cr!("202.3e", "305.2a");
    supported("Loot, Exuberant Explorer");
    ruling!(
        "Loot, Exuberant Explorer",
        "If a creature card in your library has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    // With no lands, Endless One (mana value 0) qualifies; Grizzly Bears doesn't.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Loot, Exuberant Explorer");
    let lib = set_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Endless One", "Island", "Swamp", "Plains", "Forest"],
    );
    mana(&mut t, P0, ManaType::C, 4);
    mana(&mut t, P0, ManaType::G, 2);
    let from = n_asked(&t);
    t.answer_choose(P0, &[obj(lib[1])]);
    act(&mut t, P0, l, "Look at the top six", &[]).unwrap();
    t.resolve_all();
    let cands = &entity_choices(&t, P0, from)[0].2;
    assert!(cands.contains(&obj(lib[1])) && !cands.contains(&obj(lib[0])));
    // (It entered as a 0/0 and died.)
    assert!(is_in(&t, lib[1], Zone::Graveyard(P0)));

    ruling!(
        "Loot, Exuberant Explorer",
        "The effect of Loot's first ability is cumulative with similar effects."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loot, Exuberant Explorer");
    t.battlefield(P0, "Exploration");
    for _ in 0..4 {
        t.hand(P0, "Forest");
    }
    assert_eq!(play_forests(&mut t, P0, 4), 3);
}

#[test]
fn trail_of_crumbs_pays_once_per_trigger() {
    cr!("603.5", "118.8");
    supported("Trail of Crumbs");
    ruling!(
        "Trail of Crumbs",
        "While resolving the second triggered ability, you can't pay {1} multiple times to look at more cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trail of Crumbs");
    let food = create_token(&mut t, P0, "Food");
    let lib = set_library(&mut t, P0, &["Island", "Swamp", "Plains", "Forest"]);
    mana(&mut t, P0, ManaType::C, 6);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let from = n_asked(&t);
    act(&mut t, P0, food, "gain 3 life", &[]).unwrap();
    t.resolve_all();
    let pays = asked_of(&t, P0, from)
        .iter()
        .filter(|d| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(pays, 1);
    // Two cards were looked at (and put on the bottom).
    assert_eq!(library_top_first(&t, P0)[0], lib[2]);
}

fn titan_modes(t: &TestGame, from: usize) -> Vec<(u32, u32)> {
    asked_of(t, P0, from)
        .into_iter()
        .filter_map(|d| match d {
            Decision::ChooseModes { min, max, .. } => Some((min, max)),
            _ => None,
        })
        .collect()
}

#[test]
fn prophetic_titan_counts_card_types() {
    cr!("205.2a", "205.4a", "205.3a");
    supported("Prophetic Titan");
    ruling!(
        "Prophetic Titan",
        "The card types that can appear in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types"
    );
    // Crib Swap (kindred instant), Invasion of Ravnica (battle) and Snow-Covered Forest
    // (land): four card types.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Crib Swap");
    t.graveyard(P0, "Invasion of Ravnica");
    t.graveyard(P0, "Snow-Covered Forest");
    let from = n_asked(&t);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Prophetic Titan");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), 1);
    assert!(titan_modes(&t, from).is_empty() || titan_modes(&t, from)[0].0 == 2);
    // Svella (legendary snow creature), Snow-Covered Forest (basic snow land), Prophetic
    // Titan (Giant Wizard creature): two card types.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Svella, Ice Shaper");
    t.graveyard(P0, "Snow-Covered Forest");
    t.graveyard(P0, "Prophetic Titan");
    let from = n_asked(&t);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.enter(P0, "Prophetic Titan");
    t.resolve_all();
    assert_eq!(titan_modes(&t, from)[0], (1, 1));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn prophetic_titan_modes_are_fixed_as_it_triggers() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Prophetic Titan",
        "Prophetic Titan's ability checks the number of card types in your graveyard as it's put on the stack. If you have four or more types in your graveyard at that time, but fewer than four as it resolves, you will still get to use both modes chosen."
    );
    let mut t = TestGame::new(2);
    let gy = [
        t.graveyard(P0, "Crib Swap"),
        t.graveyard(P0, "Invasion of Ravnica"),
        t.graveyard(P0, "Snow-Covered Forest"),
    ];
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Prophetic Titan");
    t.settle();
    for c in gy {
        crate::r_p117_common::exile(&mut t, c);
    }
    assert_eq!(t.graveyard_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), 1);

    ruling!(
        "Prophetic Titan",
        "If only the first mode is chosen, or if both modes are chosen, and the chosen target is illegal as the ability tries to resolve, the ability won't resolve and none of its effects will happen."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Crib Swap");
    t.graveyard(P0, "Invasion of Ravnica");
    t.graveyard(P0, "Snow-Covered Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lib = set_library(&mut t, P0, &["Island", "Swamp", "Plains", "Forest", "Memnite"]);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Prophetic Titan");
    t.settle();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(library_top_first(&t, P0), lib);
}

#[test]
fn icon_of_ancestry_chooses_a_creature_type() {
    cr!("205.3e");
    supported("Icon of Ancestry");
    ruling!(
        "Icon of Ancestry",
        "You must choose an existing creature type, such as Vampire or Warrior. Card types (such as artifact) and supertypes (such as legendary) can't be chosen."
    );
    let mut t = TestGame::new(2);
    let from = n_asked(&t);
    t.enter(P0, "Icon of Ancestry");
    let opts = asked_of(&t, P0, from)
        .into_iter()
        .find_map(|d| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .expect("a creature type choice");
    let has = |s: &str| opts.iter().any(|o| o.eq_ignore_ascii_case(s));
    assert!(has("Vampire") && has("Warrior"));
    assert!(!has("Artifact") && !has("Legendary") && !has("Forest"));
}

#[test]
fn monumental_henge_needs_a_plains_already() {
    cr!("614.1c", "614.12");
    supported("Monumental Henge");
    ruling!(
        "Monumental Henge",
        "You must already control a Plains as Monumental Henge enters the battlefield for it to enter untapped. If it enters the battlefield at the same time as a Plains when you control no other Plains, it will enter tapped."
    );
    let mut t = TestGame::new(2);
    let both = enter_together(&mut t, P0, &["Monumental Henge", "Plains"]);
    assert!(t.obj_now(both[0]).tapped);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Plains");
    let h = t.enter(P0, "Monumental Henge");
    assert!(!t.obj_now(h).tapped);
}

#[test]
fn otherwise_after_an_optional_instruction_covers_declining_it() {
    cr!("608.2c", "706.3a");
    // Treasure Chest's 20: "Search your library for a card. If it's an artifact card, you
    // may put it onto the battlefield. Otherwise, put that card into your hand. Then
    // shuffle."
    supported("Treasure Chest");
    ruling!(
        "Treasure Chest",
        "The “otherwise” in the last ability refers to whether or not you put it onto the battlefield, not whether or not it's an artifact card. You may put it into your hand even if it's an artifact card."
    );
    for put in [true, false] {
        let mut t = TestGame::new(2);
        let chest = t.battlefield(P0, "Treasure Chest");
        let ring = t.library_top(P0, "Sol Ring");
        t.lands(P0, "Plains", 4);
        t.set_step(P0, Step::PrecombatMain);
        t.g.dice.loaded.push_back(20);
        t.answer_choose(P0, &[obj(ring)]);
        t.answer_yes(P0, put);
        t.activate(P0, chest, 0, &[]).unwrap();
        t.resolve_all();
        let want = if put { Zone::Battlefield } else { Zone::Hand(P0) };
        assert_eq!(t.zone(ring), want);
    }

    // Aid from the Cowl: "... reveal the top card of your library. If it's a permanent
    // card, you may put it onto the battlefield. Otherwise, you may put it on the bottom
    // of your library."
    supported("Aid from the Cowl");
    ruling!(
        "Aid from the Cowl",
        "If the revealed card is not a permanent card or if you choose not to put it onto the battlefield, you may put it on the bottom of your library. If you don't, it remains on top of your library."
    );
    for (top, bottom) in [("Memnite", true), ("Memnite", false), ("Lightning Bolt", true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Aid from the Cowl");
        let lib = set_library(&mut t, P0, &[top, "Island", "Swamp"]);
        let gone = t.battlefield(P0, "Ornithopter");
        destroy(&mut t, gone);
        if t.g.obj(lib[0]).is(mtg_engine::types::CardType::Artifact) {
            t.answer_yes(P0, false);
        }
        t.answer_yes(P0, bottom);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert!(is_in(&t, lib[0], Zone::Library(P0)), "{top}");
        let first = library_top_first(&t, P0)[0];
        assert_eq!(first == t.g.current(lib[0]), !bottom, "{top} {bottom}");
    }
}
