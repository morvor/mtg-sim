//! Rulings batch P019 — "reveal cards from the top of [a] library until [a card of some
//! kind] is revealed" (CR 701.20) followed by putting it onto the battlefield or casting
//! it: Polymorph, Transmogrify, Blessed Reincarnation, Spellshift, Clifftop Lookout,
//! Thicket Elemental and Gamekeeper. With no such card the whole library is revealed and
//! then shuffled or randomized; with an illegal target nothing is revealed (CR 608.2b).

use crate::r_p019_common::*;
use crate::r_p125_common::shuffled;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use crate::r_s02_common::destroy;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The name of the top card of `p`'s library.
fn top(t: &TestGame, p: PlayerId) -> String {
    let id = *t.g.players[p.idx()].library.last().unwrap();
    t.g.obj(id).chars.name.to_string()
}

#[test]
fn a_creature_card_is_any_card_with_the_type_creature() {
    cr!("205.2a", "109.2");
    ruling!(
        "Polymorph",
        "A “creature card” is any card with the type creature, even if it has other types such as artifact, enchantment, or land. Older cards of type summon are also creature cards."
    );
    supported("Polymorph");
    supported("Disentomb");
    supported("Liliana Vess");
    supported("Sphinx Ambassador");
    // Polymorph: "Destroy target creature. It can't be regenerated. Its controller reveals
    // cards from the top of their library until they reveal a creature card. The player
    // puts that card onto the battlefield, then shuffles all other cards revealed this way
    // into their library." Sol Ring is passed over; Dryad Arbor (a land creature) is put
    // onto the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P1, &["Sol Ring", "Dryad Arbor"]);
    cast_new(&mut t, P0, "Polymorph", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.named_on_battlefield("Dryad Arbor").len(), 1);
    assert!(t.named_on_battlefield("Sol Ring").is_empty());
    assert_eq!(in_library(&t, P1, "Sol Ring"), 1);

    // Disentomb: "Return target creature card from your graveyard to your hand." An
    // artifact creature card is one.
    let mut t = TestGame::new(2);
    let thopter = t.graveyard(P0, "Ornithopter");
    cast_new(&mut t, P0, "Disentomb", &[Entity::Object(thopter)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ornithopter"));

    // Liliana Vess: "−8: Put all creature cards from all graveyards onto the battlefield
    // under your control." Ornithopter and Dryad Arbor, not Sol Ring.
    let mut t = TestGame::new(2);
    let liliana = t.battlefield(P0, "Liliana Vess");
    t.g.add_counters(Entity::Object(liliana), counters::LOYALTY, 4, None);
    t.graveyard(P1, "Ornithopter");
    t.graveyard(P1, "Dryad Arbor");
    t.graveyard(P1, "Sol Ring");
    crate::r_s06_common::activate_containing(&mut t, P0, liliana, "all creature cards")
        .unwrap()
        .unwrap();
    t.resolve_all();
    let mine: Vec<String> =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.owner == P1)
            .map(|o| o.chars.name.to_string())
            .collect();
    assert_eq!(mine.len(), 2, "{mine:?}");
    assert!(t.in_graveyard(P1, "Sol Ring"));

    // Sphinx Ambassador: "Whenever this creature deals combat damage to a player, search
    // that player's library for a card, then that player chooses a card name. If you
    // searched for a creature card that doesn't have that name, you may put it onto the
    // battlefield under your control." An artifact creature card found.
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Sphinx Ambassador");
    let thopter = t.library_top(P1, "Ornithopter");
    attack_with(&mut t, &[(sphinx, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.answer(P1, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer_yes(P0, true);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    let thopters = t.named_on_battlefield("Ornithopter");
    assert_eq!(thopters.len(), 1);
    assert_eq!(t.obj_now(thopters[0]).controller, P0);
}

#[test]
fn polymorph_on_an_indestructible_creature_still_reveals() {
    cr!("702.12b", "608.2b");
    ruling!(
        "Polymorph",
        "If the targeted creature has indestructible, it’s still a legal target — it just isn’t destroyed. The rest of Polymorph’s effect happens as normal."
    );
    supported("Darksteel Myr");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    stack_library(&mut t, P1, &["Grizzly Bears"]);
    cast_new(&mut t, P0, "Polymorph", &[Entity::Object(myr)]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn with_an_illegal_target_nothing_is_revealed_or_shuffled() {
    cr!("608.2b");
    ruling!(
        "Polymorph",
        "If the targeted creature is an illegal target by the time Polymorph would resolve, the entire spell doesn’t resolve. Nothing else happens."
    );
    ruling!(
        "Transmogrify",
        "If the target creature is an illegal target by the time Transmogrify tries to resolve, the spell won't resolve. No player reveals cards or shuffles their library."
    );
    supported("Transmogrify");
    // Transmogrify: "Exile target creature. That creature's controller reveals cards from
    // the top of their library until they reveal a creature card. That player puts that
    // card onto the battlefield, then shuffles the rest into their library."
    for spell in ["Polymorph", "Transmogrify"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        stack_library(&mut t, P1, &["Sol Ring", "Hill Giant"]);
        cast_new(&mut t, P0, spell, &[Entity::Object(bears)]);
        destroy(&mut t, bears);
        t.resolve_all();
        assert!(t.named_on_battlefield("Hill Giant").is_empty(), "{spell}");
        assert_eq!(top(&t, P1), "Sol Ring", "{spell}");
        assert!(!shuffled(&t, P1), "{spell}");
    }
}

#[test]
fn with_no_creature_card_the_library_is_revealed_and_shuffled() {
    cr!("701.20a", "701.24a");
    ruling!(
        "Polymorph",
        "If there are no creature cards in the player’s library, all the cards in that library are revealed, then the library is shuffled. (The targeted creature remains destroyed.)"
    );
    ruling!(
        "Polymorph",
        "If there are no creatures in the player’s library, then the target creature is still destroyed, you see all the cards in that player’s library, and then they shuffle and continue play."
    );
    ruling!(
        "Transmogrify",
        "If the player reveals their entire library without revealing a creature card, that player just shuffles their library."
    );
    ruling!(
        "Blessed Reincarnation",
        "If there are no creature cards in the library, it will be revealed and shuffled. The target creature will remain exiled."
    );
    supported("Blessed Reincarnation");
    // The libraries hold no creature cards (a Sol Ring and fillers).
    for (spell, zone) in [
        ("Polymorph", Zone::Graveyard(P1)),
        ("Transmogrify", Zone::Exile),
        ("Blessed Reincarnation", Zone::Exile),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.library_top(P1, "Sol Ring");
        let library = t.library_size(P1);
        cast_new(&mut t, P0, spell, &[Entity::Object(bears)]);
        t.resolve();
        assert_eq!(t.zone(bears), zone, "{spell}");
        assert_eq!(t.library_size(P1), library, "{spell}");
        assert_eq!(in_library(&t, P1, "Sol Ring"), 1, "{spell}");
        assert!(shuffled(&t, P1), "{spell}");
        assert!(crate::r_s01_common::creatures(&t, P1).is_empty(), "{spell}");
    }
}

/// P1 casts Shock at P0; P0 casts Spellshift ("Counter target instant or sorcery spell.
/// Its controller reveals cards from the top of their library until they reveal an
/// instant or sorcery card. That player may cast that card without paying its mana cost.
/// Then the player shuffles.") targeting `target` (the Shock if `None`).
fn spellshift(t: &mut TestGame, spell: &str) -> ObjectId {
    t.lands(P1, "Mountain", 1);
    let card = t.hand(P1, spell);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.g.turn.priority = Some(P1);
    let s = t.g.cast_spell(P1, card, CastMethod::Normal).unwrap();
    t.g.flush_events();
    cast_new(t, P0, "Spellshift", &[Entity::Object(s)]);
    s
}

#[test]
fn spellshift_reveals_and_the_card_may_be_cast_or_is_shuffled_back() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Spellshift",
        "If the revealed instant or sorcery card isn’t cast, it gets shuffled back into the library."
    );
    supported("Spellshift");
    // P1 reveals Grizzly Bears, then Opt, and doesn't cast it.
    let mut t = TestGame::new(2);
    let library = t.library_size(P1) + 2;
    stack_library(&mut t, P1, &["Grizzly Bears", "Opt"]);
    spellshift(&mut t, "Shock");
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(in_library(&t, P1, "Opt"), 1);
    assert_eq!(in_library(&t, P1, "Grizzly Bears"), 1);
    assert_eq!(t.library_size(P1), library);
    assert!(shuffled(&t, P1));
    // ... or casts it (for free): Opt resolves and P1 draws.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P1, &["Grizzly Bears", "Opt"]);
    spellshift(&mut t, "Shock");
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Opt"));
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn spellshift_on_a_spell_that_cant_be_countered_still_reveals() {
    cr!("701.6a", "608.2c");
    ruling!(
        "Spellshift",
        "If the targeted spell can’t be countered, it remains on the stack and the rest of Spellshift’s effect continues to happen."
    );
    supported("Abrupt Decay");
    // Abrupt Decay: "This spell can't be countered. Destroy target nonland permanent with
    // mana value 3 or less." P1 still reveals (and casts) Opt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Forest", 1);
    let decay = t.hand(P1, "Abrupt Decay");
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.g.turn.priority = Some(P1);
    let decay = t.g.cast_spell(P1, decay, CastMethod::Normal).unwrap();
    t.g.flush_events();
    stack_library(&mut t, P1, &["Opt"]);
    cast_new(&mut t, P0, "Spellshift", &[Entity::Object(decay)]);
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    t.resolve();
    assert!(t.g.stack.contains(&decay));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Opt"));
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn spellshift_with_no_instant_or_sorcery_reveals_the_library_and_shuffles() {
    cr!("701.20a", "701.24a");
    ruling!(
        "Spellshift",
        "If there are no instant or sorcery cards in the player’s library, the entire library is revealed and then shuffled."
    );
    let mut t = TestGame::new(2);
    t.library_top(P1, "Grizzly Bears");
    let library = t.library_size(P1);
    spellshift(&mut t, "Shock");
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(yes_nos(&t, from), 0);
    assert_eq!(t.library_size(P1), library);
    assert!(shuffled(&t, P1));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn clifftop_lookout_with_no_land_randomizes_the_whole_library() {
    cr!("701.20a");
    ruling!(
        "Clifftop Lookout",
        "If you get through your whole library without revealing a land card, you’ll just end up revealing and randomizing your library."
    );
    supported("Clifftop Lookout");
    // "When this creature enters, reveal cards from the top of your library until you
    // reveal a land card. Put that card onto the battlefield tapped and the rest on the
    // bottom of your library in a random order."
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    let library = t.library_size(P0);
    t.enter(P0, "Clifftop Lookout");
    t.resolve_all();
    assert_eq!(t.library_size(P0), library);
    assert_eq!(in_library(&t, P0, "Grizzly Bears"), 1);
    assert_eq!(t.g.permanents().filter(|o| o.is(CardType::Land)).count(), 0);
}

#[test]
fn thicket_elemental_shuffles_if_you_chose_to_reveal() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Thicket Elemental",
        "The \"if you do\" means \"if you choose to reveal cards\". You still shuffle if you have no creature card in your library."
    );
    supported("Thicket Elemental");
    // "Kicker {1}{G}. When this creature enters, if it was kicked, you may reveal cards
    // from the top of your library until you reveal a creature card. If you do, put that
    // card onto the battlefield and shuffle all other cards revealed this way into your
    // library." No creature card: still a shuffle.
    for reveal in [true, false] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Sol Ring");
        let library = t.library_size(P0);
        t.lands(P0, "Forest", 3);
        t.lands(P0, "Wastes", 4);
        let card = t.hand(P0, "Thicket Elemental");
        t.cast(P0, card).kicked(true).go();
        t.resolve();
        t.answer_yes(P0, reveal);
        t.resolve_all();
        assert_eq!(shuffled(&t, P0), reveal);
        assert_eq!(t.library_size(P0), library);
        assert_eq!(in_library(&t, P0, "Sol Ring"), 1);
    }
}

#[test]
fn gamekeeper_gone_from_the_graveyard_cant_be_exiled_and_nothing_is_revealed() {
    cr!("603.10a", "608.2c");
    ruling!(
        "Gamekeeper",
        "If Gamekeeper is no longer in your graveyard as its triggered ability resolves, you can’t choose to exile it. You won’t reveal cards from your library."
    );
    supported("Gamekeeper");
    // "When this creature dies, you may exile it. If you do, reveal cards from the top of
    // your library until you reveal a creature card. Put that card onto the battlefield
    // and put all other cards revealed this way into your graveyard."
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Gamekeeper");
    stack_library(&mut t, P0, &["Sol Ring", "Grizzly Bears"]);
    destroy(&mut t, keeper);
    assert_eq!(t.stack_len(), 1);
    // The card leaves the graveyard (to its owner's hand) before the trigger resolves.
    let card = t.g.current(keeper);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    crate::r_s05_common::move_to(&mut t, card, Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Gamekeeper"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(top(&t, P0), "Sol Ring");
    // With it still there: exiled, and the Bears are put onto the battlefield.
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Gamekeeper");
    stack_library(&mut t, P0, &["Sol Ring", "Grizzly Bears"]);
    destroy(&mut t, keeper);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("Gamekeeper"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P0, "Sol Ring"));
}
