//! Rulings on surge (CR 702.117): a discount only, or extra effects if its cost was paid.

use crate::r_s01_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const SURGE: CastMethod = CastMethod::Keyword(KeywordKind::Surge);

/// P0 casts a Lightning Bolt at P1 and it resolves.
fn cast_another_spell(t: &mut TestGame) {
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b).target(P1).go();
    t.resolve_all();
}

#[test]
fn surge_is_a_discount_for_some_cards_and_gives_other_cards_extra_effects() {
    cr!("702.117a");
    ruling!(
        "Reckless Bushwhacker",
        "For some cards, surge represents only an alternative cost, a discount that applies if you or a teammate has cast another spell this turn. Other cards, like Reckless Bushwhacker, have additional abilities or effects if you paid the surge cost to cast the spell."
    );
    supported("Goblin Freerunner");
    supported("Reckless Bushwhacker");
    // Goblin Freerunner ({3}{R} 3/2 menace, surge {1}{R}): only a discount. Cast either
    // way, it's the same creature.
    let freerunner = |surge: bool| {
        let mut t = TestGame::new(2);
        cast_another_spell(&mut t);
        let lands = t.lands(P0, "Mountain", 4);
        let c = t.hand(P0, "Goblin Freerunner");
        let method = if surge { SURGE } else { CastMethod::Normal };
        t.cast(P0, c).method(method).go();
        let paid = lands.iter().filter(|l| t.obj_now(**l).tapped).count();
        t.resolve_all();
        let o = t.obj_now(c);
        assert!(t.on_battlefield(c));
        (paid, o.power(), o.toughness(), o.chars.has_keyword(KeywordKind::Menace))
    };
    assert_eq!(freerunner(false), (4, 3, 2, true));
    assert_eq!(freerunner(true), (2, 3, 2, true));
    // Reckless Bushwhacker ({2}{R}, surge {1}{R}): "When this creature enters, if its
    // surge cost was paid, other creatures you control get +1/+0 and gain haste until end
    // of turn."
    let bushwhacker = |surge: bool| {
        let mut t = TestGame::new(2);
        cast_another_spell(&mut t);
        let bears = t.battlefield_sick(P0, "Grizzly Bears");
        t.lands(P0, "Mountain", 3);
        let c = t.hand(P0, "Reckless Bushwhacker");
        let method = if surge { SURGE } else { CastMethod::Normal };
        t.cast(P0, c).method(method).go();
        t.resolve_all();
        (
            t.pt(bears),
            t.obj_now(bears).chars.has_keyword(KeywordKind::Haste),
        )
    };
    assert_eq!(bushwhacker(false), ((2, 2), false));
    assert_eq!(bushwhacker(true), ((3, 2), true));
}
