//! Rulings batch P162 — putting a land card onto the battlefield isn't playing a land
//! (CR 305.4), lands that return a land on entering, and additional land plays.

use crate::r_p162_common::*;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s02_common::can_play_land;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[derive(Clone, Copy)]
enum How {
    Spell,
    Tap,
    TapForTwo,
}

/// Puts `land` from `p`'s hand onto the battlefield with `card`'s effect.
fn put_with(t: &mut TestGame, p: PlayerId, card: &str, how: How, land: ObjectId) {
    t.answer_yes(p, true);
    t.answer_choose(p, &[Entity::Object(land)]);
    match how {
        How::Spell => {
            cast_resolve(t, p, card, &[]);
        }
        How::Tap | How::TapForTwo => {
            if matches!(how, How::TapForTwo) {
                t.lands(p, "Wastes", 2);
            }
            let src = t.battlefield(p, card);
            let idx = if matches!(how, How::TapForTwo) { 1 } else { 0 };
            t.activate(p, src, idx, &[]).unwrap();
            t.resolve_all();
        }
    }
}

#[test]
fn putting_a_land_onto_the_battlefield_is_not_playing_a_land() {
    cr!("305.4", "305.2", "305.3");
    ruling!(
        "Growth Spiral",
        "Growth Spiral's effect doesn't count as playing a land. It can put a land card onto the battlefield even if it's not your turn or if you've already played your land for the turn."
    );
    ruling!(
        "Llanowar Scout",
        "Llanowar Scout’s effect doesn’t count as playing a land. It can put a land card onto the battlefield even if you’ve already played as many lands as able this turn or if it’s not your turn."
    );
    ruling!(
        "Eureka Moment",
        "Putting a land card from your hand onto the battlefield is not the same as playing a land. You may do it even if you have already played a land on your turn, or if you're casting Eureka Moment on your opponent's turn."
    );
    ruling!(
        "Embrace the Paradox",
        "Putting a land card onto the battlefield with Embrace the Paradox doesn't count as playing a land. You can put a land card onto the battlefield this way even if you've already played a land for the turn."
    );
    ruling!(
        "Walking Atlas",
        "Putting a land onto the battlefield as a result of Walking Atlas’s ability isn’t the same as playing a land."
    );
    ruling!(
        "Terrain Generator",
        "Putting a land onto the battlefield this way does not count as your one land play for the turn."
    );
    let cases: &[(&str, How, bool)] = &[
        ("Growth Spiral", How::Spell, false),
        ("Llanowar Scout", How::Tap, false),
        ("Eureka Moment", How::Spell, false),
        ("Embrace the Paradox", How::Spell, true),
        ("Walking Atlas", How::Tap, false),
        ("Terrain Generator", How::TapForTwo, true),
    ];
    for &(card, how, tapped) in cases {
        supported(card);
        // On your turn, after you've played your land for the turn.
        let mut t = TestGame::new(2);
        let played = t.hand(P0, "Forest");
        t.play_land(P0, played).unwrap();
        assert_eq!(lands_played(&t, P0), 1);
        let land = t.hand(P0, "Island");
        let other = t.hand(P0, "Plains");
        assert!(!can_play_land(&mut t, P0, other), "{card}");
        put_with(&mut t, P0, card, how, land);
        assert_eq!(t.zone(land), Zone::Battlefield, "{card}");
        assert_eq!(t.obj_now(land).tapped, tapped, "{card}");
        assert_eq!(lands_played(&t, P0), 1, "{card}");
        // On an opponent's turn (all of these can be used at instant speed).
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let land = t.hand(P0, "Island");
        put_with(&mut t, P0, card, how, land);
        assert_eq!(t.zone(land), Zone::Battlefield, "{card} on P1's turn");
    }
}

#[test]
fn putting_a_land_onto_the_battlefield_first_still_leaves_the_land_play() {
    cr!("305.4", "305.2");
    ruling!(
        "Walking Atlas",
        "Similarly, putting a land onto the battlefield during your turn doesn’t preclude you from playing a land later in that turn."
    );
    ruling!(
        "Into the Wilds",
        "Putting a land onto the battlefield this way doesn’t count as playing a land. You’ll still be able to play a land during one of your main phases."
    );
    supported("Walking Atlas");
    supported("Into the Wilds");
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Island");
    put_with(&mut t, P0, "Walking Atlas", How::Tap, land);
    assert_eq!(t.zone(land), Zone::Battlefield);
    let forest = t.hand(P0, "Forest");
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).unwrap();
    // Into the Wilds puts the land onto the battlefield in the upkeep; the land play is
    // still available in the main phase.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Into the Wilds");
    let top = t.library_top(P0, "Mountain");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Battlefield);
    t.advance_to(P0, Step::PrecombatMain);
    let forest = t.hand(P0, "Forest");
    assert!(can_play_land(&mut t, P0, forest));
}

#[test]
fn into_the_wilds_leaves_a_nonland_card_on_top() {
    cr!("504.1");
    ruling!(
        "Into the Wilds",
        "If the card is not a land card, or you choose not to put it onto the battlefield, it will stay on top of your library. In most cases, you’ll then draw it during your draw step."
    );
    for (top_card, put) in [("Grizzly Bears", true), ("Mountain", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Into the Wilds");
        let top = stack_library(&mut t, P0, &[top_card])[0];
        t.set_step(P1, Step::End);
        t.answer_yes(P0, put);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.zone(top), Zone::Library(P0), "{top_card}");
        assert_eq!(t.g.player(P0).library.last(), Some(&top), "{top_card}");
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(t.zone(top), Zone::Hand(P0), "{top_card}");
    }
}

#[test]
fn guildless_commons_returns_itself_without_another_land() {
    cr!("603.3", "608.2b");
    ruling!(
        "Guildless Commons",
        "Guildless Commons will return itself to its owner's hand if you control no other lands when it enters the battlefield or if you choose to return it to its owner's hand."
    );
    supported("Guildless Commons");
    // Alone: it has to return itself.
    let mut t = TestGame::new(2);
    let gc = t.hand(P0, "Guildless Commons");
    t.play_land(P0, gc).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Guildless Commons"));
    // With another land: you may still choose it.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let gc = t.hand(P0, "Guildless Commons");
    t.play_land(P0, gc).unwrap();
    let gc_now = t.g.current(gc);
    t.answer_choose(P0, &[Entity::Object(gc_now)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Guildless Commons"));
    assert!(t.on_battlefield(forest));
}

#[test]
fn enter_the_unknown_land_play_follows_normal_timing_and_is_cumulative() {
    cr!("305.2", "305.2a", "305.2b");
    ruling!(
        "Enter the Unknown",
        "Enter the Unknown's effect allows you to play an additional land during your main phase. Doing so follows the normal timing rules for playing lands."
    );
    ruling!(
        "Enter the Unknown",
        "The effects of multiples of Enter the Unknown in the same turn are cumulative. They're also cumulative with other effects that let you play additional lands, such as the one from Wayward Swordtooth."
    );
    supported("Enter the Unknown");
    supported("Wayward Swordtooth");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Wayward Swordtooth");
    for _ in 0..2 {
        cast_resolve(&mut t, P0, "Enter the Unknown", &[Entity::Object(bears)]);
    }
    // 1 + 1 (Swordtooth) + 2 (two Enter the Unknown).
    let lands: Vec<ObjectId> = (0..5).map(|_| t.hand(P0, "Forest")).collect();
    for &l in &lands[..4] {
        assert!(can_play_land(&mut t, P0, l));
        t.play_land(P0, l).unwrap();
    }
    assert!(!can_play_land(&mut t, P0, lands[4]));
    // Normal timing: not while a spell is on the stack, nor outside a main phase.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Enter the Unknown", &[Entity::Object(bears)]);
    let a = t.hand(P0, "Forest");
    t.play_land(P0, a).unwrap();
    let b = t.hand(P0, "Forest");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, b));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, b));
    crate::r_s01_common::give_mana_for(&mut t, P0, "Lightning Bolt");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, b));
}

#[test]
fn walking_atlas_is_an_artifact_creature() {
    cr!("205.2a");
    ruling!(
        "Walking Atlas",
        "The word “artifact” was inadvertently omitted from Walking Atlas’s type line. The card has received errata to correct this omission; it is an artifact creature."
    );
    let mut t = TestGame::new(2);
    let atlas = t.battlefield(P0, "Walking Atlas");
    let o = t.obj_now(atlas);
    assert!(o.is(types::CardType::Artifact) && o.is(types::CardType::Creature));
}
