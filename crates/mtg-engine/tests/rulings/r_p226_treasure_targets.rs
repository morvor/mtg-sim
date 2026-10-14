//! Rulings batch P226 — spells and abilities that create Treasure tokens (CR 111.10a)
//! alongside a targeted instruction: with no legal target left, nothing happens
//! (CR 608.2b); with a legal target that isn't affected (indestructible, can't be
//! countered), the Treasure is still created (CR 608.2c).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Treasure").len()
}

/// P0 casts the real spell `name` (with lands for it) at `target`; the target is
/// destroyed before the spell resolves, and the spell leaves the stack.
fn fizzles(name: &str, target_name: &str) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, target_name);
    give_mana_for(&mut t, P0, name);
    let card = t.hand(P0, name);
    let spell = t.cast(P0, card).target(target).go();
    destroy(&mut t, target);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    (t, target)
}

/// P0 casts the real spell `name` at the real creature `target_name` P1 controls, and
/// it resolves.
fn resolves_on(name: &str, target_name: &str) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, target_name);
    give_mana_for(&mut t, P0, name);
    let card = t.hand(P0, name);
    t.cast(P0, card).target(target).go();
    t.resolve_all();
    (t, target)
}

#[test]
fn reckless_ransacking_does_nothing_without_its_target() {
    cr!("608.2b", "111.10a");
    ruling!(
        "Reckless Ransacking",
        "If the target creature is an illegal target as Reckless Ransacking tries to resolve, it won't resolve and none of its effects will happen. You won't create a Treasure token."
    );
    supported("Reckless Ransacking");
    let (t, _) = fizzles("Reckless Ransacking", "Grizzly Bears");
    assert_eq!(treasures(&t, P0), 0);
    // "Target creature gets +3/+2 until end of turn. Create a Treasure token."
    let (t, bears) = resolves_on("Reckless Ransacking", "Grizzly Bears");
    assert_eq!(t.pt(bears), (5, 4));
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn ancestors_aid_does_nothing_without_its_target() {
    cr!("608.2b", "111.10a");
    ruling!(
        "Ancestors' Aid",
        "If the target of Ancestors' Aid is illegal as the spell tries to resolve, it won't resolve and none of its effects will happen. You won't create a Treasure token."
    );
    supported("Ancestors' Aid");
    let (t, _) = fizzles("Ancestors' Aid", "Grizzly Bears");
    assert_eq!(treasures(&t, P0), 0);
    let (t, bears) = resolves_on("Ancestors' Aid", "Grizzly Bears");
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn deadly_derision_and_grim_bounty_do_nothing_without_their_targets() {
    cr!("608.2b", "111.10a");
    ruling!(
        "Deadly Derision",
        "If the target of Deadly Derision is illegal as the spell tries to resolve, it won't resolve and none of its effects will happen. You won't create a Treasure token."
    );
    ruling!(
        "Grim Bounty",
        "If the target is not legal as Grim Bounty tries to resolve, perhaps because the creature or planeswalker is no longer on the battlefield, it will do nothing. You will not create a Treasure token."
    );
    for name in ["Deadly Derision", "Grim Bounty"] {
        supported(name);
        let (t, _) = fizzles(name, "Grizzly Bears");
        assert_eq!(treasures(&t, P0), 0, "{name}");
        // "Destroy target creature or planeswalker. Create a Treasure token."
        let (t, bears) = resolves_on(name, "Grizzly Bears");
        assert!(!t.on_battlefield(bears));
        assert_eq!(treasures(&t, P0), 1, "{name}");
    }
}

#[test]
fn contract_killing_needs_a_legal_target_but_not_a_destroyed_one() {
    cr!("608.2b", "608.2c", "702.12b");
    ruling!(
        "Contract Killing",
        "If the target creature is an illegal target by the time Contract Killing resolves, the entire spell doesn't resolve. You won't get Treasures. If, on the other hand, the target is a legal target but isn't destroyed (most likely because it has indestructible), you'll get Treasures."
    );
    supported("Contract Killing");
    let (t, _) = fizzles("Contract Killing", "Grizzly Bears");
    assert_eq!(treasures(&t, P0), 0);
    let (t, myr) = resolves_on("Contract Killing", "Darksteel Myr");
    assert!(t.on_battlefield(myr));
    assert_eq!(treasures(&t, P0), 2);
}

#[test]
fn flick_a_coin_does_nothing_without_its_target() {
    cr!("608.2b", "111.10a");
    ruling!(
        "Flick a Coin",
        "If the target is not legal as Flick a Coin tries to resolve, Flick a Coin is removed from the stack. You won't create a Treasure token, and you won't draw a card."
    );
    supported("Flick a Coin");
    let (t, _) = fizzles("Flick a Coin", "Grizzly Bears");
    assert_eq!(treasures(&t, P0), 0);
    assert_eq!(t.hand_size(P0), 0);
    // "Flick a Coin deals 1 damage to any target. You create a Treasure token. Draw a
    // card."
    let (t, bears) = resolves_on("Flick a Coin", "Grizzly Bears");
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(treasures(&t, P0), 1);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn vraskas_minus_three_needs_a_legal_target_but_not_a_destroyed_one() {
    cr!("608.2b", "608.2c", "702.12b", "606.3");
    ruling!(
        "Vraska, Relic Seeker",
        "If the target artifact, creature, or enchantment is an illegal target by the time Vraska's second ability resolves, the entire ability doesn't resolve. You won't get a Treasure. If, on the other hand, the target is a legal target but isn't destroyed (most likely because it has indestructible), you will get a Treasure."
    );
    supported("Vraska, Relic Seeker");
    // "−3: Destroy target artifact, creature, or enchantment. Create a Treasure token."
    for (target, gone_first) in [
        ("Grizzly Bears", true),
        ("Darksteel Myr", false),
        ("Grizzly Bears", false),
    ] {
        let mut t = TestGame::new(2);
        let vraska = t.battlefield(P0, "Vraska, Relic Seeker");
        let victim = t.battlefield(P1, target);
        t.answer_targets(P0, &[Entity::Object(victim)]);
        activate_containing(&mut t, P0, vraska, "Destroy target").unwrap();
        if gone_first {
            destroy(&mut t, victim);
        }
        t.resolve_all();
        let destroyable = target == "Grizzly Bears";
        assert_eq!(treasures(&t, P0), usize::from(!gone_first), "{target}");
        assert_eq!(t.on_battlefield(victim), !destroyable, "{target}");
    }
}

#[test]
fn volatile_fault_needs_a_legal_target_but_the_search_and_treasure_dont_need_it_destroyed() {
    cr!("608.2b", "608.2c", "702.12b", "701.23g");
    ruling!(
        "Volatile Fault",
        "If the target of Volatile Fault's last ability is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't create a Treasure token."
    );
    ruling!(
        "Volatile Fault",
        "If Volatile Fault's ability resolves, the target nonbasic land's controller gets to search for a basic land card even if the land wasn't destroyed by Volatile Fault's ability. This may happen because the land has indestructible. In this case, you'll still create a Treasure token."
    );
    supported("Volatile Fault");
    // "{1}, {T}, Sacrifice this land: Destroy target nonbasic land an opponent controls.
    // That player may search their library for a basic land card, put it onto the
    // battlefield, then shuffle. You create a Treasure token."
    for (target, gone_first) in [
        ("Darksteel Citadel", true),
        ("Darksteel Citadel", false),
        ("Mishra's Factory", false),
    ] {
        let mut t = TestGame::new(2);
        let fault = t.battlefield(P0, "Volatile Fault");
        t.lands(P0, "Wastes", 1);
        let land = t.battlefield(P1, target);
        let forest = t.library_top(P1, "Forest");
        t.answer_targets(P0, &[Entity::Object(land)]);
        activate_containing(&mut t, P0, fault, "Destroy target").unwrap();
        if gone_first {
            move_to(&mut t, land, Zone::Exile);
        }
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[Entity::Object(forest)]);
        t.resolve_all();
        assert!(!t.on_battlefield(fault));
        let resolved = !gone_first;
        assert_eq!(treasures(&t, P0), usize::from(resolved), "{target}");
        assert_eq!(t.on_battlefield(forest), resolved, "{target}");
        if resolved {
            assert_eq!(t.on_battlefield(land), target == "Darksteel Citadel");
        }
    }
}

/// P1 casts the real spell `name` (with lands for it; at P0 if it has a target) in
/// P1's main phase; it's left on the stack.
fn p1_casts(t: &mut TestGame, name: &str) -> ObjectId {
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    give_mana_for(t, P1, name);
    let card = t.hand(P1, name);
    let spell = t.cast(P1, card);
    let spell = if name == "Lightning Bolt" {
        spell.target(P0)
    } else {
        spell
    };
    spell.go()
}

#[test]
fn hornswoggle_gives_a_treasure_even_if_the_spell_cant_be_countered() {
    cr!("608.2c", "701.6a");
    ruling!(
        "Hornswoggle",
        "Hornswoggle can target a spell that can't be countered, such as Nezahal, Primal Tide. When Hornswoggle resolves, that spell won't be countered, but you'll still get a Treasure."
    );
    supported("Hornswoggle");
    // "Counter target creature spell. You create a Treasure token."
    let mut t = TestGame::new(2);
    let tyrant = p1_casts(&mut t, "Carnage Tyrant");
    give_mana_for(&mut t, P0, "Hornswoggle");
    let h = t.hand(P0, "Hornswoggle");
    t.cast(P0, h).target(tyrant).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Carnage Tyrant").len(), 1);
    assert_eq!(treasures(&t, P0), 1);
    // A spell that can be countered is.
    let mut t = TestGame::new(2);
    let bears = p1_casts(&mut t, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Hornswoggle");
    let h = t.hand(P0, "Hornswoggle");
    t.cast(P0, h).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn spell_swindle_gives_treasures_even_if_the_spell_cant_be_countered() {
    cr!("608.2c", "202.3");
    ruling!(
        "Spell Swindle",
        "You may target a spell that can't be countered. When Spell Swindle resolves, the target spell will be unaffected, but you'll still get Treasures."
    );
    supported("Spell Swindle");
    // "Counter target spell. Create X Treasure tokens, where X is that spell's mana
    // value." Carnage Tyrant: mana value 6.
    let mut t = TestGame::new(2);
    let tyrant = p1_casts(&mut t, "Carnage Tyrant");
    give_mana_for(&mut t, P0, "Spell Swindle");
    let s = t.hand(P0, "Spell Swindle");
    t.cast(P0, s).target(tyrant).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Carnage Tyrant").len(), 1);
    assert_eq!(treasures(&t, P0), 6);
}

#[test]
fn an_offer_you_cant_refuse_needs_its_target_but_not_for_it_to_be_countered() {
    cr!("608.2b", "608.2c");
    ruling!(
        "An Offer You Can't Refuse",
        "If the target is still legal as it resolves but the spell can't be countered for some reason, its controller will still create two Treasure tokens."
    );
    ruling!(
        "An Offer You Can't Refuse",
        "If the target is no longer legal as An Offer You Can't Refuse resolves, no Treasure tokens are created."
    );
    supported("An Offer You Can't Refuse");
    // "Counter target noncreature spell. Its controller creates two Treasure tokens."
    // Abrupt Decay can't be countered: P1 still gets two Treasures.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P1, "Abrupt Decay");
    let decay = t.hand(P1, "Abrupt Decay");
    let decay = t.cast(P1, decay).target(bears).go();
    give_mana_for(&mut t, P0, "An Offer You Can't Refuse");
    let offer = t.hand(P0, "An Offer You Can't Refuse");
    t.cast(P0, offer).target(decay).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(treasures(&t, P1), 2);
    assert_eq!(treasures(&t, P0), 0);
    // Lightning Bolt is countered first by Counterspell: no Treasures.
    let mut t = TestGame::new(2);
    let bolt = p1_casts(&mut t, "Lightning Bolt");
    give_mana_for(&mut t, P0, "An Offer You Can't Refuse");
    let offer = t.hand(P0, "An Offer You Can't Refuse");
    t.cast(P0, offer).target(bolt).go();
    give_mana_for(&mut t, P0, "Counterspell");
    let cs = t.hand(P0, "Counterspell");
    t.cast(P0, cs).target(bolt).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "An Offer You Can't Refuse"));
    assert_eq!(treasures(&t, P1), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn gold_rush_counts_treasures_once_and_may_have_no_target() {
    cr!("608.2h", "611.2c", "115.6", "608.2b");
    ruling!(
        "Gold Rush",
        "Count the number of Treasures you control as Gold Rush resolves to determine how big a bonus the target creature receives. That bonus won’t change even if the number of Treasures you control does during the turn."
    );
    ruling!(
        "Gold Rush",
        "You don’t have to choose a target for Gold Rush. However, if you do, and that creature is an illegal target at the time Gold Rush tries to resolve, it won’t resolve and none of its effects will happen. You won’t create a Treasure token."
    );
    supported("Gold Rush");
    // "Create a Treasure token. Until end of turn, up to one target creature gets +2/+2
    // for each Treasure you control." With one Treasure already: two, +4/+4.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    give_mana_for(&mut t, P0, "Gold Rush");
    let g = t.hand(P0, "Gold Rush");
    t.cast(P0, g).target(bears).go();
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 2);
    assert_eq!(t.pt(bears), (6, 6));
    // Sacrificing both Treasures doesn't change it.
    for tr in with_subtype(&t, P0, "Treasure") {
        t.g.sacrifice(tr, P0);
    }
    t.settle();
    assert_eq!(treasures(&t, P0), 0);
    assert_eq!(t.pt(bears), (6, 6));
    // No target: just the Treasure.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Gold Rush");
    let g = t.hand(P0, "Gold Rush");
    t.cast(P0, g).targets(&[]).go();
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    // A target that's gone: nothing.
    let (t, _) = fizzles("Gold Rush", "Grizzly Bears");
    assert_eq!(treasures(&t, P0), 0);
}

#[test]
fn bloodroot_apothecarys_trigger_does_nothing_if_its_target_player_is_illegal() {
    cr!("608.2b", "702.11c", "111.10a");
    ruling!(
        "Bloodroot Apothecary",
        "If the target player is an illegal target as Bloodroot Apothecary's second ability tries to resolve, it won't resolve and none of its effects will happen. No player will create a Treasure token."
    );
    supported("Bloodroot Apothecary");
    // "When this creature enters, you and target opponent each create a Treasure token."
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Bloodroot Apothecary");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // P1 gains hexproof before it resolves.
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 0);
    assert_eq!(treasures(&t, P1), 0);
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Bloodroot Apothecary");
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    assert_eq!(treasures(&t, P1), 1);
}

#[test]
fn fake_your_own_death_gives_the_creatures_controller_a_treasure_even_for_a_token() {
    cr!("603.10a", "111.7", "111.2", "608.2b");
    ruling!(
        "Fake Your Own Death",
        "If the target creature is a token, the ability still triggers when it dies. Its controller won't return the token to the battlefield, but they will creature a Treasure token."
    );
    ruling!(
        "Fake Your Own Death",
        "The Treasure token is created by the creature's controller, who may be different from Fake Your Own Death's controller and may be different from the creature's owner."
    );
    supported("Fake Your Own Death");
    // "Until end of turn, target creature gets +2/+0 and gains "When this creature dies,
    // return it to the battlefield tapped under its owner's control and you create a
    // Treasure token.""
    // P0 casts it on P1's Bears, which P2 owns: P1 creates the Treasure; P2 gets the
    // Bears back.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P2, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, bears, P1);
    give_mana_for(&mut t, P0, "Fake Your Own Death");
    let f = t.hand(P0, "Fake Your Own Death");
    t.cast(P0, f).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(treasures(&t, P1), 1);
    assert_eq!(treasures(&t, P0), 0);
    assert_eq!(treasures(&t, P2), 0);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P2);
    assert!(t.obj(back[0]).tapped);
    // A token: it triggers, the token doesn't return, its controller gets a Treasure.
    let mut t = TestGame::new(2);
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    give_mana_for(&mut t, P0, "Fake Your Own Death");
    let f = t.hand(P0, "Fake Your Own Death");
    t.cast(P0, f).target(token).go();
    t.resolve_all();
    destroy(&mut t, token);
    t.resolve_all();
    assert_eq!(treasures(&t, P1), 1);
    assert!(with_subtype(&t, P1, "Soldier").is_empty());
}
