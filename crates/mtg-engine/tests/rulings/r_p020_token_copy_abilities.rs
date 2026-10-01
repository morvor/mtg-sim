//! Rulings batch P020 — tokens created as copies of permanents (or cards) by activated and
//! triggered abilities of permanents: the copied object's "enters" triggers trigger and
//! its "enters with" abilities apply as each token enters (CR 707.2, 111.4, 614.1c,
//! 603.6a); a copy checks the copiable values as the token is created (CR 707.2, 707.3).

use crate::r_p020_common::*;
use crate::r_s01_common::{supported, tokens};
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s26_common::{dress_up, fresh};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A game with an Aven Riftwatcher controlled by P0 (and the copying permanent `card`).
fn with_riftwatcher_and(card: &str) -> (TestGame, ObjectId, ObjectId) {
    supported(RIFTWATCHER);
    supported(card);
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    let c = t.battlefield(P0, card);
    (t, rift, c)
}

/// P0 activates the ability of `source` containing `needle` targeting the Riftwatcher;
/// one token copy enters with the copied abilities. Returns the token.
fn activate_on_riftwatcher(
    t: &mut TestGame,
    rift: ObjectId,
    source: ObjectId,
    needle: &str,
) -> ObjectId {
    let before = tokens(t, P0);
    let life = t.life(P0);
    t.answer_targets(P0, &[Entity::Object(rift)]);
    activate_containing(t, P0, source, needle).expect("activation failed");
    t.resolve_all();
    riftwatcher_tokens_entered(t, P0, &before, life, 1)[0]
}

#[test]
fn kiki_jiki_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "614.1c", "603.6a");
    ruling!(
        "Kiki-Jiki, Mirror Breaker",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the copied creature card will also work."
    );
    let (mut t, rift, kiki) = with_riftwatcher_and("Kiki-Jiki, Mirror Breaker");
    activate_on_riftwatcher(&mut t, rift, kiki, "Create a token");
}

#[test]
fn saheeli_the_suns_brilliance_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Saheeli, the Sun's Brilliance",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the target permanent will also work."
    );
    let (mut t, rift, saheeli) = with_riftwatcher_and("Saheeli, the Sun's Brilliance");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let tok = activate_on_riftwatcher(&mut t, rift, saheeli, "Create a token");
    assert!(t.obj_now(tok).is(CardType::Artifact));
}

#[test]
fn orthion_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Orthion, Hero of Lavabrink",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the copied creature will also work."
    );
    let (mut t, rift, orthion) = with_riftwatcher_and("Orthion, Hero of Lavabrink");
    t.lands(P0, "Mountain", 2);
    activate_on_riftwatcher(&mut t, rift, orthion, "Create a token");
}

#[test]
fn jolly_balloon_man_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "The Jolly Balloon Man",
        "Any enters abilities of the copied creature will trigger when the token enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the copied creature will also work."
    );
    let (mut t, rift, man) = with_riftwatcher_and("The Jolly Balloon Man");
    t.lands(P0, "Wastes", 1);
    let tok = activate_on_riftwatcher(&mut t, rift, man, "Create a token");
    assert_eq!(t.pt(tok), (1, 1));
    assert!(t.obj_now(tok).chars.has_subtype("Balloon"));
}

#[test]
fn feldon_token_has_the_copied_cards_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Feldon of the Third Path",
        "Any enters-the-battlefield abilities of the copied creature card will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the copied creature card will also work."
    );
    // "{2}{R}, {T}: Create a token that's a copy of target creature card in your graveyard,
    // except it's an artifact in addition to its other types."
    supported("Feldon of the Third Path");
    let mut t = TestGame::new(2);
    let feldon = t.battlefield(P0, "Feldon of the Third Path");
    let card = t.graveyard(P0, RIFTWATCHER);
    t.lands(P0, "Mountain", 3);
    let tok = activate_on_riftwatcher(&mut t, card, feldon, "Create a token");
    assert!(t.obj_now(tok).is(CardType::Artifact));
    assert!(t.in_graveyard(P0, RIFTWATCHER));
}

#[test]
fn helm_of_the_host_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Helm of the Host",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    // "At the beginning of combat on your turn, create a token that's a copy of equipped
    // creature, except the token isn't legendary."
    supported("Helm of the Host");
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    attach_new(&mut t, P0, "Helm of the Host", rift);
    let life = t.life(P0);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life, 1);
}

#[test]
fn followed_footsteps_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Followed Footsteps",
        "Any \"enters\" abilities of the copied creature trigger when the creature tokens enter the battlefield. The creature tokens also have any \"this enters with\" or \"as this enters\" abilities that the copied creature has."
    );
    // "At the beginning of your upkeep, create a token that's a copy of enchanted
    // creature." It enchants an opponent's Riftwatcher.
    supported("Followed Footsteps");
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P1, RIFTWATCHER);
    attach_new(&mut t, P0, "Followed Footsteps", rift);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    let life = t.life(P0);
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life, 1);
}

/// The Riftwatcher P0 controls is destroyed with Murder while `card` is on the battlefield
/// (its "you may exile it" answered yes). Its leaves trigger and the token's enters
/// trigger each gain P0 2 life. Returns the token.
fn riftwatcher_dies_with(card: &str) -> (TestGame, ObjectId) {
    let (mut t, rift, _) = with_riftwatcher_and(card);
    supported("Murder");
    let life = t.life(P0);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Murder", &[Entity::Object(rift)]);
    t.resolve_all();
    assert!(t.in_exile(RIFTWATCHER));
    let new = riftwatcher_tokens_entered(&t, P0, &[], life + 2, 1);
    (t, new[0])
}

#[test]
fn nightmare_shepherd_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a", "603.10a");
    ruling!(
        "Nightmare Shepherd",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the creature will also work."
    );
    let (t, tok) = riftwatcher_dies_with("Nightmare Shepherd");
    assert_eq!(t.pt(tok), (1, 1));
    assert!(t.obj_now(tok).chars.has_subtype("Nightmare"));
}

#[test]
fn brenard_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a", "603.10a");
    ruling!(
        "Brenard, Ginger Sculptor",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the creature will also work."
    );
    // The token is a 1/1 Food Golem; Brenard gives Food and Golems +2/+2.
    let (t, tok) = riftwatcher_dies_with("Brenard, Ginger Sculptor");
    assert!(t.obj_now(tok).chars.has_subtype("Golem"));
    assert_eq!(t.pt(tok), (3, 3));
}

/// The new tokens P0 controls named `name`.
fn tokens_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    tokens(t, P0)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.name == name)
        .collect()
}

/// The Treasure tokens P0 controls.
fn treasures(t: &TestGame) -> usize {
    tokens(t, P0)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.has_subtype("Treasure"))
        .count()
}

#[test]
fn ratadrabik_tokens_have_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a", "603.10a");
    ruling!(
        "Ratadrabik of Urborg",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the chosen creature will also work."
    );
    // "Whenever another legendary creature you control dies, create a token that's a copy
    // of that creature, except it's not legendary and it's a 2/2 black Zombie..."
    // Daghatar the Adamant "enters with four +1/+1 counters"; Dori, Bearer of Friends:
    // "When Dori enters, create a Treasure token."
    supported("Ratadrabik of Urborg");
    supported("Daghatar the Adamant");
    supported("Dori, Bearer of Friends");
    supported("Murder");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ratadrabik of Urborg");
    let daghatar = t.enter(P0, "Daghatar the Adamant");
    let dori = t.battlefield(P0, "Dori, Bearer of Friends");
    assert_eq!(t.counters(daghatar, counters::PLUS1), 4);
    cast_new(&mut t, P0, "Murder", &[Entity::Object(daghatar)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Murder", &[Entity::Object(dori)]);
    t.resolve_all();
    let d = tokens_named(&t, "Daghatar the Adamant");
    assert_eq!(d.len(), 1);
    assert_eq!(t.counters(d[0], counters::PLUS1), 4);
    assert_eq!(t.pt(d[0]), (6, 6));
    assert_eq!(tokens_named(&t, "Dori, Bearer of Friends").len(), 1);
    assert_eq!(treasures(&t), 1);
}

#[test]
fn cadric_tokens_have_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Cadric, Soul Kindler",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any “As [this permanent] enters the battlefield” or “[This permanent] enters the battlefield with” abilities of the copied permanent will also work."
    );
    // "Whenever another nontoken legendary permanent you control enters, you may pay {1}.
    // If you do, create a token that's a copy of it."
    supported("Cadric, Soul Kindler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cadric, Soul Kindler");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.enter(P0, "Daghatar the Adamant");
    t.resolve_all();
    t.answer_yes(P0, true);
    t.enter(P0, "Dori, Bearer of Friends");
    t.resolve_all();
    let d = tokens_named(&t, "Daghatar the Adamant");
    assert_eq!(d.len(), 1);
    assert_eq!(t.counters(d[0], counters::PLUS1), 4);
    assert_eq!(tokens_named(&t, "Dori, Bearer of Friends").len(), 1);
    // Dori and its copy each created a Treasure.
    assert_eq!(treasures(&t), 2);
}

#[test]
fn necroduality_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Necroduality",
        "Any “enters” abilities of the copied permanent will trigger when the token enters. Any “as [this permanent] enters” or “[this permanent] enters with” abilities of the copied permanent will also work."
    );
    // "Whenever a nontoken Zombie you control enters, create a token that's a copy of that
    // creature." Xenograft makes P0's creatures Zombies.
    supported("Necroduality");
    supported("Xenograft");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necroduality");
    let zombie = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Zombie")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(zombie));
    t.enter(P0, "Xenograft");
    let life = t.life(P0);
    let rift = t.enter(P0, RIFTWATCHER);
    t.resolve_all();
    entered_as_riftwatcher(&t, rift);
    riftwatcher_tokens_entered(&t, P0, &[], life + 2, 1);
}

#[test]
fn molten_echoes_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Molten Echoes",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “As [this creature] enters the battlefield” or “[This creature] enters the battlefield with” abilities of the copied creature will also work."
    );
    // "As this enchantment enters, choose a creature type. Whenever a nontoken creature you
    // control of the chosen type enters, create a token that's a copy of that creature."
    supported("Molten Echoes");
    let mut t = TestGame::new(2);
    let bird = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Bird")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(bird));
    t.enter(P0, "Molten Echoes");
    let life = t.life(P0);
    t.enter(P0, RIFTWATCHER);
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life + 2, 1);
}

#[test]
fn vesuvan_duplimancy_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Vesuvan Duplimancy",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied permanent will also work."
    );
    // "Whenever you cast a spell that targets only a single artifact or creature you
    // control, create a token that's a copy of that artifact or creature..."
    let (mut t, rift, _) = with_riftwatcher_and("Vesuvan Duplimancy");
    let life = t.life(P0);
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(rift)]);
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life, 1);
}

#[test]
fn impostor_syndrome_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Impostor Syndrome",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the creature will also work."
    );
    // "Whenever a nontoken creature you control deals combat damage to a player, create a
    // token that's a copy of it, except it isn't legendary."
    let (mut t, rift, _) = with_riftwatcher_and("Impostor Syndrome");
    let life = t.life(P0);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(rift, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    riftwatcher_tokens_entered(&t, P0, &[], life, 1);
}

#[test]
fn nexus_of_becoming_token_has_the_copied_cards_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Nexus of Becoming",
        "Any enters-the-battlefield abilities of the copied card will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied card will also work."
    );
    // "At the beginning of combat on your turn, draw a card. Then you may exile an artifact
    // or creature card from your hand. If you do, create a token that's a copy of the
    // exiled card, except it's a 3/3 Golem artifact creature..."
    supported("Nexus of Becoming");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nexus of Becoming");
    t.library_top(P0, "Island");
    let card = t.hand(P0, RIFTWATCHER);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    let life = t.life(P0);
    t.resolve_all();
    assert!(t.in_exile(RIFTWATCHER));
    let new = riftwatcher_tokens_entered(&t, P0, &[], life, 1);
    assert_eq!(t.pt(new[0]), (3, 3));
    assert!(t.obj_now(new[0]).chars.has_subtype("Golem"));
}

#[test]
fn coiling_rebirth_triggers_wait_and_can_target_the_token_copy() {
    cr!("603.3", "608.2", "707.2", "702.174a");
    ruling!(
        "Coiling Rebirth",
        "Any abilities that trigger during the resolution of Coiling Rebirth will wait to be put onto the stack until Coiling Rebirth finishes resolving. An ability that triggers when the creature is returned to the battlefield from the graveyard may target the token copy, and vice versa."
    );
    // "Gift a card. Return target creature card from your graveyard to the battlefield.
    // Then if the gift was promised and that creature isn't legendary, create a token
    // that's a copy of that creature, except it's 1/1." Jeong Jeong's Deserters: "When this
    // creature enters, put a +1/+1 counter on target creature."
    supported("Coiling Rebirth");
    supported("Jeong Jeong's Deserters");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Jeong Jeong's Deserters");
    lands_for_cost(&mut t, P0, "Coiling Rebirth");
    let spell = t.hand(P0, "Coiling Rebirth");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, spell).target(Entity::Object(card)).go();
    t.settle();
    t.g.resolve_top();
    // Both are on the battlefield and neither trigger is on the stack yet.
    assert!(t.g.stack.is_empty());
    let new = tokens(&t, P0);
    assert_eq!(new.len(), 1);
    let token = new[0];
    let returned = t.g.current(card);
    assert!(t.on_battlefield(returned));
    // Both triggers target the token — so the returned creature's trigger does.
    t.answer_targets(P0, &[Entity::Object(token)]);
    t.answer_targets(P0, &[Entity::Object(token)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(token, counters::PLUS1), 2);
    assert_eq!(t.counters(returned, counters::PLUS1), 0);
}

/// P0 has a Clone that entered as a copy of Hill Giant (a copy effect), and an ability
/// that copies it has triggered; before it resolves the Clone is tapped, gets two +1/+1
/// counters and +3/+3 and new colors. Returns the resolved token copy, which copies only
/// Hill Giant's printed values (through the Clone's copy effect).
fn copy_of_a_dressed_up_clone(t: &mut TestGame) -> ObjectId {
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert!(t.stack_len() >= 1, "nothing triggered on the Clone entering");
    dress_up(t, clone);
    t.resolve_all();
    let new = tokens(t, P0);
    assert_eq!(new.len(), 1);
    let tok = new[0];
    assert_eq!(t.obj_now(tok).chars.name, "Hill Giant");
    assert_eq!(t.pt(tok), (3, 3));
    assert!(fresh(t, tok));
    assert!(t.obj_now(tok).chars.colors.contains(Color::Red));
    assert!(!t.obj_now(tok).chars.colors.contains(Color::Green));
    tok
}

#[test]
fn riku_token_copies_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3");
    ruling!(
        "Riku of Two Reflections",
        "As the token is created, it checks the printed values of the creature it's copying, as well as any copy effects that have been applied to it."
    );
    // "Whenever another nontoken creature you control enters, you may pay {G}{U}. If you
    // do, create a token that's a copy of that creature."
    supported("Riku of Two Reflections");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Two Reflections");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.answer_yes(P0, true);
    copy_of_a_dressed_up_clone(&mut t);
}

#[test]
fn minion_reflector_token_copies_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3");
    ruling!(
        "Minion Reflector",
        "As the token is created, it checks the printed values of the creature it’s copying, as well as any copy effects that have been applied to it. It won’t copy counters on the creature, nor will it copy other effects that have changed the creature’s power, toughness, types, color, and so on."
    );
    // "Whenever a nontoken creature you control enters, you may pay {2}. If you do, create
    // a token that's a copy of that creature, except it has haste and..."
    supported("Minion Reflector");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Minion Reflector");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    copy_of_a_dressed_up_clone(&mut t);
}
