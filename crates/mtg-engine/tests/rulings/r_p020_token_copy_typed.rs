//! Rulings batch P020 — tokens that are copies of permanents created by modal, kindred,
//! X and noncreature-copying spells: the copied permanent's "enters" triggers trigger and
//! its "enters with" abilities apply as each token enters (CR 707.2, 614.1c, 603.6a), and
//! abilities that trigger while such a spell resolves wait until it has finished (CR
//! 603.3, 608.2), so the trigger of one new token can target another.

use crate::r_p020_common::*;
use crate::r_s01_common::{supported, tokens, triggers_on_stack};
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts Xenograft onto the battlefield for P0 naming `subtype`: each creature P0 controls
/// is that creature type.
fn xenograft(t: &mut TestGame, subtype: &str) {
    supported("Xenograft");
    let i = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == subtype)
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.enter(P0, "Xenograft");
    t.g.recompute();
}

/// P0 controls an Aven Riftwatcher that is also a `subtype` (via Xenograft) and casts
/// `spell` with `modes`; `targets` are answered in order. One token copy enters with its
/// time counters and its "enters" trigger.
fn kindred_copy(spell: &str, subtype: &str, modes: &[usize], extra_targets: &[Entity]) {
    supported(spell);
    supported(RIFTWATCHER);
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    xenograft(&mut t, subtype);
    assert!(t.obj_now(rift).chars.has_subtype(subtype));
    let life = t.life(P0);
    lands_for_cost(&mut t, P0, spell);
    let card = t.hand(P0, spell);
    t.answer_targets(P0, &[Entity::Object(rift)]);
    for e in extra_targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, card).modes(modes).go();
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life, 1);
}

#[test]
fn ashlings_command_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "700.2");
    ruling!(
        "Ashling's Command",
        "Any enters abilities of the copied permanent will trigger when the token enters. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied permanent will also work."
    );
    // "• Create a token that's a copy of target Elemental you control. • Target player
    // draws two cards."
    kindred_copy("Ashling's Command", "Elemental", &[0, 1], &[Entity::Player(P0)]);
}

#[test]
fn brigids_command_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "700.2");
    ruling!(
        "Brigid's Command",
        "Any enters abilities of the copied permanent will trigger when the token enters. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied permanent will also work."
    );
    // "• Create a token that's a copy of target Kithkin you control. • Target player
    // creates a 1/1 green and white Kithkin creature token." — the opponent does.
    kindred_copy("Brigid's Command", "Kithkin", &[0, 1], &[Entity::Player(P1)]);
}

#[test]
fn syggs_command_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "700.2");
    ruling!(
        "Sygg's Command",
        "Any enters abilities of the copied permanent will trigger when the token enters. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied permanent will also work."
    );
    // "• Create a token that's a copy of target Merfolk you control. • Target player draws
    // a card."
    kindred_copy("Sygg's Command", "Merfolk", &[0, 2], &[Entity::Player(P0)]);
}

#[test]
fn trystans_command_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "700.2");
    ruling!(
        "Trystan's Command",
        "Any enters abilities of the copied permanent will trigger when the token enters. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied permanent will also work."
    );
    // "• Create a token that's a copy of target Elf you control. • Creatures target player
    // controls get +3/+3 until end of turn. Untap them."
    kindred_copy("Trystan's Command", "Elf", &[0, 3], &[Entity::Player(P1)]);
}

/// P0 casts `spell` targeting P0's Aven Riftwatcher; `extra_lands` Wastes pay for X or
/// additional costs. Asserts `n` token copies entered with the copied abilities and
/// returns them.
fn copy_mine(
    spell: &str,
    modes: &[usize],
    x: Option<i64>,
    extra_lands: usize,
    n: usize,
) -> (TestGame, Vec<ObjectId>) {
    supported(spell);
    supported(RIFTWATCHER);
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    let life = t.life(P0);
    lands_for_cost(&mut t, P0, spell);
    t.lands(P0, "Wastes", extra_lands);
    let card = t.hand(P0, spell);
    let mut b = t.cast(P0, card).target(Entity::Object(rift));
    if !modes.is_empty() {
        b = b.modes(modes);
    }
    if let Some(x) = x {
        b = b.x(x);
    }
    b.go();
    t.resolve_all();
    let new = riftwatcher_tokens_entered(&t, P0, &[], life, n);
    (t, new)
}

#[test]
fn devastating_onslaught_tokens_have_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "107.3a");
    ruling!(
        "Devastating Onslaught",
        "Any enters abilities of the copied permanent will trigger when the tokens enter. Any “as [this permanent] enters” or “[this permanent] enters with” abilities of the copied permanent will also work."
    );
    // {X}{X}{R} with X = 2.
    copy_mine("Devastating Onslaught", &[], Some(2), 4, 2);
}

#[test]
fn molten_duplication_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Molten Duplication",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied permanent will also work."
    );
    let (t, new) = copy_mine("Molten Duplication", &[], None, 0, 1);
    assert!(t.obj_now(new[0]).is(CardType::Artifact));
}

#[test]
fn three_steps_ahead_token_has_the_copied_enters_abilities() {
    cr!("707.2", "702.172a", "614.1c", "603.6a");
    ruling!(
        "Three Steps Ahead",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied permanent will also work."
    );
    // "+ {3} — Create a token that's a copy of target artifact or creature you control."
    copy_mine("Three Steps Ahead", &[1], None, 3, 1);
}

#[test]
fn stolen_identity_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Stolen Identity",
        "Any \"enters\" abilities of the copied permanent will trigger when the token enters the battlefield. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the chosen permanent will also work."
    );
    copy_mine("Stolen Identity", &[], None, 0, 1);
}

#[test]
fn applied_geometry_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Applied Geometry",
        "Any enters abilities of the copied permanent will trigger when the token enters. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied permanent will also work."
    );
    // "...except it's a 0/0 Fractal creature in addition to its other types. Put six
    // +1/+1 counters on it."
    let (t, new) = copy_mine("Applied Geometry", &[], None, 0, 1);
    assert_eq!(t.counters(new[0], counters::PLUS1), 6);
    assert_eq!(t.pt(new[0]), (6, 6));
}

#[test]
fn season_of_weaving_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "700.2i");
    ruling!(
        "Season of Weaving",
        "Any \"enters\" abilities of the copied permanent will trigger when the token enters. Any \"as [this permanent enters\" or \"[this permanent] enters with\" abilities of the chosen permanent will also work."
    );
    // "{P}{P} — Choose an artifact or creature you control. Create a token that's a copy
    // of it." Chosen twice: two tokens.
    supported("Season of Weaving");
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    let life = t.life(P0);
    lands_for_cost(&mut t, P0, "Season of Weaving");
    let card = t.hand(P0, "Season of Weaving");
    t.answer_choose(P0, &[Entity::Object(rift)]);
    t.answer_choose(P0, &[Entity::Object(rift)]);
    t.cast(P0, card).modes(&[1, 1]).go();
    t.resolve_all();
    riftwatcher_tokens_entered(&t, P0, &[], life, 2);
}

#[test]
fn astral_dragon_tokens_have_the_copied_noncreature_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Astral Dragon",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the chosen permanent will also work."
    );
    // "When this creature enters, create two tokens that are copies of target noncreature
    // permanent, except they're 3/3 Dragon creatures..." Crack in Time has vanishing 3 and
    // "When this enchantment enters ..., exile target creature an opponent controls until
    // this enchantment leaves the battlefield."
    supported("Astral Dragon");
    supported("Crack in Time");
    let mut t = TestGame::new(2);
    let crack = t.battlefield(P0, "Crack in Time");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(crack)]);
    t.enter(P0, "Astral Dragon");
    t.settle();
    // The tokens' "enters" triggers target the opponent's creatures.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve();
    let new = new_tokens_of(&t, P0, &[]);
    assert_eq!(new.len(), 2);
    for id in &new {
        assert_eq!(t.obj_now(*id).chars.name, "Crack in Time");
        assert_eq!(t.pt(*id), (3, 3));
        assert_eq!(t.counters(*id, counters::TIME), 3);
    }
    assert_eq!(triggers_on_stack(&t, "exile target creature"), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
}

#[test]
fn saheelis_artistry_tokens_have_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "700.2");
    ruling!(
        "Saheeli's Artistry",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the chosen permanent will also work."
    );
    // "• Create a token that's a copy of target creature, except it's an artifact in
    // addition to its other types."
    let (t, new) = copy_mine("Saheeli's Artistry", &[1], None, 0, 1);
    assert!(t.obj_now(new[0]).is(CardType::Artifact));
}

#[test]
fn saheelis_artistry_token_triggers_wait_and_can_target_the_other_token() {
    cr!("603.3", "608.2", "707.2", "603.6a");
    ruling!(
        "Saheeli's Artistry",
        "Any abilities that trigger during the resolution of Saheeli's Artistry will wait to be put onto the stack until Saheeli's Artistry finishes resolving. An ability that triggers on the first token entering the battlefield may target the second token and vice versa."
    );
    // Mode 1 copies Ornithopter (an artifact); mode 2 copies Jeong Jeong's Deserters ("When
    // this creature enters, put a +1/+1 counter on target creature."). The Deserters
    // token's trigger targets the Ornithopter token, created by the same spell.
    supported("Saheeli's Artistry");
    supported("Jeong Jeong's Deserters");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let deserters = t.battlefield(P0, "Jeong Jeong's Deserters");
    lands_for_cost(&mut t, P0, "Saheeli's Artistry");
    let card = t.hand(P0, "Saheeli's Artistry");
    t.cast(P0, card)
        .modes(&[0, 1])
        .targets(&[Entity::Object(thopter)])
        .targets(&[Entity::Object(deserters)])
        .go();
    t.settle();
    t.g.resolve_top();
    // The spell has finished resolving: both tokens are on the battlefield, and the
    // Deserters token's trigger hasn't been put on the stack yet.
    assert!(t.g.stack.is_empty());
    let new = new_tokens_of(&t, P0, &[]);
    assert_eq!(new.len(), 2);
    let thopter_token = *new
        .iter()
        .find(|x| t.obj_now(**x).chars.name == "Ornithopter")
        .unwrap();
    // Its trigger targets the Ornithopter token, created by the same spell.
    t.answer_targets(P0, &[Entity::Object(thopter_token)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter on target creature"), 1);
    t.resolve_all();
    assert_eq!(t.counters(thopter_token, counters::PLUS1), 1);
    assert_eq!(t.counters(thopter, counters::PLUS1), 0);
}
