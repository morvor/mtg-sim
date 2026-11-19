//! Rulings batch P103 — the "meme" cards: Panglacial Wurm (cast while searching your own
//! library, CR 701.23), Fact or Fiction's piles (CR 700.3d), Bone Flute (CR 611.2c),
//! Twiddle (a choice made on resolution, CR 608.2d), and Asmoranomardicadaistinaculdacar
//! (a creature dealing damage to itself, CR 120.3, 702.2b, 702.15b).

use crate::r_p103_common::*;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s02_common::create_token;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Number of "cast Panglacial Wurm from your library?" questions asked since `from`.
fn wurm_offers(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("Panglacial Wurm")))
        .count()
}

#[test]
fn panglacial_wurm_only_while_searching_your_own_library() {
    cr!("701.23a", "701.23b");
    ruling!(
        "Panglacial Wurm",
        "Panglacial Wurm's ability works only while you're searching your own library. The effect that caused you to search needs to say \"search\" and \"library,\" and you need to be looking through your own library for this to work."
    );
    supported("Panglacial Wurm");
    // (spell, its targets, whether P0 is offered the Wurm)
    type Setup = fn(&mut TestGame) -> Vec<Entity>;
    let cases: Vec<(&str, Setup, bool)> = vec![
        // Rampant Growth and Gifts Ungiven search your library.
        ("Rampant Growth", |_| vec![], true),
        ("Gifts Ungiven", |_| vec![pl(P1)], true),
        // Bribery searches an opponent's library.
        ("Bribery", |t| {
            t.library_top(P1, "Grizzly Bears");
            vec![pl(P1)]
        }, false),
        // Sage Owl looks at the top of your library without searching it.
        ("Sage Owl", |_| vec![], false),
    ];
    for (name, setup, offered) in cases {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Forest");
        t.library_top(P0, "Panglacial Wurm");
        t.lands(P0, "Forest", 7);
        let targets = setup(&mut t);
        let from = t.asked().len();
        t.answer_yes(P0, false);
        cast_new(&mut t, P0, name, &targets);
        t.resolve_all();
        assert_eq!(wurm_offers(&t, from) > 0, offered, "{name}");
    }
}

#[test]
fn panglacial_wurm_search_keeps_the_library_order() {
    cr!("701.23a", "401.2");
    ruling!(
        "Panglacial Wurm",
        "While searching your library, you must keep your library in the same order until you shuffle it."
    );
    supported("Panglacial Wurm");
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, "Island", 0);
    stack_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Panglacial Wurm", "Forest", "Hill Giant", "Island"],
    );
    let before = t.g.player(P0).library.clone();
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("Panglacial Wurm")),
        |g| g.player(P0).library.clone(),
    );
    t.lands(P0, "Forest", 9);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Rampant Growth", &[]);
    t.resolve();
    // Asked mid-search, with the library in its original order.
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen, vec![before]);
    // The Wurm was cast from it (it's on the stack).
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn fact_or_fiction_piles_may_be_empty() {
    cr!("700.3d");
    ruling!("Fact or Fiction", "Piles may be empty.");
    supported("Fact or Fiction");
    let mut t = TestGame::new(2);
    let cards = stack_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Hill Giant", "Shock", "Island", "Ornithopter"],
    );
    // P1 puts all five cards in one pile; P0 takes the other (empty) pile.
    t.answer_choose(P1, &cards.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Fact or Fiction", &[]);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand);
    for c in cards {
        assert_eq!(t.zone(c), Zone::Graveyard(P0));
    }
}

#[test]
fn bone_flute_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Bone Flute",
        "The ability affects only creatures on the battlefield at the time it resolves. A creature that enters later in the turn won't get -1/-0."
    );
    supported("Bone Flute");
    let mut t = TestGame::new(2);
    let flute = t.battlefield(P0, "Bone Flute");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, flute, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(mine), (1, 2));
    assert_eq!(t.pt(theirs), (2, 3));
    let late = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
}

/// P0 casts Twiddle at `target`; returns the number of Twiddle's tap/untap choices asked
/// before it resolves and after.
fn twiddle(t: &mut TestGame, target: ObjectId, choice: usize, between: fn(&mut TestGame)) -> (usize, usize) {
    supported("Twiddle");
    let options = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseOption { .. } | Decision::ChooseModes { .. }))
            .count()
    };
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(choice));
    cast_new(t, P0, "Twiddle", &[obj(target)]);
    let before = options(t, from);
    between(t);
    let from = t.asked().len();
    t.resolve();
    (before, options(t, from))
}

#[test]
fn twiddle_chooses_tap_or_untap_as_it_resolves() {
    cr!("608.2d", "700.2");
    ruling!(
        "Twiddle",
        "The decision whether or not to tap or untap is made on resolution. This is not a modal spell."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let (before, after) = twiddle(&mut t, bears, 0, |_| {});
    assert_eq!((before, after), (0, 1));
    assert!(t.obj_now(bears).tapped);
    // Untap.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    twiddle(&mut t, bears, 1, |_| {});
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn twiddle_doesnt_toggle() {
    cr!("608.2d", "701.26a");
    ruling!(
        "Twiddle",
        "This is not a toggle effect. If you use Twiddle to tap a card and before it takes effect your opponent taps it, Twiddle will not automatically untap the card."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P0 means to tap the Bears; P1 taps them first.
    twiddle(&mut t, bears, 0, |t| {
        let b = t.named_on_battlefield("Grizzly Bears")[0];
        t.g.tap(b);
    });
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn asmoranomardicadaistinaculdacar_makes_the_creature_the_source() {
    cr!("120.3", "702.2b", "702.15b", "704.5h");
    ruling!(
        "Asmoranomardicadaistinaculdacar",
        "The target creature of Asmoranomardicadaistinaculdacar's last ability is the source of the damage, not Asmoranomardicadaistinaculdacar itself. If the creature has deathtouch or lifelink, both will apply to the damage it deals to itself."
    );
    let mut t = TestGame::new(2);
    let asmor = t.battlefield(P0, "Asmoranomardicadaistinaculdacar");
    // Vampire Nighthawk (flying, deathtouch, lifelink) as a 7/8: 6 damage isn't lethal
    // damage, but it's deathtouch damage.
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    t.g.objects[hawk.0 as usize]
        .counters
        .insert("+1/+1".into(), 5);
    t.g.recompute();
    assert_eq!(t.pt(hawk), (7, 8));
    create_token(&mut t, P0, "Food");
    create_token(&mut t, P0, "Food");
    t.answer_choose(P0, &[]);
    t.activate(P0, asmor, 0, &[obj(hawk)]).unwrap();
    t.resolve_all();
    let dmg = crate::r_s30_common::damage_events(&t);
    assert_eq!(dmg, vec![(hawk, obj(hawk), 6, false)]);
    assert_eq!(t.life(P1), 26, "lifelink");
    assert_eq!(t.zone(hawk), Zone::Graveyard(P1), "deathtouch");
    assert_eq!(t.life(P0), 20);
}
