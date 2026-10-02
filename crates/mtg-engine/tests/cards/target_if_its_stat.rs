//! "[effect] target [object] if its [mana value / power / toughness] is [N or a value]"
//! (checked as the effect resolves, CR 608.2c), and the Shoals' alternative cost "exile a
//! [color] card with mana value X from your hand" (CR 107.3a, 118.9).

use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn rending_vines_destroys_only_a_permanent_with_small_enough_mana_value() {
    cr!("608.2c");
    compiles("Rending Vines");
    // "Destroy target artifact or enchantment if its mana value is less than or equal to
    // the number of cards in your hand. Draw a card." Two cards in hand as it resolves:
    // Glorious Anthem (3) survives; Ornithopter (0) doesn't. P0 draws either way.
    for (target, destroyed) in [("Glorious Anthem", false), ("Ornithopter", true)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 3);
        let vines = t.hand(P0, "Rending Vines");
        t.hand(P0, "Grizzly Bears");
        t.hand(P0, "Grizzly Bears");
        let victim = t.battlefield(P1, target);
        t.cast(P0, vines).target(victim).go();
        t.resolve_all();
        assert_eq!(t.in_graveyard(P1, target), destroyed, "{target}");
        assert_eq!(t.hand_size(P0), 3);
    }
}

#[test]
fn ghastly_demise_compares_toughness_with_the_graveyard_size() {
    cr!("608.2c");
    compiles("Ghastly Demise");
    // "Destroy target nonblack creature if its toughness is less than or equal to the
    // number of cards in your graveyard." Two cards in P0's graveyard: Grizzly Bears (2/2)
    // is destroyed, Hill Giant (3/3) isn't.
    for (target, destroyed) in [("Grizzly Bears", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 1);
        t.graveyard(P0, "Shock");
        t.graveyard(P0, "Opt");
        let demise = t.hand(P0, "Ghastly Demise");
        let victim = t.battlefield(P1, target);
        t.cast(P0, demise).target(victim).go();
        t.resolve_all();
        assert_eq!(t.in_graveyard(P1, target), destroyed, "{target}");
    }
}

#[test]
fn prismatic_ending_exiles_up_to_the_number_of_colors_spent() {
    cr!("608.2c", "207.2c");
    compiles("Prismatic Ending");
    // Prismatic Ending ({X}{W}) cast with X = 1 paid with {W}{R}: two colors were spent.
    // Grizzly Bears (2) is exiled; Hill Giant (4) isn't.
    for (target, exiled) in [("Grizzly Bears", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Mountain", 1);
        let ending = t.hand(P0, "Prismatic Ending");
        let victim = t.battlefield(P1, target);
        t.cast(P0, ending).x(1).target(victim).go();
        t.resolve_all();
        assert_eq!(t.in_exile(target), exiled, "{target}");
    }
}

#[test]
fn dispersal_shield_compares_with_the_greatest_mana_value_you_control() {
    cr!("608.2c", "701.6a");
    compiles("Dispersal Shield");
    // "Counter target spell if its mana value is less than or equal to the greatest mana
    // value among permanents you control." P0 controls Hill Giant (4): P1's Grizzly Bears
    // (2) is countered, P1's Craw Wurm (6) isn't.
    for (spell, countered) in [("Grizzly Bears", true), ("Craw Wurm", false)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.battlefield(P0, "Hill Giant");
        t.lands(P0, "Island", 2);
        t.lands(P1, "Forest", 6);
        let card = t.hand(P1, spell);
        let s = t.cast(P1, card).go();
        let shield = t.hand(P0, "Dispersal Shield");
        t.cast(P0, shield).target(s).go();
        t.resolve_all();
        assert_eq!(t.in_graveyard(P1, spell), countered, "{spell}");
    }
}

#[test]
fn sage_eye_avengers_returns_a_creature_with_less_power() {
    cr!("608.2c", "603.3c");
    compiles("Sage-Eye Avengers");
    // "Whenever this creature attacks, you may return target creature to its owner's hand
    // if its power is less than this creature's power." Sage-Eye Avengers is 4/5: Hill
    // Giant (3) returns, Craw Wurm (6) doesn't.
    for (target, returned) in [("Hill Giant", true), ("Craw Wurm", false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let sage = t.battlefield(P0, "Sage-Eye Avengers");
        let victim = t.battlefield(P1, target);
        t.answer_targets(P0, &[Entity::Object(victim)]);
        t.answer_yes(P0, true);
        t.attack(&[(sage, Entity::Player(P1))], &[]);
        assert_eq!(t.in_hand(P1, target), returned, "{target}");
    }
}

#[test]
fn sickening_shoal_exiles_a_black_card_for_x() {
    cr!("107.3a", "118.9", "601.2b", "601.2h");
    compiles("Sickening Shoal");
    // "You may exile a black card with mana value X from your hand rather than pay this
    // spell's mana cost. Target creature gets -X/-X until end of turn." Exiling Phyrexian
    // Obliterator ({B}{B}{B}{B}) with no lands: X is 4, and Craw Wurm (6/4) dies.
    let mut t = TestGame::new(2);
    let shoal = t.hand(P0, "Sickening Shoal");
    let obliterator = t.hand(P0, "Phyrexian Obliterator");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let alt = t
        .cast_options(P0, shoal)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the Shoal's alternative cost");
    t.answer_choose(P0, &[Entity::Object(obliterator)]);
    t.cast(P0, shoal).method(alt).x(4).target(wurm).go();
    assert!(t.in_exile("Phyrexian Obliterator"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}
