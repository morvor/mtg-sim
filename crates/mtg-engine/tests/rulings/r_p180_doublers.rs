//! Rulings batch P180 — trigger doublers (CR 603.2d): Panharmonicon, Yarok, the
//! Desecrated, Starfield Vocalist, Delney, Streetwise Lookout and Veyran, Voice of
//! Duality. Each additional instance makes its own choices; replacement effects aren't
//! doubled.

use crate::r_p180_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn doubled_enters_triggers_choose_targets_and_resolution_choices_separately() {
    cr!("603.2d", "603.3d");
    ruling!(
        "Panharmonicon",
        "Panharmonicon's effect doesn't copy the triggered ability; it just causes the ability to trigger twice."
    );
    ruling!(
        "Starfield Vocalist",
        "Starfield Vocalist’s first ability doesn’t copy the triggered ability; it just causes the ability to trigger an additional time."
    );
    ruling!(
        "Yarok, the Desecrated",
        "Yarok's effect doesn't copy the triggered ability; it just causes the ability to trigger twice."
    );
    for doubler in ["Panharmonicon", "Starfield Vocalist", "Yarok, the Desecrated"] {
        supported(doubler);
        // Targets: Satyr Grovedancer ("When this creature enters, put a +1/+1 counter on
        // target creature.") triggers twice; each instance targets a different creature.
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(a)]);
        t.answer_targets(P0, &[obj(b)]);
        t.enter(P0, "Satyr Grovedancer");
        t.settle();
        assert_eq!(t.stack_len(), 2, "{doubler}");
        t.resolve_all();
        assert_eq!((plus(&t, a), plus(&t, b)), (1, 1), "{doubler}");
        // Resolution choices: Sylvan Ranger's "you may search" is answered per instance.
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        stack_library(&mut t, P0, &["Forest", "Forest"]);
        t.answer_yes(P0, true);
        t.answer_yes(P0, false);
        let hand = t.hand_size(P0);
        t.enter(P0, "Sylvan Ranger");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{doubler}");
        assert!(t.in_hand(P0, "Forest"), "{doubler}");
    }
}

#[test]
fn veyran_doubled_magecraft_triggers_target_separately() {
    cr!("603.2d", "603.3d");
    ruling!(
        "Veyran, Voice of Duality",
        "Veyran's effect doesn't copy the triggered ability; it just causes the ability to trigger again."
    );
    supported("Veyran, Voice of Duality");
    supported("Karok Wrangler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Veyran, Voice of Duality");
    // "Magecraft — Whenever you cast or copy an instant or sorcery spell, put a +1/+1
    // counter on target creature you control."
    let wrangler = t.battlefield(P0, "Karok Wrangler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_targeting(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.answer_targets(P0, &[obj(wrangler)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!((plus(&t, wrangler), plus(&t, bears)), (1, 1));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn trigger_doublers_dont_double_enters_with_counters_replacements() {
    cr!("603.2d", "614.1c");
    ruling!(
        "Panharmonicon",
        "Replacement effects are unaffected by Panharmonicon's ability."
    );
    ruling!(
        "Yarok, the Desecrated",
        "Replacement effects are unaffected by Yarok's ability."
    );
    ruling!(
        "Starfield Vocalist",
        "Replacement effects are unaffected by Starfield Vocalist’s first ability."
    );
    ruling!(
        "Delney, Streetwise Lookout",
        "Replacement effects are unaffected by Delney, Streetwise Lookout's last ability."
    );
    supported("Festercreep");
    for doubler in [
        "Panharmonicon",
        "Yarok, the Desecrated",
        "Starfield Vocalist",
        "Delney, Streetwise Lookout",
    ] {
        supported(doubler);
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        // Festercreep: "This creature enters with a +1/+1 counter on it."
        let creep = t.enter(P0, "Festercreep");
        t.settle();
        assert_eq!(plus(&t, creep), 1, "{doubler}");
        assert_eq!(t.pt(creep), (1, 1), "{doubler}");
        // The doubler is active: a 1/1's enters trigger triggers twice.
        t.answer_targets(P0, &[obj(creep)]);
        t.answer_targets(P0, &[obj(creep)]);
        t.enter(P0, "Satyr Grovedancer");
        t.resolve_all();
        assert_eq!(plus(&t, creep), 3, "{doubler}");
    }
}

#[test]
fn starfield_vocalist_doubles_own_others_and_keyword_enters_triggers() {
    cr!("603.2d", "603.6a", "702.123a");
    ruling!(
        "Starfield Vocalist",
        "Starfield Vocalist’s first ability affects a permanent’s own “enters” triggered abilities as well as other triggered abilities"
    );
    supported("Starfield Vocalist");
    supported("Glint-Sleeve Artisan");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starfield Vocalist");
    t.battlefield(P0, "Soul Warden");
    let (hand, life) = (t.hand_size(P0), t.life(P0));
    // Its own "when this creature enters" ability, and another permanent's.
    t.enter(P0, "Elvish Visionary");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), life + 2);
    // A keyword's enters trigger: fabricate 1 triggers twice (two counters by default,
    // or Servos).
    let artisan = t.enter(P0, "Glint-Sleeve Artisan");
    t.resolve_all();
    let servos = t.named_on_battlefield("Servo").len() as u32;
    assert_eq!(plus(&t, artisan) + servos, 2);
    // Any permanent entering, not just an artifact or creature.
    t.battlefield(P0, "Entity Tracker");
    let hand = t.hand_size(P0);
    t.enter(P0, "Glorious Anthem");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}
