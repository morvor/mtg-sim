//! Rulings batch P142 — casting with engines: cascade granted by a permanent (it must be
//! there as the spell is cast, not as cascade resolves), permissions to play cards from
//! graveyards (each is its own permission, a new object has a new one, losing the source
//! mid-cast doesn't matter), spells cast from a library during a resolution, copies cast
//! during a chapter ability, and triggers on casting.

use crate::r_p142_common::*;
use crate::r_s01_common::watch;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::ability::ZoneKind;
use mtg_engine::*;

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, m: CastMethod) -> bool {
    crate::r_s02_common::can_cast(t, p, card, m)
}

/// The number of cascade triggers on the stack.
fn cascades(t: &TestGame) -> usize {
    triggers_on_stack(t, "ascade")
}

#[test]
fn wildsear_sacrificed_while_casting_gives_no_cascade() {
    cr!("702.85a", "601.2h", "601.2i");
    ruling!(
        "Wildsear, Scouring Maw",
        "In some unusual situations, you may lose control of Wildsear during the process of casting an enchantment spell from your hand. In those situations, that spell won't have cascade."
    );
    supported("Wildsear, Scouring Maw");
    supported("Grafted Identity");
    for sac_wildsear in [true, false] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Wildsear, Scouring Maw");
        let b = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        stack_library(&mut t, P0, &["Opt", "Opt"]);
        lots(&mut t, P0);
        let gi = t.hand(P0, "Grafted Identity");
        t.answer_choose(P0, &[obj(if sac_wildsear { w } else { b })]);
        t.cast(P0, gi).target(giant).go();
        t.settle();
        assert_eq!(cascades(&t), usize::from(!sac_wildsear), "{sac_wildsear}");
    }
}

#[test]
fn cascade_granters_leaving_after_the_cast_dont_stop_cascade() {
    cr!("702.85a", "603.2", "113.7a");
    ruling!(
        "Wildsear, Scouring Maw",
        "Once you announce that you're casting a spell, players can't take actions until you've finished casting it. Causing Wildsear to leave the battlefield after you've cast an enchantment spell from your hand won't stop that spell's cascade ability from resolving."
    );
    ruling!(
        "Zhulodok, Void Gorger",
        "Once you announce that you're casting a spell, players can't take action until you've finished casting it. Causing Zhulodok, Void Gorger to leave the battlefield after you've cast a colorless spell from your hand with mana value 7 or greater won't stop that spell's cascade abilities from resolving."
    );
    supported("Zhulodok, Void Gorger");
    // Wildsear: Pacifism cascades into Opt.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wildsear, Scouring Maw");
    let giant = t.battlefield(P1, "Hill Giant");
    stack_library(&mut t, P0, &["Opt"]);
    lots(&mut t, P0);
    yes(&mut t, P0);
    let p = t.hand(P0, "Pacifism");
    t.cast(P0, p).target(giant).go();
    t.settle();
    assert_eq!(cascades(&t), 1);
    destroy(&mut t, w);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Opt"), "Opt was cast");
    // Zhulodok: Ulamog's Crusher cascades twice.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zhulodok, Void Gorger");
    stack_library(&mut t, P0, &["Ornithopter", "Ornithopter"]);
    lots(&mut t, P0);
    lots(&mut t, P0);
    yes(&mut t, P0);
    yes(&mut t, P0);
    let c = t.hand(P0, "Ulamog's Crusher");
    t.cast(P0, c).go();
    t.settle();
    assert_eq!(cascades(&t), 2);
    destroy(&mut t, z);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 2);
}

#[test]
fn zhulodok_cascade_into_a_split_card_casts_one_half() {
    cr!("702.85a", "709.3", "202.3c");
    ruling!(
        "Zhulodok, Void Gorger",
        "The mana value of a split card is determined by the combined mana value of its two halves. If cascade allows you to cast a split card, you may cast either half but not both halves."
    );
    // Fire // Ice (mana value 4) is found under Ulamog's Crusher (8); Ice is cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zhulodok, Void Gorger");
    let giant = t.battlefield(P1, "Hill Giant");
    let fi = t.library_top(P0, "Fire // Ice");
    assert_eq!(t.g.mana_value_of(fi), 4);
    stack_library(&mut t, P0, &["Opt"]);
    // Opt is on top now; Fire // Ice under it: the first cascade finds Opt.
    lots(&mut t, P0);
    lots(&mut t, P0);
    t.answer_yes(P0, false);
    let c = t.hand(P0, "Ulamog's Crusher");
    t.cast(P0, c).go();
    t.settle();
    t.resolve();
    // The second cascade: cast Ice (index 1) targeting the Giant.
    let hand = t.hand_size(P0);
    yes(&mut t, P0);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped, "Ice tapped the Giant");
    assert_eq!(t.obj_now(giant).damage, 0, "Fire wasn't cast too");
    assert_eq!(t.hand_size(P0), hand + 1, "Ice drew a card");
    assert!(t.in_graveyard(P0, "Fire // Ice"));
}

#[test]
fn yidris_cascade_lasts_after_yidris_leaves() {
    cr!("611.2a", "702.85a");
    ruling!(
        "Yidris, Maelstrom Wielder",
        "Spells you cast from your hand this turn will gain cascade even if Yidris leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Yidris, Maelstrom Wielder");
    t.attack(&[(y, Entity::Player(P1))], &[]);
    t.resolve_all();
    destroy(&mut t, y);
    t.advance_to(P0, Step::PostcombatMain);
    stack_library(&mut t, P0, &["Opt"]);
    yes(&mut t, P0);
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(cascades(&t), 1);
}

#[test]
fn maelstrom_nexus_counts_spells_cast_before_it_entered() {
    cr!("702.85a", "601.2i");
    ruling!(
        "Maelstrom Nexus",
        "This effect takes into account spells that were cast earlier in the turn before Maelstrom Nexus entered the battlefield, including any spells still on the stack. If you've already cast any spells that turn (including Maelstrom Nexus itself), this ability won't give any of your spells cascade that turn."
    );
    // Cast Maelstrom Nexus itself: the next spell has no cascade.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Opt", "Opt"]);
    lots(&mut t, P0);
    let n = t.hand(P0, "Maelstrom Nexus");
    t.cast(P0, n).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Maelstrom Nexus").len(), 1);
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(cascades(&t), 0);
    // An Opt still on the stack as the Nexus enters counts too.
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.battlefield(P0, "Maelstrom Nexus");
    cast_new(&mut t, P0, "Think Twice", &[]);
    t.settle();
    assert_eq!(cascades(&t), 0);
    // Positive control: the first spell of the turn cascades.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maelstrom Nexus");
    stack_library(&mut t, P0, &["Opt"]);
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(cascades(&t), 1);
}

#[test]
fn smoldering_stagecoach_cascade_exiles_even_if_nothing_is_cast_and_its_power_counts_everywhere(
) {
    cr!("702.85a", "604.3");
    ruling!(
        "Smoldering Stagecoach",
        "When the cascade ability resolves, you must exile cards. The only optional part of the ability is whether you cast the last card exiled."
    );
    ruling!(
        "Smoldering Stagecoach",
        "The ability that defines Smoldering Stagecoach's power works in all zones, not just the battlefield."
    );
    // Power in the hand and graveyard.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Opt");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Forest");
    let h = t.hand(P0, "Smoldering Stagecoach");
    assert_eq!(t.pt(h).0, 2);
    let g = t.graveyard(P0, "Smoldering Stagecoach");
    assert_eq!(t.pt(g).0, 2);
    // Declining the cast: the cards are still exiled and put on the bottom.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Smoldering Stagecoach");
    let crew = t.battlefield(P0, "Hill Giant");
    assert!(crate::r_s04_common::crew(&mut t, P0, s, &[crew]));
    t.resolve_all();
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    stack_library(&mut t, P0, &["Forest", "Opt"]);
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(cascades(&t), 1);
    t.resolve();
    let names = library_names(&t, P0);
    assert_ne!(names[0], "Forest");
    let bottom2: Vec<&String> = names.iter().rev().take(2).collect();
    assert!(bottom2.contains(&&"Forest".to_string()));
    assert!(bottom2.contains(&&"Opt".to_string()));
}

/// P0 with `engines` on the battlefield (returned in order) and the cards `milled` (top
/// first) milled this turn (returned as they are in the graveyard).
fn graveyard_engines(
    t: &mut TestGame,
    engines: &[&str],
    milled: &[&str],
) -> (Vec<ObjectId>, Vec<ObjectId>) {
    let e: Vec<ObjectId> = engines.iter().map(|n| t.battlefield(P0, n)).collect();
    let cards = stack_library(t, P0, milled);
    mill(t, P0, milled.len() as u32);
    let cards = cards.iter().map(|c| now(t, *c)).collect();
    (e, cards)
}

#[test]
fn kagha_and_coram_permissions_are_separate() {
    cr!("601.3", "305.1");
    ruling!(
        "Kagha, Shadow Archdruid",
        "If you play a land or cast a permanent spell from your graveyard using another permission, it won't count as the \"once during each of your turns\" for Kagha's effect."
    );
    ruling!(
        "Coram, the Undertaker",
        "If you play a land or cast a spell from a graveyard using another permission, it won't stop you from using Coram's permission to do the same thing later in the turn, provided you haven't already done so."
    );
    supported("Kagha, Shadow Archdruid");
    supported("Coram, the Undertaker");
    let mut t = TestGame::new(2);
    let (e, cards) = graveyard_engines(
        &mut t,
        &["Kagha, Shadow Archdruid", "Coram, the Undertaker"],
        &["Forest", "Grizzly Bears", "Hill Giant"],
    );
    let (kagha, coram) = (e[0], e[1]);
    lots(&mut t, P0);
    // The land with Coram's permission.
    t.answer_choose(P0, &[obj(coram)]);
    t.play_land(P0, cards[0]).expect("Forest with Coram");
    // Kagha's permission is still unused: Grizzly Bears with it.
    t.answer_choose(P0, &[obj(kagha)]);
    t.cast(P0, cards[1]).go();
    t.resolve_all();
    // Coram's spell permission is still unused: Hill Giant with it.
    assert!(can_cast(&mut t, P0, cards[2], CastMethod::Normal));
    t.cast(P0, cards[2]).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Both used now.
    let more = t.library_top(P0, "Craw Wurm");
    mill(&mut t, P0, 1);
    let more = now(&t, more);
    assert!(!can_cast(&mut t, P0, more, CastMethod::Normal));
}

#[test]
fn a_new_kagha_or_coram_has_a_new_permission() {
    cr!("400.7", "601.3");
    ruling!(
        "Kagha, Shadow Archdruid",
        "If you play a land or cast a permanent spell from your graveyard via Kagha, and then have a new Kagha come under your control in the same turn, you may do so again that turn."
    );
    ruling!(
        "Coram, the Undertaker",
        "If you play a land, cast a spell, or both using the permission granted by Coram's last ability and then have a new Coram come under your control in the same turn, you can still play a land, cast a spell, or both using the new Coram's permission."
    );
    for engine in ["Kagha, Shadow Archdruid", "Coram, the Undertaker"] {
        let mut t = TestGame::new(2);
        let (e, cards) = graveyard_engines(&mut t, &[engine], &["Grizzly Bears", "Hill Giant"]);
        lots(&mut t, P0);
        t.cast(P0, cards[0]).go();
        t.resolve_all();
        assert!(!can_cast(&mut t, P0, cards[1], CastMethod::Normal), "{engine}");
        destroy(&mut t, e[0]);
        t.battlefield(P0, engine);
        assert!(can_cast(&mut t, P0, cards[1], CastMethod::Normal), "{engine}");
        t.cast(P0, cards[1]).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1, "{engine}");
    }
}

#[test]
fn losing_kagha_or_coram_while_casting_doesnt_stop_the_spell() {
    cr!("601.2h", "601.3");
    ruling!(
        "Kagha, Shadow Archdruid",
        "Once you begin to cast the spell, losing control of Kagha won't affect it. You can finish casting it as normal."
    );
    ruling!(
        "Coram, the Undertaker",
        "Once you begin to cast a spell this way, losing control of Coram won't affect it. You can finish casting it as normal."
    );
    for engine in ["Kagha, Shadow Archdruid", "Coram, the Undertaker"] {
        // Grafted Identity from the graveyard, sacrificing the engine to cast it.
        let mut t = TestGame::new(2);
        let (e, cards) = graveyard_engines(&mut t, &[engine], &["Grafted Identity"]);
        let giant = t.battlefield(P1, "Hill Giant");
        lots(&mut t, P0);
        t.answer_choose(P0, &[obj(e[0])]);
        let spell = t.cast(P0, cards[0]).target(giant).try_go();
        assert!(spell.is_ok(), "{engine}: {spell:?}");
        assert!(t.in_graveyard(P0, engine));
        t.resolve_all();
        assert_eq!(t.obj_now(giant).controller, P0, "{engine}");
    }
}

#[test]
fn coram_power_follows_the_graveyards() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Coram, the Undertaker",
        "The value of X will change as the greatest power among creature cards in all graveyards changes."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Coram, the Undertaker");
    assert_eq!(t.pt(c), (0, 5));
    t.graveyard(P1, "Hill Giant");
    assert_eq!(t.pt(c), (3, 5));
    let w = t.graveyard(P0, "Craw Wurm");
    assert_eq!(t.pt(c), (6, 5));
    exile(&mut t, w);
    assert_eq!(t.pt(c), (3, 5));
}

#[test]
fn intet_card_played_leaves_exile_for_good() {
    cr!("400.7", "601.2a");
    ruling!(
        "Intet, the Dreamer",
        "Once you play a card this way, it leaves the Exile zone and goes to the stack. You won't be able to play it again."
    );
    let mut t = TestGame::new(2);
    let i = t.battlefield(P0, "Intet, the Dreamer");
    t.lands(P0, "Island", 3);
    t.library_top(P0, "Grizzly Bears");
    yes(&mut t, P0);
    t.attack(&[(i, Entity::Player(P1))], &[]);
    t.resolve_all();
    let card = t.g.exile.iter().copied().find(|id| t.g.obj(*id).face_down).expect("exiled");
    t.advance_to(P0, Step::PostcombatMain);
    let spell = t.cast(P0, card).method(CastMethod::Free).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(t.g.exile.is_empty());
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    destroy(&mut t, b);
    let in_gy = t.g.player(P0).graveyard.last().copied().unwrap();
    assert!(!can_cast(&mut t, P0, in_gy, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, in_gy, CastMethod::Normal));
}

#[test]
fn spells_cast_from_among_looked_at_cards_are_cast_from_the_library() {
    cr!("601.2a", "608.2g", "401.4");
    ruling!(
        "Aetherworks Marvel",
        "The card cast with Aetherworks Marvel's second ability is cast from your library."
    );
    ruling!(
        "Svella, Ice Shaper",
        "The spell you cast with Svella’s last ability is cast from your library."
    );
    ruling!(
        "Svella, Ice Shaper",
        "In some unusual cases, the order of your library will matter as you cast the spell from among the revealed cards. Remember that looking at the top cards of your library doesn’t change their order or allow you to change their order. You don’t put the cards on the bottom of your library until after you are done casting the spell."
    );
    supported("Aetherworks Marvel");
    // Aetherworks Marvel: Grizzly Bears is cast from the library.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Aetherworks Marvel");
    give_player_counters(&mut t, P0, "energy", 6);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    t.answer_choose_moved(P0, &[obj(cards[1])]);
    yes(&mut t, P0);
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    let cast = t.obj_now(b[0]).cast.clone().expect("it was cast");
    assert_eq!(cast.from, Some(ZoneKind::Library));
    // Svella: Shock is cast from the library; as its target is chosen, the other three
    // cards are still on top in their order.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Svella, Ice Shaper");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Forest", 4);
    let cards = stack_library(&mut t, P0, &["Shock", "Forest", "Island", "Swamp", "Plains"]);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| {
            let lib = &g.player(PlayerId(0)).library;
            lib.iter()
                .rev()
                .take(3)
                .map(|id| g.obj(*id).chars.name.to_string())
                .collect::<Vec<_>>()
        },
    );
    t.answer_choose_moved(P0, &[obj(cards[0])]);
    yes(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from_lib = std::sync::Arc::new(std::sync::Mutex::new(None));
    t.activate(P0, s, 1, &[]).unwrap();
    t.resolve_top();
    // Shock is on the stack, cast from the library.
    let shock = t.g.stack.last().copied().expect("Shock on the stack");
    *from_lib.lock().unwrap() = t.g.obj(shock).stack.as_ref().map(|si| si.cast.from);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(*from_lib.lock().unwrap(), Some(Some(ZoneKind::Library)));
    assert_eq!(
        seen.lock().unwrap()[0],
        vec!["Forest", "Island", "Swamp"],
        "still on top while casting"
    );
    assert_eq!(top_name(&t, P0), "Plains");
}

#[test]
fn aetherworks_marvel_gets_energy_for_dying_tokens() {
    cr!("111.7", "700.4");
    ruling!(
        "Aetherworks Marvel",
        "Tokens that are sacrificed or destroyed are put into their owner's graveyard before ceasing to exist. If you controlled the token, Aetherworks Marvel's first ability will trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aetherworks Marvel");
    let tok = create_token(&mut t, P0, "Spirit");
    destroy(&mut t, tok);
    t.resolve_all();
    assert_eq!(player_counter(&t, P0, "energy"), 1);
}

#[test]
fn tale_of_tamiyo_copies_are_cast_during_the_chapter_or_cease_to_exist() {
    cr!("707.12", "704.5d", "714.2b");
    ruling!(
        "The Tale of Tamiyo",
        "If you don't want to cast some (or all) of the copies, you may choose not to. Any copies that you didn't cast cease to exist the next time state-based actions are checked."
    );
    ruling!(
        "The Tale of Tamiyo",
        "You cast the copies while the last chapter ability of The Tale of Tamiyo is resolving and still on the stack. You can't wait to cast them later in the turn."
    );
    let mut t = TestGame::new(2);
    let tale = t.battlefield(P0, "The Tale of Tamiyo");
    put_counters(&mut t, tale, "lore", 3);
    t.resolve_all();
    let shock = t.graveyard(P0, "Shock");
    let opt = t.graveyard(P0, "Opt");
    t.answer_targets(P0, &[obj(shock), obj(opt)]);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { text, .. } if text.contains("any target")),
        |g| g.stack.len(),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // Cast the copies: the Shock, not the Opt.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    lots(&mut t, P0);
    let hand = t.hand_size(P0);
    put_counters(&mut t, tale, "lore", 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "the Shock copy was cast");
    assert_eq!(t.hand_size(P0), hand, "the Opt copy wasn't");
    // While its target was chosen, the chapter ability was still on the stack (with the
    // copy being cast).
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 1);
    assert!(seen[0] >= 2, "{seen:?}");
    assert!(t.in_exile("Shock") && t.in_exile("Opt"));
    // No copy remains anywhere, and nothing can be cast later.
    assert!(t.g.stack.is_empty());
    let copies = t
        .g
        .objects
        .iter()
        .filter(|o| {
            o.kind == mtg_engine::object::ObjKind::SpellCopy
                && matches!(
                    o.zone,
                    Zone::Stack | Zone::Exile | Zone::Graveyard(_) | Zone::Hand(_)
                )
        })
        .count();
    assert_eq!(copies, 0);
}

#[test]
fn possibility_storm_triggers_on_a_miracle_cast_from_the_hand() {
    cr!("702.94a", "601.2a");
    ruling!(
        "Possibility Storm",
        "Instant and sorcery cards with miracle allow a player to cast a card immediately upon drawing it. If a player casts a spell this way, the spell is cast from that player's hand. Possibility Storm will trigger."
    );
    supported("Temporal Mastery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Possibility Storm");
    t.lands(P0, "Island", 2);
    t.set_step(P1, Step::End);
    stack_library(&mut t, P0, &["Temporal Mastery", "Opt"]);
    // Reveal it for miracle, cast it; then decline casting Opt from Possibility Storm.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    // Possibility Storm exiled the miracle spell (then put it on the bottom): it never
    // resolved.
    assert!(t.dump_log().contains("Resolving Possibility Storm ability"));
    assert!(!t.in_graveyard(P0, "Temporal Mastery"));
    assert_eq!(library_names(&t, P0).iter().filter(|n| *n == "Temporal Mastery").count(), 1);
}

#[test]
fn mindmoil_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Mindmoil",
        "Whenever you cast a spell, Mindmoil’s ability triggers. The ability will resolve before the spell does."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindmoil");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.hand(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Island"]);
    let spell = t.cast(P0, opt).go();
    t.settle();
    assert_eq!(t.g.stack.first().copied(), Some(spell));
    t.resolve();
    assert_eq!(hand_names(&t, P0), vec!["Island"]);
    assert!(t.g.stack.contains(&spell));
}

#[test]
fn teferis_puzzle_box_after_the_draw_and_each_separately() {
    cr!("504.1", "603.2", "503.1");
    ruling!(
        "Teferi's Puzzle Box",
        "If you have more than one of these, each effect triggers separately."
    );
    ruling!(
        "Teferi's Puzzle Box",
        "You do your normal draw before this ability is put on the stack."
    );
    supported("Teferi's Puzzle Box");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi's Puzzle Box");
    t.battlefield(P0, "Teferi's Puzzle Box");
    t.hand(P0, "Shock");
    t.set_step(P1, Step::End);
    stack_library(&mut t, P0, &["Opt", "Island", "Forest", "Mountain", "Plains"]);
    t.advance_to(P0, Step::Draw);
    t.settle();
    assert_eq!(stacked_triggers(&t), 2);
    // The normal draw happened first.
    let mut h = hand_names(&t, P0);
    h.sort();
    assert_eq!(h, vec!["Opt", "Shock"]);
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    let mut h = hand_names(&t, P0);
    h.sort();
    assert_eq!(h, vec!["Forest", "Island"]);
    t.resolve();
    let mut h = hand_names(&t, P0);
    h.sort();
    assert_eq!(h, vec!["Mountain", "Plains"]);
}
