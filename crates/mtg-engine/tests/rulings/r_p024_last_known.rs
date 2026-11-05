//! Rulings batch P024 — a triggered ability that copies an object that has left its zone
//! by the time it resolves uses the object's last known information (CR 608.2h, 707.2,
//! 113.7a), and its "you may pay" cost can still be paid (CR 608.2c); the object a
//! triggered ability refers to is fixed as it triggers (CR 603.2, 603.7c).

use crate::r_p024_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// With P0's `source` on the battlefield (and lands for `cost`), `p`'s Clone enters as a
/// copy of `what` (a card name, put onto the battlefield under P1's control); the
/// ability `source` triggers is answered "yes" ("you may pay"), and the Clone dies before
/// it resolves. Returns the game, the Clone and the tokens `token_p` got.
fn clone_enters_and_dies(
    source: &str,
    cost: &str,
    p: PlayerId,
    what: &str,
    setup: impl FnOnce(&mut TestGame),
) -> (TestGame, Vec<ObjectId>) {
    supported(source);
    let mut t = TestGame::new(2);
    t.battlefield(P0, source);
    setup(&mut t);
    let orig = t.battlefield(P1, what);
    lands_for(&mut t, P0, cost);
    let before = t.g.battlefield.clone();
    t.answer_choose(p, &[obj(orig)]);
    let clone = t.enter(p, "Clone");
    t.settle();
    assert_eq!(t.stack_len(), 1, "{source} didn't trigger: {}", t.dump_log());
    // The Clone leaves before the ability resolves: in the graveyard, it's a Clone.
    kill(&mut t, clone);
    assert!(t.in_graveyard(p, "Clone"));
    t.answer_yes(P0, true);
    t.resolve_all();
    let mut toks = new_tokens(&t, P0, &before);
    toks.extend(new_tokens(&t, P1, &before));
    (t, toks)
}

#[test]
fn inalla_copies_a_wizard_that_left_as_it_last_existed() {
    cr!("608.2h", "707.2", "608.2c");
    ruling!("Inalla, Archmage Ritualist", "If the creature that caused Inalla's ability to trigger has already left the battlefield by the time the ability resolves, you can still pay {1}. If you do, you'll create a token with the copiable values of the characteristics of that creature as it last existed on the battlefield.");
    let (t, toks) =
        clone_enters_and_dies("Inalla, Archmage Ritualist", "{1}", P0, "Prodigal Sorcerer", |_| {});
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Prodigal Sorcerer");
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1, "{{1}} wasn't paid");
}

#[test]
fn minion_reflector_copies_a_creature_that_left_as_it_last_existed() {
    cr!("608.2h", "707.2", "608.2c");
    ruling!("Minion Reflector", "If the creature that caused Minion Reflector’s ability to trigger has already left the battlefield by the time the ability resolves, you can still pay {2}. If you do, you’ll still put a token onto the battlefield. That token has the copiable values of the characteristics of that nontoken creature as it last existed on the battlefield.");
    let (t, toks) = clone_enters_and_dies("Minion Reflector", "{2}", P0, "Hill Giant", |_| {});
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
    assert_eq!(abilities_with(&t, toks[0], "sacrifice this permanent"), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
}

#[test]
fn riku_copies_a_creature_that_left_as_it_last_existed() {
    cr!("608.2h", "707.2", "608.2c");
    ruling!("Riku of Two Reflections", "If the creature that caused Riku's second ability to trigger has already left the battlefield by the time the ability resolves, you can still pay {G}{U}. If you do, you'll still put a token onto the battlefield. That token has the copiable values of the characteristics of that creature as it last existed on the battlefield.");
    let (t, toks) = clone_enters_and_dies("Riku of Two Reflections", "{G}{U}", P0, "Hill Giant", |_| {});
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
}

#[test]
fn bramble_sovereign_copies_a_creature_that_left_as_it_last_existed() {
    cr!("608.2h", "707.2", "608.2c");
    ruling!("Bramble Sovereign", "If the entering creature leaves the battlefield before Bramble Sovereign's ability resolves, use that creature's last known existence on the battlefield to determine the token's characteristics.");
    // P1's Clone: P1 (its controller as it last existed) creates the token.
    let (t, toks) = clone_enters_and_dies("Bramble Sovereign", "{1}{G}", P1, "Hill Giant", |_| {});
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
    assert_eq!(t.obj_now(toks[0]).controller, P1);
}

#[test]
fn mirror_march_copies_a_creature_that_left_as_it_last_existed() {
    cr!("608.2h", "707.2", "705.2");
    ruling!("Mirror March", "If the entering creature leaves the battlefield before Mirror March’s ability resolves, use that creature’s last known existence on the battlefield to determine the token’s characteristics.");
    // Two flips won, then one lost: two tokens.
    let (t, toks) = clone_enters_and_dies("Mirror March", "", P0, "Hill Giant", |t| {
        t.g.dice.loaded_coins.extend([true, true, false]);
    });
    assert_eq!(toks.len(), 2, "{}", t.dump_log());
    for c in toks {
        assert_eq!(name_of(&t, c), "Hill Giant");
        assert!(has_kw(&t, c, KeywordKind::Haste));
    }
}

#[test]
fn molten_echoes_copies_a_creature_that_left_as_it_last_existed() {
    cr!("608.2h", "707.2");
    ruling!("Molten Echoes", "If the nontoken creature leaves the battlefield before Molten Echoes's triggered ability resolves, the token that's created uses the creature's last known information from before it left.");
    supported("Molten Echoes");
    let mut t = TestGame::new(2);
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Giant");
    t.enter(P0, "Molten Echoes");
    t.resolve_all();
    let giant = t.battlefield(P1, "Hill Giant");
    let before = t.g.battlefield.clone();
    t.answer_choose(P0, &[obj(giant)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    kill(&mut t, clone);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn cadric_may_still_copy_a_legendary_permanent_that_changed() {
    cr!("608.2h", "707.2", "608.2c", "603.2");
    ruling!("Cadric, Soul Kindler", "If the legendary permanent that entered the battlefield is no longer on the battlefield, no longer under the same player’s control, or no longer legendary when the triggered ability resolves, the ability’s controller may still pay {1} to copy it. If it’s no longer on the battlefield, use its copiable characteristics as it last existed on the battlefield to determine the characteristics of the token copy.");
    supported("Cadric, Soul Kindler");
    for change in ["dies", "control", "nonlegendary"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Cadric, Soul Kindler");
        t.lands(P0, "Wastes", 1);
        let before = t.g.battlefield.clone();
        let isamaru = t.enter(P0, "Isamaru, Hound of Konda");
        t.settle();
        assert_eq!(t.stack_len(), 1, "{change}");
        match change {
            "dies" => kill(&mut t, isamaru),
            "control" => crate::r_s06_common::give_control(&mut t, isamaru, P1),
            _ => crate::r_s26_common::modify_until_eot(
                &mut t,
                isamaru,
                vec![Modification::RemoveSupertypes(vec![Supertype::Legendary])],
            ),
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), 1, "{change}: {}", t.dump_log());
        let c = toks[0];
        assert_eq!(name_of(&t, c), "Isamaru, Hound of Konda", "{change}");
        // The token copies the copiable values: it's legendary.
        assert!(t.obj_now(c).chars.supertypes.contains(Supertype::Legendary), "{change}");
        assert!(has_kw(&t, c, KeywordKind::Haste));
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1);
    }
}

// --- Vesuvan Duplimancy -----------------------------------------------------------------

/// P0's Vesuvan Duplimancy; P0 casts Giant Growth targeting `what` (P0's), and the
/// trigger goes on the stack above it.
fn duplimancy_trigger(t: &mut TestGame, what: ObjectId) {
    supported("Vesuvan Duplimancy");
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(obj(what)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn vesuvan_duplimancy_copies_a_target_that_left_as_it_last_existed() {
    cr!("608.2h", "707.2", "603.2");
    ruling!("Vesuvan Duplimancy", "If the target of the spell leaves the battlefield before the triggered ability resolves, use its characteristics as it last existed on the battlefield to determine the characteristics of the token created.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vesuvan Duplimancy");
    let giant = t.battlefield(P1, "Hill Giant");
    let clone = clone_of(&mut t, P0, giant);
    let before = t.g.battlefield.clone();
    duplimancy_trigger(&mut t, clone);
    kill(&mut t, clone);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
}

#[test]
fn vesuvan_duplimancy_copies_the_spells_original_target() {
    cr!("603.2", "115.7");
    ruling!("Vesuvan Duplimancy", "If the target of the spell is changed after the triggered ability is put on the stack, the token that is created is a copy of the spell’s original target, not the new target.");
    supported("Redirect");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vesuvan Duplimancy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let before = t.g.battlefield.clone();
    duplimancy_trigger(&mut t, bears);
    let gg = t.g.stack[0];
    // Redirect ("You may choose new targets for target spell.") changes Giant Growth's
    // target to the Giant before the trigger resolves. (Redirect targets a spell, not a
    // creature: it doesn't trigger Duplimancy.)
    t.lands(P0, "Island", 2);
    let redirect = t.hand(P0, "Redirect");
    t.g.turn.priority = Some(P0);
    t.cast(P0, redirect).target(obj(gg)).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve();
    assert_eq!(crate::r_s25_common::targets_of(&t, gg), vec![obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 6));
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Grizzly Bears");
}

// --- Riku's spell copy ----------------------------------------------------------------------

#[test]
fn riku_copies_a_spell_that_left_the_stack_as_it_last_existed() {
    cr!("608.2h", "707.10", "608.2c");
    ruling!("Riku of Two Reflections", "If the spell that caused Riku's first ability to trigger has left the stack by the time the ability resolves, you can still pay {U}{R}. If you do, you'll copy the spell as it last existed on the stack.");
    supported("Riku of Two Reflections");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Two Reflections");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters the Bolt in response.
    lands_for(&mut t, P1, "{1}{U}{U}");
    let cancel = t.hand(P1, "Cancel");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cancel).target(obj(spell)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    lands_for(&mut t, P0, "{U}{R}");
    t.answer_yes(P0, true);
    t.resolve_all();
    // The copy of the Bolt as it last existed: 3 damage to P1.
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}
