//! Rulings batch P111 — miscellaneous: "lands you control enter untapped" against lands
//! entering at the same time and "enters tapped" replacements (CR 614.12, 616.1); a token
//! copy triggering its own "whenever this or another ... enters" (CR 603.6a, 707.2);
//! "at the beginning of the next end step" (CR 603.7b); last known toughness (CR 608.2h);
//! a target gone means the whole ability does nothing (CR 608.2b); sacrificing nothing
//! (CR 701.21a); sacrificing the source as a cost (CR 603.10a); a token replacement and
//! control-changing effects (CR 614.1a, 616.1, 111.2).

use crate::r_p009_common::kill;
use crate::r_p116_common::enter_together;
use crate::r_s01_common::{attack_with, supported, tokens};
use crate::r_s02_common::create_token;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::activate_containing;
use crate::r_s09_common::to_combat;
use crate::r_s17_common::token_copy;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn mana(t: &mut TestGame, p: PlayerId, cost: &str) {
    for c in cost.chars() {
        let ty = match c {
            'W' => ManaType::W,
            'U' => ManaType::U,
            'B' => ManaType::B,
            'R' => ManaType::R,
            'G' => ManaType::G,
            d => {
                let n = d.to_digit(10).expect("a mana symbol");
                t.g.players[p.idx()].mana_pool.add_type(ManaType::C, n);
                continue;
            }
        };
        t.g.players[p.idx()].mana_pool.add_type(ty, 1);
    }
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn the_wandering_minstrel_and_lands_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "The Wandering Minstrel",
        "If The Wandering Minstrel would enter the battlefield at the same time a land you control would enter the battlefield tapped, that land still enters the battlefield tapped."
    );
    supported("The Wandering Minstrel");
    let mut t = TestGame::new(2);
    let both = enter_together(&mut t, P0, &["The Wandering Minstrel", "Coastal Peak"]);
    assert_eq!(both.len(), 2);
    assert!(t.obj_now(both[1]).tapped);
    // Control: once it's on the battlefield, Coastal Peak may enter untapped (applying
    // its own replacement first, then the Minstrel's).
    let untapped = (0..2).any(|pick| {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        let peak = t.enter(P0, "Coastal Peak");
        t.clear_answers();
        !t.obj_now(peak).tapped
    });
    assert!(untapped);
}

#[test]
fn the_wandering_minstrel_against_enters_tapped_abilities_and_effects() {
    cr!("616.1", "614.12", "614.1c");
    ruling!(
        "The Wandering Minstrel",
        "If a land has an ability that says it enters the battlefield tapped, you choose the order in which that ability's effect and The Wandering Minstrel's effect apply. This means you can choose to have the land enter tapped or untapped. If a land you control is simply put onto the battlefield tapped without a replacement effect being applied, it always enters untapped if you control The Wandering Minstrel."
    );
    supported("Rampant Growth");
    // The order of the two replacement effects: the land's own first (then the Minstrel
    // untaps it), or the Minstrel's first (then it's tapped).
    let mut seen = vec![];
    for pick in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "The Wandering Minstrel");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        let peak = t.enter(P0, "Coastal Peak");
        seen.push(t.obj_now(peak).tapped);
    }
    seen.sort();
    assert_eq!(seen, vec![false, true], "the player chooses");
    // Rampant Growth: "put that card onto the battlefield tapped".
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Wandering Minstrel");
    let forest = t.library_top(P0, "Forest");
    t.lands(P0, "Forest", 2);
    t.answer_choose(P0, &[obj(forest)]);
    let g = t.hand(P0, "Rampant Growth");
    t.cast(P0, g).go();
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert!(!t.obj_now(forest).tapped);
}

#[test]
fn a_token_copy_of_go_shintai_triggers_its_own_ability() {
    cr!("603.6a", "707.2", "111.1");
    ruling!(
        "Go-Shintai of Life's Origin",
        "If a token copy of Go-Shintai of Life's Origin enters the battlefield, it will cause its own triggered ability to trigger."
    );
    supported("Go-Shintai of Life's Origin");
    // "Whenever Go-Shintai of Life's Origin or another nontoken Shrine you control enters,
    // create a 1/1 colorless Shrine enchantment creature token."
    // (The copy is legendary too: the legend rule puts one of them into the graveyard,
    // but the copy's ability has triggered.)
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Go-Shintai of Life's Origin");
    token_copy(&mut t, P0, gs);
    t.resolve_all();
    let shrines: Vec<_> = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.name != "Go-Shintai of Life's Origin")
        .collect();
    assert_eq!(shrines.len(), 1);
    assert_eq!(t.pt(shrines[0]), (1, 1));
    assert!(t.obj(shrines[0]).chars.has_subtype("Shrine"));
}

#[test]
fn a_token_made_after_the_end_step_triggers_lasts_until_the_next_end_step() {
    cr!("603.7b", "513.1a", "603.2");
    ruling!(
        "Balduvian Dead",
        "If the ability is activated during a turn’s Ending phase (after any “at the beginning of the End step” triggers would have triggered that turn), the Graveborn token will remain on the battlefield until the following turn’s End step."
    );
    supported("Balduvian Dead");
    // "{2}{R}, Exile a creature card from your graveyard: Create a 3/1 black and red
    // Graveborn creature token with haste. Sacrifice it at the beginning of the next end
    // step."
    let mut t = TestGame::new(2);
    let dead = t.battlefield(P0, "Balduvian Dead");
    t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    mana(&mut t, P0, "R2");
    activate_containing(&mut t, P0, dead, "Graveborn").unwrap();
    t.resolve_all();
    let graveborn = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Graveborn"))
            .count()
    };
    assert_eq!(graveborn(&t), 1);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(graveborn(&t), 1, "survived this turn's end");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(graveborn(&t), 0);
}

#[test]
fn the_granted_trigger_draws_by_last_known_toughness() {
    cr!("608.2h", "603.10a", "510.3a");
    ruling!(
        "The Pride of Hull Clade",
        "If the target of The Pride of Hull Clade's last ability deals combat damage to a player but leaves the battlefield before the granted triggered ability resolves, use its toughness as it last existed on the battlefield to determine how many cards to draw."
    );
    supported("The Pride of Hull Clade");
    // "{2}{U}{U}: Until end of turn, target creature you control gets +1/+0, gains
    // "Whenever this creature deals combat damage to a player, draw cards equal to its
    // toughness," ..." Hill Giant (3/3).
    let mut t = TestGame::new(2);
    let pride = t.battlefield(P0, "The Pride of Hull Clade");
    let giant = t.battlefield(P0, "Hill Giant");
    mana(&mut t, P0, "UU2");
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, pride, "draw cards").unwrap();
    t.resolve_all();

    to_combat(&mut t, P0);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    let library = t.library_size(P0);
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.stack_len(), 1, "the granted trigger");
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(library - t.library_size(P0), 3);
}

#[test]
fn if_the_only_target_is_gone_nothing_happens() {
    cr!("608.2b");
    ruling!(
        "Creakwood Ghoul",
        "If the targeted card is removed from the graveyard before the ability resolves, the entire ability doesn’t resolve. You won’t gain 1 life."
    );
    ruling!(
        "Bagel and Schmear",
        "If you choose a target for the first ability and that target is illegal as the ability resolves, the ability will be removed from the stack and you will not draw a card."
    );
    supported("Creakwood Ghoul");
    supported("Bagel and Schmear");
    // Creakwood Ghoul: "{B/G}{B/G}: Exile target card from a graveyard. You gain 1 life."
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let ghoul = t.battlefield(P0, "Creakwood Ghoul");
        let card = t.graveyard(P1, "Grizzly Bears");
        mana(&mut t, P0, "BB");
        t.answer_targets(P0, &[obj(card)]);
        activate_containing(&mut t, P0, ghoul, "Exile target card").unwrap();
        if gone {
            move_to(&mut t, card, mtg_engine::object::Zone::Hand(P1));
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if gone { 20 } else { 21 });
    }
    // Bagel and Schmear: "Share — {W}, {T}, Sacrifice this artifact: Put a +1/+1 counter
    // on up to one target creature. Draw a card. Activate only as a sorcery."
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let bagel = t.battlefield(P0, "Bagel and Schmear");
        let bears = t.battlefield(P0, "Grizzly Bears");
        mana(&mut t, P0, "W");
        t.answer_targets(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, bagel, "+1/+1 counter").unwrap();
        if gone {
            kill(&mut t, bears);
        }
        let library = t.library_size(P0);
        t.resolve_all();
        assert_eq!(library - t.library_size(P0), if gone { 0 } else { 1 });
    }
    // With no target chosen, it draws.
    let mut t = TestGame::new(2);
    let bagel = t.battlefield(P0, "Bagel and Schmear");
    mana(&mut t, P0, "W");
    t.answer_targets(P0, &[]);
    activate_containing(&mut t, P0, bagel, "+1/+1 counter").unwrap();
    let library = t.library_size(P0);
    t.resolve_all();
    assert_eq!(library - t.library_size(P0), 1);
}

#[test]
fn sacrificing_a_homunculus_you_dont_have_does_nothing() {
    cr!("701.21a", "101.3");
    ruling!(
        "Puppet Conjurer",
        "If you control no Homunculi when the second ability resolves, nothing happens. There’s no penalty for failing to sacrifice a Homunculus."
    );
    supported("Puppet Conjurer");
    // "At the beginning of your upkeep, sacrifice a Homunculus."
    let mut t = TestGame::new(2);
    let pc = t.battlefield(P0, "Puppet Conjurer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::Draw);
    assert!(t.on_battlefield(pc) && t.on_battlefield(bears));
    // Control: with a Homunculus token, it's sacrificed.
    mana(&mut t, P0, "U");
    activate_containing(&mut t, P0, pc, "Homunculus").unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert!(tokens(&t, P0).is_empty());
    assert!(t.on_battlefield(pc));
}

#[test]
fn kels_sees_itself_and_creatures_sacrificed_with_it() {
    cr!("603.10a", "602.2b", "701.21a");
    ruling!(
        "Kels, Fight Fixer",
        "If you sacrifice Kels, its middle ability triggers. It also triggers for any other creatures you sacrifice at the same time. You can sacrifice Kels to pay the cost of its own ability."
    );
    supported("Kels, Fight Fixer");
    supported("Barter in Blood");
    // "Whenever you sacrifice a creature, you may pay {U/B}. If you do, draw a card."
    // "{1}, Sacrifice a creature: Kels gains indestructible until end of turn."
    let mut t = TestGame::new(2);
    let kels = t.battlefield(P0, "Kels, Fight Fixer");
    t.lands(P0, "Swamp", 2);
    mana(&mut t, P0, "1");
    t.answer_choose(P0, &[obj(kels)]);
    t.answer_yes(P0, true);
    let library = t.library_size(P0);
    activate_containing(&mut t, P0, kels, "indestructible").unwrap();
    assert!(t.in_graveyard(P0, "Kels, Fight Fixer"));
    t.resolve_all();
    assert_eq!(library - t.library_size(P0), 1);
    // Barter in Blood: "Each player sacrifices two creatures." Kels and the Bears: two
    // triggers.
    let mut t = TestGame::new(2);
    let kels = t.battlefield(P0, "Kels, Fight Fixer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 6);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let bib = t.hand(P0, "Barter in Blood");
    t.cast(P0, bib).go();
    let library = t.library_size(P0);
    t.resolve_all();
    assert!(!t.on_battlefield(kels) && !t.on_battlefield(bears));
    assert_eq!(library - t.library_size(P0), 2);
}

#[test]
fn chatterfang_and_every_kind_of_token() {
    cr!("614.1a", "111.10a", "616.1");
    ruling!(
        "Chatterfang, Squirrel General",
        "Chatterfang's second ability applies to all kinds of tokens, including Clue, Food, and Treasure tokens."
    );
    ruling!(
        "Chatterfang, Squirrel General",
        "If an effect changes under whose control a token would be created, that effect applies before Chatterfang's effect applies. If an effect changes under whose control a token would enter the battlefield, that effect applies after Chatterfang's effect is able to be applied."
    );
    supported("Chatterfang, Squirrel General");
    supported("Crafty Cutpurse");
    supported("Gather Specimens");
    let squirrels = |t: &TestGame, p: PlayerId| {
        tokens(t, p)
            .into_iter()
            .filter(|id| t.obj(*id).chars.has_subtype("Squirrel"))
            .count()
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chatterfang, Squirrel General");
    create_token(&mut t, P0, "Treasure");
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(squirrels(&t, P0), 1);
    // Crafty Cutpurse: "each token that would be created under an opponent's control
    // this turn is created under your control instead." P1's token is created under P0's
    // control: Chatterfang applies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chatterfang, Squirrel General");
    enter(&mut t, P0, "Crafty Cutpurse");
    t.resolve_all();
    create_token(&mut t, P1, "Food");
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(squirrels(&t, P0), 1);
    assert!(tokens(&t, P1).is_empty());
    // Gather Specimens: "If a creature would enter the battlefield under an opponent's
    // control this turn, it enters the battlefield under your control instead." P0's
    // token is created under P0's control (Chatterfang applies), then both enter under
    // P1's control.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chatterfang, Squirrel General");
    let gs = t.hand(P1, "Gather Specimens");
    mana(&mut t, P1, "UUU3");
    t.cast(P1, gs).go();
    t.resolve_all();
    create_token(&mut t, P0, "Soldier");
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(tokens(&t, P1).len(), 2);
    assert_eq!(squirrels(&t, P1), 1);
}

