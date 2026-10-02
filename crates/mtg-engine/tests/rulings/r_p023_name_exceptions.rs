//! Rulings batch P023 — permanents that become a copy of a creature "except its name is
//! ~" (Sarkhan, Soul Aflame; Kimahri, Valiant Guardian; Sunfrill Imitator): copying a
//! token copies the original characteristics the effect that created it gave it and
//! doesn't make the permanent a token (nor stop a token being one); copying something
//! that's copying something else copies what it copies; {X} in the copied mana cost is 0
//! (CR 707.2, 707.3, 707.9b, 111.3, 107.3m, 202.3e).

use crate::r_p023_common::*;
use crate::r_s01_common::supported;
use crate::r_s26_common::dress_up;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p`'s 4/4 red Dragon creature token named "Dragon".
fn dragon_token(t: &mut TestGame, p: PlayerId) -> ObjectId {
    token_of(t, p, spec("Dragon", Color::Red, 4))
}

/// `p`'s 5/5 green Dinosaur creature token named "Dinosaur".
fn dinosaur_token(t: &mut TestGame, p: PlayerId) -> ObjectId {
    token_of(t, p, spec("Dinosaur", Color::Green, 5))
}

fn spec(sub: &str, color: Color, n: i32) -> TokenSpec {
    TokenSpec {
        name: sub.into(),
        colors: ColorSet::single(color),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![sub.into()],
        power: Some(n),
        toughness: Some(n),
        abilities: vec![],
        scryfall_name: None,
    }
}

fn is_legendary(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .supertypes
        .contains(Supertype::Legendary)
}

// --- Sarkhan, Soul Aflame -----------------------------------------------------------------

#[test]
fn sarkhan_copying_a_dragon_token() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Sarkhan, Soul Aflame",
        "If the copied Dragon is a token, Sarkhan copies the original characteristics of that token as stated by the effect that created that token, with the noted exceptions."
    );
    supported("Sarkhan, Soul Aflame");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, Soul Aflame");
    t.answer_yes(P0, true);
    // The Dragon token enters; Sarkhan's trigger waits on the stack while the token is
    // tapped, gets counters, +3/+3 and becomes green-blue.
    let dragon = dragon_token(&mut t, P0);
    dress_up(&mut t, dragon);
    t.resolve_all();
    let o = t.obj_now(sarkhan);
    assert_eq!(o.chars.name, "Sarkhan, Soul Aflame");
    assert!(has_subtype(&t, sarkhan, "Dragon"));
    assert!(!has_subtype(&t, sarkhan, "Shaman"));
    assert!(o.chars.colors.contains(Color::Red));
    assert!(!o.chars.colors.contains(Color::Green) && !o.chars.colors.contains(Color::Blue));
    assert!(is_legendary(&t, sarkhan));
    assert!(!o.is_token() && !o.tapped);
    assert!(o.counters.values().all(|n| *n == 0));
    assert_eq!(t.pt(sarkhan), (4, 4));
}

#[test]
fn sarkhan_treats_x_in_the_copied_dragons_cost_as_zero() {
    cr!("107.3m", "202.3e", "707.2");
    ruling!(
        "Sarkhan, Soul Aflame",
        "If the copied Dragon has {X} in its mana cost, X is 0."
    );
    supported("Sarkhan, Soul Aflame");
    supported("Shivan Devastator");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, Soul Aflame");
    // A +1/+1 counter keeps Sarkhan alive as a copy of the 0/0 Devastator.
    t.g
        .add_counters(Entity::Object(sarkhan), counters::PLUS1, 1, None);
    t.lands(P0, "Mountain", 4);
    let dev = t.hand(P0, "Shivan Devastator");
    t.answer_yes(P0, true);
    t.cast(P0, dev).x(3).go();
    t.resolve_all();
    let dev = t.named_on_battlefield("Shivan Devastator")[0];
    assert_eq!(t.counters(dev, counters::PLUS1), 3);
    assert_eq!(mv_now(&mut t, dev), 1);
    let o = t.obj_now(sarkhan);
    assert_eq!(o.chars.name, "Sarkhan, Soul Aflame");
    assert!(has_subtype(&t, sarkhan, "Dragon"));
    assert!(o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Haste));
    assert!(is_legendary(&t, sarkhan));
    // {X}{R} with X = 0.
    assert_eq!(mv_now(&mut t, sarkhan), 1);
    assert_eq!(t.pt(sarkhan), (1, 1));
}

// --- Kimahri, Valiant Guardian ------------------------------------------------------------

/// P0's Kimahri's Ronso Rage resolves at the beginning of combat targeting `what`.
fn kimahri_copies(t: &mut TestGame, kimahri: ObjectId, what: ObjectId) {
    t.answer_targets(P0, &[obj(what)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(what).tapped);
    let o = t.obj_now(kimahri);
    assert_eq!(o.chars.name, "Kimahri, Valiant Guardian");
    assert!(o.has_keyword(KeywordKind::Vigilance));
    assert!(!o.is_token());
    assert_eq!(t.counters(kimahri, counters::PLUS1), 1);
}

#[test]
fn kimahri_copying_a_token() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Kimahri, Valiant Guardian",
        "If the copied creature is a token, Kimahri copies the original characteristics of that token as stated by the effect that created the token, with the listed exceptions."
    );
    supported("Kimahri, Valiant Guardian");
    let mut t = TestGame::new(2);
    let kimahri = t.battlefield(P0, "Kimahri, Valiant Guardian");
    let wolf = dressed_wolf(&mut t, P1);
    kimahri_copies(&mut t, kimahri, wolf);
    let o = t.obj_now(kimahri);
    assert!(has_subtype(&t, kimahri, "Wolf") && !has_subtype(&t, kimahri, "Cat"));
    assert!(o.chars.colors.contains(Color::Green) && !o.chars.colors.contains(Color::Blue));
    assert!(!is_legendary(&t, kimahri));
    // The Wolf's original 2/2, plus Kimahri's own +1/+1 counter.
    assert_eq!(t.pt(kimahri), (3, 3));
}

#[test]
fn kimahri_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Kimahri, Valiant Guardian",
        "If the copied creature is copying something else, then Kimahri becomes a copy of whatever that creature is copying, with the listed exceptions."
    );
    supported("Kimahri, Valiant Guardian");
    let mut t = TestGame::new(2);
    let kimahri = t.battlefield(P0, "Kimahri, Valiant Guardian");
    let clone = cloned_angel(&mut t, P1);
    kimahri_copies(&mut t, kimahri, clone);
    let o = t.obj_now(kimahri);
    assert!(has_subtype(&t, kimahri, "Angel"));
    assert!(o.chars.colors.contains(Color::White));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(kimahri), (5, 5));
}

// --- Sunfrill Imitator --------------------------------------------------------------------

/// `imitator` attacks P1 and becomes a copy of `what`; combat damage is dealt.
fn imitator_attacks(t: &mut TestGame, imitator: ObjectId, what: ObjectId) {
    unsick(t, imitator);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(what)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(imitator, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.obj_now(imitator).chars.name, "Sunfrill Imitator");
}

#[test]
fn sunfrill_imitator_copying_a_token() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Sunfrill Imitator",
        "If the copied creature is a token, Sunfrill Imitator copies the original characteristics of that token as stated by the effect that created the token. Copying a token doesn't make Sunfrill Imitator a token."
    );
    supported("Sunfrill Imitator");
    let mut t = TestGame::new(2);
    let imitator = t.battlefield(P0, "Sunfrill Imitator");
    let dino = dinosaur_token(&mut t, P0);
    dress_up(&mut t, dino);
    imitator_attacks(&mut t, imitator, dino);
    let o = t.obj_now(imitator);
    assert!(!o.is_token());
    assert!(o.chars.colors.contains(Color::Green) && !o.chars.colors.contains(Color::Blue));
    assert!(o.counters.values().all(|n| *n == 0));
    assert_eq!(t.pt(imitator), (5, 5));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn a_sunfrill_imitator_token_copying_a_nontoken_stays_a_token() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Sunfrill Imitator",
        "Similarly, if Sunfrill Imitator itself is a token, copying a nontoken permanent doesn't make it stop being a token."
    );
    supported("Sunfrill Imitator");
    let mut t = TestGame::new(2);
    let original = t.battlefield(P0, "Sunfrill Imitator");
    let dreadmaw = t.battlefield(P0, "Colossal Dreadmaw");
    // A token copy of Sunfrill Imitator.
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(Some(original), P0);
    t.g.exec(
        &Effect::CreateTokenCopy {
            of: Sel::This,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &mut ctx,
    );
    t.g.recompute();
    t.settle();
    let toks = crate::r_s26_common::new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    imitator_attacks(&mut t, toks[0], dreadmaw);
    let o = t.obj_now(toks[0]);
    assert!(o.is_token());
    assert!(has_subtype(&t, toks[0], "Dinosaur"));
    assert!(o.has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(toks[0]), (6, 6));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn sunfrill_imitator_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!(
        "Sunfrill Imitator",
        "If the copied creature is copying something else, then Sunfrill Imitator becomes a copy of whatever that creature is copying."
    );
    supported("Sunfrill Imitator");
    let mut t = TestGame::new(2);
    let imitator = t.battlefield(P0, "Sunfrill Imitator");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    let clone = clone_of(&mut t, P0, dreadmaw);
    imitator_attacks(&mut t, imitator, clone);
    let o = t.obj_now(imitator);
    assert!(!o.is_token());
    assert!(has_subtype(&t, imitator, "Dinosaur"));
    assert!(o.has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(imitator), (6, 6));
    assert_eq!(t.life(P1), 14);
}

// --- Irma, Part-Time Mutant ---------------------------------------------------------------

#[test]
fn irma_copying_a_clone() {
    cr!("707.3", "707.9a", "707.9b");
    ruling!(
        "Irma, Part-Time Mutant",
        "If the copied creature is copying something else, then Irma becomes a copy of whatever that creature copied (with the listed exceptions)."
    );
    supported("Irma, Part-Time Mutant");
    let mut t = TestGame::new(2);
    let irma = t.battlefield(P0, "Irma, Part-Time Mutant");
    let clone = cloned_angel(&mut t, P0);
    t.answer_targets(P0, &[obj(clone)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(irma);
    assert_eq!(o.chars.name, "Irma, Part-Time Mutant");
    assert!(has_subtype(&t, irma, "Angel"));
    assert!(o.chars.colors.contains(Color::White));
    assert!(o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Vigilance));
    assert!(!o.is_token());
    // She keeps "this ability" and gets the +1/+1 counter.
    assert!(o
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("becomes a copy of")));
    assert_eq!(t.counters(irma, counters::PLUS1), 1);
    assert_eq!(t.pt(irma), (5, 5));
}
