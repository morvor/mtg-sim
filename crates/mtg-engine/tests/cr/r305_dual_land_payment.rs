//! Regression tests: paying mana costs with lands that have several basic land types
//! (CR 305.6), and more generally with permanents that have several mana abilities of
//! which only one can be activated at a time (they all tap the permanent, CR 118.3).
//!
//! The payment planner used to keep a single {T} mana ability per permanent, so a
//! Volcanic Island could pay only {U} (its first intrinsic ability): Lightning Bolt
//! couldn't be cast with it, and a painland could pay only {C}.

use crate::r300_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj(id).tapped
}

fn pool(t: &TestGame, ty: ManaType) -> usize {
    t.player(P0).mana_pool.count(ty)
}

#[test]
fn a_dual_land_pays_with_either_of_its_basic_land_types_mana_abilities() {
    cr!("305.6", "601.2g", "601.2h");
    ruling!(
        "Volcanic Island",
        "This has the mana abilities associated with both of its basic land types"
    );
    // {R} from a Volcanic Island: its Mountain ability.
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Volcanic Island");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(can_cast(&mut t, P0, bolt));
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(tapped(&t, island));
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // {U} from a Volcanic Island: its Island ability.
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Volcanic Island");
    let opt = t.hand(P0, "Opt");
    assert!(can_cast(&mut t, P0, opt));
    t.cast(P0, opt).go();
    assert!(tapped(&t, island));
    assert_eq!(t.stack_len(), 1);
    // Likewise Underground Sea's {B}: Dark Ritual nets {B}{B}{B}.
    let mut t = TestGame::new(2);
    let sea = t.battlefield(P0, "Underground Sea");
    let ritual = t.hand(P0, "Dark Ritual");
    t.cast(P0, ritual).go();
    t.resolve();
    assert!(tapped(&t, sea));
    assert_eq!(pool(&t, ManaType::B), 3);
}

#[test]
fn tapping_a_dual_land_for_mana_activates_one_of_its_abilities() {
    cr!("118.3", "305.6");
    // Tapping it for mana is activating one of its {T} mana abilities, chosen by the
    // player; afterwards the other can't be activated (it's tapped).
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Volcanic Island");
    let abilities = t.obj(island).chars.abilities.len();
    assert_eq!(abilities, 2, "{{T}}: Add {{U}} and {{T}}: Add {{R}}");
    t.activate(P0, island, 1, &[]).unwrap();
    assert!(tapped(&t, island));
    assert_eq!(pool(&t, ManaType::R), 1);
    assert_eq!(pool(&t, ManaType::U), 0);
    assert!(t.activate(P0, island, 0, &[]).is_err());
    assert_eq!(t.player(P0).mana_pool.total(), 1);
}

#[test]
fn one_dual_land_cant_pay_both_of_its_colors() {
    cr!("118.3", "305.6", "601.2h");
    // Stormchaser Mage costs {U}{R}: a single Volcanic Island taps for one or the other,
    // and a tapped permanent can't be tapped again to pay a cost.
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Volcanic Island");
    let mage = t.hand(P0, "Stormchaser Mage");
    assert!(!can_cast(&mut t, P0, mage));
    assert!(t.cast(P0, mage).try_go().is_err());
    assert_eq!(t.zone(mage), Zone::Hand(P0));
    assert!(!tapped(&t, island), "no partial payment");
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // It can pay for one spell only: after Lightning Bolt, Opt can't be cast.
    let bolt = t.hand(P0, "Lightning Bolt");
    let opt = t.hand(P0, "Opt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_cast(&mut t, P0, opt));
    assert!(t.cast(P0, opt).try_go().is_err());
    // Three dual lands make three mana, not six.
    let mut t = TestGame::new(2);
    t.lands(P0, "Volcanic Island", 3);
    assert_eq!(t.g.max_mana_available(P0), 3);
}

#[test]
fn two_dual_lands_pay_a_two_color_cost() {
    cr!("305.6", "601.2g", "601.2h");
    let mut t = TestGame::new(2);
    let islands = t.lands(P0, "Volcanic Island", 2);
    let mage = t.hand(P0, "Stormchaser Mage");
    assert!(can_cast(&mut t, P0, mage));
    t.cast(P0, mage).go();
    assert!(islands.iter().all(|i| tapped(&t, *i)));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Stormchaser Mage").len(), 1);
    // {1}{U}{R} with three of them.
    let mut t = TestGame::new(2);
    let islands = t.lands(P0, "Volcanic Island", 3);
    let drake = t.hand(P0, "Enigma Drake");
    assert!(can_cast(&mut t, P0, drake));
    t.cast(P0, drake).go();
    assert!(islands.iter().all(|i| tapped(&t, *i)));
}

#[test]
fn dual_lands_and_basic_lands_pay_together() {
    cr!("305.6", "601.2g", "601.2h");
    // Lightning Strike ({1}{R}) with an Island and a Volcanic Island: the Volcanic Island
    // has to make the {R}, the Island the {1}.
    let mut t = TestGame::new(2);
    let basic = t.battlefield(P0, "Island");
    let dual = t.battlefield(P0, "Volcanic Island");
    let strike = t.hand(P0, "Lightning Strike");
    assert!(can_cast(&mut t, P0, strike));
    t.cast(P0, strike).target(Entity::Player(P1)).go();
    assert!(tapped(&t, basic) && tapped(&t, dual));
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // With a Mountain and a Volcanic Island, Lightning Bolt taps the Mountain and leaves
    // the Volcanic Island's {U} for Opt.
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    let dual = t.battlefield(P0, "Volcanic Island");
    let bolt = t.hand(P0, "Lightning Bolt");
    let opt = t.hand(P0, "Opt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(tapped(&t, mountain) && !tapped(&t, dual));
    assert!(can_cast(&mut t, P0, opt));
    t.cast(P0, opt).go();
    assert!(tapped(&t, dual));
    // Stormchaser Mage with a Mountain and a Volcanic Island: only the island makes {U}.
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    let dual = t.battlefield(P0, "Volcanic Island");
    let mage = t.hand(P0, "Stormchaser Mage");
    t.cast(P0, mage).go();
    assert!(tapped(&t, mountain) && tapped(&t, dual));
}

#[test]
fn the_payment_chooses_which_ability_of_each_dual_land_to_activate() {
    cr!("305.6", "601.2g", "601.2h");
    // Lightning Helix ({R}{W}) with a Plateau (R/W) and a Taiga (R/G): the Taiga must make
    // the {R} and the Plateau the {W}, whichever is considered first.
    for plateau_first in [true, false] {
        let mut t = TestGame::new(2);
        let (plateau, taiga) = if plateau_first {
            let p = t.battlefield(P0, "Plateau");
            (p, t.battlefield(P0, "Taiga"))
        } else {
            let tg = t.battlefield(P0, "Taiga");
            (t.battlefield(P0, "Plateau"), tg)
        };
        let helix = t.hand(P0, "Lightning Helix");
        assert!(can_cast(&mut t, P0, helix));
        t.cast(P0, helix).target(Entity::Player(P1)).go();
        assert!(tapped(&t, plateau) && tapped(&t, taiga));
        t.resolve();
        assert_eq!((t.life(P0), t.life(P1)), (23, 17));
    }
    // Sliver Queen ({W}{U}{B}{R}{G}) with a ring of five dual lands, each sharing one
    // color with the next: only two of the 32 ways to tap them pay the cost.
    let mut t = TestGame::new(2);
    let duals: Vec<ObjectId> = [
        "Tundra",
        "Underground Sea",
        "Badlands",
        "Taiga",
        "Savannah",
    ]
    .iter()
    .map(|n| t.battlefield(P0, n))
    .collect();
    let queen = t.hand(P0, "Sliver Queen");
    assert!(can_cast(&mut t, P0, queen));
    t.cast(P0, queen).go();
    assert!(duals.iter().all(|d| tapped(&t, *d)));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Sliver Queen").len(), 1);
    // Five lands that make every color between them, but only the Tundra makes {W} or
    // {U}: it can't pay both.
    let mut t = TestGame::new(2);
    let lands: Vec<ObjectId> = ["Tundra", "Badlands", "Taiga", "Bayou", "Mountain"]
        .iter()
        .map(|n| t.battlefield(P0, n))
        .collect();
    let queen = t.hand(P0, "Sliver Queen");
    assert!(!can_cast(&mut t, P0, queen));
    assert!(t.cast(P0, queen).try_go().is_err());
    assert!(lands.iter().all(|l| !tapped(&t, *l)));
}

#[test]
fn a_nonbasic_dual_land_under_blood_moon_makes_only_red() {
    cr!("305.6", "305.7");
    ruling!(
        "Blood Moon",
        "They will gain the land type Mountain and gain the ability \"{T}: Add {R}.\""
    );
    let mut t = TestGame::new(2);
    let sea = t.battlefield(P0, "Underground Sea");
    t.battlefield(P1, "Blood Moon");
    let ritual = t.hand(P0, "Dark Ritual");
    let opt = t.hand(P0, "Opt");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!can_cast(&mut t, P0, ritual));
    assert!(!can_cast(&mut t, P0, opt));
    assert!(t.cast(P0, ritual).try_go().is_err());
    assert!(can_cast(&mut t, P0, bolt));
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(tapped(&t, sea));
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_painland_pays_colored_mana_with_its_second_ability() {
    cr!("601.2g", "605.3b");
    ruling!(
        "Shivan Reef",
        "The damage dealt to you is part of the second mana ability"
    );
    // Shivan Reef: "{T}: Add {C}." and "{T}: Add {U} or {R}. This land deals 1 damage to
    // you." Lightning Bolt needs the second; the damage is dealt as the mana is made,
    // while Bolt is being cast.
    let mut t = TestGame::new(2);
    let reef = t.battlefield(P0, "Shivan Reef");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(can_cast(&mut t, P0, bolt));
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(tapped(&t, reef));
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P0), 19);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Generic mana is paid with the painless {C} ability.
    let mut t = TestGame::new(2);
    let reef = t.battlefield(P0, "Shivan Reef");
    let ring = t.hand(P0, "Sol Ring");
    t.cast(P0, ring).go();
    assert!(tapped(&t, reef));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_land_whose_mana_abilities_both_tap_it_pays_with_the_one_that_suffices() {
    cr!("118.3", "601.2g", "601.2h");
    // Crystal Vein: "{T}: Add {C}." and "{T}, Sacrifice this land: Add {C}{C}." Only one
    // can be activated (both tap it). {2} needs the second; {1} doesn't.
    let mut t = TestGame::new(2);
    let vein = t.battlefield(P0, "Crystal Vein");
    let stone = t.hand(P0, "Mind Stone");
    assert!(can_cast(&mut t, P0, stone));
    t.cast(P0, stone).go();
    assert_eq!(t.zone(vein), Zone::Graveyard(P0));
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    let mut t = TestGame::new(2);
    let vein = t.battlefield(P0, "Crystal Vein");
    let ring = t.hand(P0, "Sol Ring");
    t.cast(P0, ring).go();
    assert!(t.on_battlefield(vein) && tapped(&t, vein));
    // With a Wastes too: {3} can be paid, {4} can't (the Vein makes two mana at most).
    let mut t = TestGame::new(2);
    let vein = t.battlefield(P0, "Crystal Vein");
    let wastes = t.battlefield(P0, "Wastes");
    let dynamo = t.hand(P0, "Thran Dynamo");
    assert!(!can_cast(&mut t, P0, dynamo));
    assert!(t.cast(P0, dynamo).try_go().is_err());
    assert!(!tapped(&t, vein) && !tapped(&t, wastes));
    let powerstone = t.hand(P0, "Worn Powerstone");
    assert!(can_cast(&mut t, P0, powerstone));
    t.cast(P0, powerstone).go();
    assert_eq!(t.zone(vein), Zone::Graveyard(P0));
    assert!(tapped(&t, wastes));
}

#[test]
fn tapped_for_mana_triggers_follow_the_dual_lands_chosen_ability() {
    cr!("106.12", "106.12a", "118.3a", "605.4a");
    // Wild Growth on a Volcanic Island: tapping it for {R} also adds {G}. Rip-Clan Crasher
    // ({R}{G}) can be cast with that one land.
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Volcanic Island");
    let growth = t.hand(P0, "Wild Growth");
    t.battlefield(P0, "Forest");
    t.cast(P0, growth).target(island).go();
    t.resolve();
    let crasher = t.hand(P0, "Rip-Clan Crasher");
    assert!(can_cast(&mut t, P0, crasher));
    t.cast(P0, crasher).go();
    assert!(tapped(&t, island));
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // Lightning Bolt with it: the extra {G} is left in the pool once the cost is paid.
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Volcanic Island");
    let growth = t.hand(P0, "Wild Growth");
    t.battlefield(P0, "Forest");
    t.cast(P0, growth).target(island).go();
    t.resolve();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(tapped(&t, island));
    assert_eq!(pool(&t, ManaType::G), 1);
    assert_eq!(pool(&t, ManaType::R), 0);
}

#[test]
fn a_replacement_for_tapping_for_mana_changes_what_a_dual_land_pays_with() {
    cr!("106.12b", "305.6");
    ruling!(
        "Contamination",
        "The second ability of this card is a replacement effect, not a triggered ability"
    );
    // Contamination: "If a land is tapped for mana, it produces {B} instead of any other
    // type and amount." A Volcanic Island then pays for Dark Ritual but not Lightning Bolt.
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Volcanic Island");
    t.battlefield(P1, "Contamination");
    let bolt = t.hand(P0, "Lightning Bolt");
    let ritual = t.hand(P0, "Dark Ritual");
    assert!(!can_cast(&mut t, P0, bolt));
    assert!(t.cast(P0, bolt).target(Entity::Player(P1)).try_go().is_err());
    assert!(!tapped(&t, island));
    assert!(can_cast(&mut t, P0, ritual));
    t.cast(P0, ritual).go();
    assert!(tapped(&t, island));
    t.resolve();
    assert_eq!(pool(&t, ManaType::B), 3);
}

#[test]
fn a_cost_larger_than_the_mana_the_lands_can_make_cant_be_paid() {
    cr!("118.3", "601.2h");
    // Twelve dual lands make twelve mana: Emrakul, the Aeons Torn ({15}) can't be cast,
    // and finding that out doesn't try every order of tapping them.
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Volcanic Island", 12);
    let emrakul = t.hand(P0, "Emrakul, the Aeons Torn");
    let start = std::time::Instant::now();
    assert!(!can_cast(&mut t, P0, emrakul));
    assert!(t.cast(P0, emrakul).try_go().is_err());
    assert!(start.elapsed() < std::time::Duration::from_secs(5));
    assert!(lands.iter().all(|l| !tapped(&t, *l)));
    // Blightsteel Colossus ({12}) can.
    let colossus = t.hand(P0, "Blightsteel Colossus");
    assert!(can_cast(&mut t, P0, colossus));
    t.cast(P0, colossus).go();
    assert!(lands.iter().all(|l| tapped(&t, *l)));
}

#[test]
fn a_doubled_mana_ability_makes_more_of_the_one_type_chosen_for_it() {
    cr!("106.12b", "601.2g", "601.2h");
    ruling!(
        "Mana Reflection",
        "you'll get four times the original amount and type of mana"
    );
    // Mana Reflection: "If you tap a permanent for mana, it produces twice as much of that
    // mana instead." Shivan Reef's second ability then makes {U}{U} or {R}{R}: one Reef
    // pays for Lord of Atlantis ({U}{U}) but not Stormchaser Mage ({U}{R}).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Reflection");
    let reef = t.battlefield(P0, "Shivan Reef");
    let mage = t.hand(P0, "Stormchaser Mage");
    assert!(!can_cast(&mut t, P0, mage));
    assert!(t.cast(P0, mage).try_go().is_err());
    assert!(!tapped(&t, reef));
    let lord = t.hand(P0, "Lord of Atlantis");
    assert!(can_cast(&mut t, P0, lord));
    t.cast(P0, lord).go();
    assert!(tapped(&t, reef));
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // A Volcanic Island's two abilities make {U}{U} or {R}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Reflection");
    let island = t.battlefield(P0, "Volcanic Island");
    let mage = t.hand(P0, "Stormchaser Mage");
    assert!(!can_cast(&mut t, P0, mage));
    let zealot = t.hand(P0, "Ash Zealot");
    assert!(can_cast(&mut t, P0, zealot));
    t.cast(P0, zealot).go();
    assert!(tapped(&t, island));
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // Birds of Paradise ("one mana of any color") likewise makes two of one color.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Reflection");
    let birds = t.battlefield(P0, "Birds of Paradise");
    let mage = t.hand(P0, "Stormchaser Mage");
    assert!(!can_cast(&mut t, P0, mage));
    let lord = t.hand(P0, "Lord of Atlantis");
    assert!(can_cast(&mut t, P0, lord));
    t.cast(P0, lord).go();
    assert!(tapped(&t, birds));
}

#[test]
fn colors_that_only_a_few_lands_make_between_them_are_counted_together() {
    cr!("118.3", "601.2h");
    // Progenitus ({W}{W}{U}{U}{B}{B}{R}{R}{G}{G}) with ten Tundras, three Badlands and
    // four Forests: every color has enough lands on its own, but {B}{B}{R}{R} needs four
    // Badlands. It can't be cast, and finding that out doesn't try every way of tapping
    // the Tundras first.
    let mut t = TestGame::new(2);
    let mut lands = t.lands(P0, "Tundra", 10);
    lands.extend(t.lands(P0, "Badlands", 3));
    lands.extend(t.lands(P0, "Forest", 4));
    let progenitus = t.hand(P0, "Progenitus");
    let start = std::time::Instant::now();
    assert!(!can_cast(&mut t, P0, progenitus));
    assert!(t.cast(P0, progenitus).try_go().is_err());
    assert!(start.elapsed() < std::time::Duration::from_secs(5));
    assert!(lands.iter().all(|l| !tapped(&t, *l)));
    // With a fourth Badlands it can.
    let more = t.battlefield(P0, "Badlands");
    assert!(can_cast(&mut t, P0, progenitus));
    t.cast(P0, progenitus).go();
    assert!(tapped(&t, more));
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}
