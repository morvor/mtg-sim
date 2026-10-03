//! Rulings batch P225 — copies of transforming double-faced permanents: a card that isn't
//! a double-faced card can't be put onto the battlefield transformed and stays where it
//! is (CR 712.14a, 701.27c), and becoming a copy of a back face isn't transforming
//! (CR 701.27e, 707.2).

use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s13_common::add;
use crate::r_s17_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const BOLAS: &str = "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen";
const EGG: &str = "Biolume Egg // Biolume Serpent";
const DESERTER: &str = "Afflicted Deserter // Werewolf Ransacker";

/// P0's Grizzly Bears becomes a copy of P1's `name` (P1's, so the legend rule doesn't
/// apply), and is returned.
fn bears_copying(t: &mut TestGame, name: &str) -> ObjectId {
    let original = t.battlefield(P1, name);
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(t, bears, original);
    assert_eq!(t.obj_now(bears).chars.name, t.obj_now(original).chars.name);
    bears
}

#[test]
fn a_single_faced_copy_of_an_ojer_god_isnt_returned_when_it_dies() {
    cr!("712.14a", "707.2");
    ruling!(
        "Aclazotz, Deepest Betrayal // Temple of the Dead",
        "If a card that isn't a transforming double-faced card is a copy of Aclazotz, Deepest Betrayal, it won't return to the battlefield when it dies."
    );
    ruling!(
        "Ojer Axonil, Deepest Might // Temple of Power",
        "If a card that isn't a transforming double-faced card is a copy of Ojer Axonil, Deepest Might, it won't return to the battlefield when it dies."
    );
    ruling!(
        "Ojer Kaslem, Deepest Growth // Temple of Cultivation",
        "If a card that isn't a transforming double-faced card is a copy of Ojer Kaslem, Deepest Growth, it won't return to the battlefield when it dies."
    );
    ruling!(
        "Ojer Pakpatiq, Deepest Epoch // Temple of Cyclical Time",
        "If a card that isn't a transforming double-faced card is a copy of Ojer Pakpatiq, Deepest Epoch, it won't return to the battlefield when it dies."
    );
    ruling!(
        "Ojer Taq, Deepest Foundation // Temple of Civilization",
        "If a card that isn't a transforming double-faced card is a copy of Ojer Taq, Deepest Foundation, it won't return to the battlefield when it dies."
    );
    // (Aclazotz's attack trigger and Ojer Axonil's other abilities don't compile; their
    // dies triggers do.)
    for name in [
        "Aclazotz, Deepest Betrayal // Temple of the Dead",
        "Ojer Axonil, Deepest Might // Temple of Power",
        "Ojer Kaslem, Deepest Growth // Temple of Cultivation",
        "Ojer Pakpatiq, Deepest Epoch // Temple of Cyclical Time",
        "Ojer Taq, Deepest Foundation // Temple of Civilization",
    ] {
        let mut t = TestGame::new(2);
        let bears = bears_copying(&mut t, name);
        crate::r_s02_common::destroy(&mut t, bears);
        assert_eq!(triggers_on_stack(&t, "transformed"), 1, "{name}");
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Graveyard(P0), "{name}");
        // The real god does return, transformed.
        let god = t.battlefield(P0, name);
        crate::r_s02_common::destroy(&mut t, god);
        t.resolve_all();
        assert!(t.on_battlefield(god), "{name}");
        assert!(t.obj_now(god).tapped, "{name}");
    }
}

#[test]
fn a_single_faced_copy_of_biolume_egg_stays_in_the_graveyard() {
    cr!("712.14a", "603.7");
    ruling!(
        "Biolume Egg // Biolume Serpent",
        "that card can't be returned to the battlefield transformed. It will remain in the graveyard."
    );
    supported(EGG);
    let mut t = TestGame::new(2);
    let bears = bears_copying(&mut t, EGG);
    t.g.sacrifice(bears, P0);
    t.settle();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert!(t.named_on_battlefield("Biolume Serpent").is_empty());
}

#[test]
fn a_single_faced_copy_of_nicol_bolas_is_only_exiled() {
    cr!("712.14a");
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "activating the ability to exile it and return it transformed will only exile it"
    );
    supported(BOLAS);
    let mut t = TestGame::new(2);
    let bears = bears_copying(&mut t, BOLAS);
    for land in ["Island", "Swamp", "Mountain"] {
        t.lands(P0, land, 1);
    }
    t.lands(P0, "Wastes", 4);
    t.set_step(P0, Step::PrecombatMain);
    activate_containing(&mut t, P0, bears, "{4}{U}{B}{R}").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn a_single_faced_copy_of_an_avatar_legend_saga_stays_in_exile() {
    cr!("712.14a", "714.2b");
    ruling!(
        "The Legend of Kuruk // Avatar Kuruk",
        "if a single-faced card is a copy of The Legend of Kuruk, the chapter III ability will cause it to be exiled and then remain in exile"
    );
    ruling!(
        "The Legend of Kyoshi // Avatar Kyoshi",
        "if a single-faced card is a copy of The Legend of Kyoshi, the chapter III ability will cause it to be exiled and then remain in exile"
    );
    ruling!(
        "The Legend of Roku // Avatar Roku",
        "if a single-faced card is a copy of The Legend of Roku, the chapter III ability will cause it to be exiled and then remain in exile"
    );
    ruling!(
        "The Legend of Yangchen // Avatar Yangchen",
        "if a single-faced card is a copy of The Legend of Yangchen, the chapter III ability will cause it to be exiled and then remain in exile"
    );
    // (The Legend of Yangchen's chapter I doesn't compile; its chapter III does.)
    for name in [
        "The Legend of Kuruk // Avatar Kuruk",
        "The Legend of Kyoshi // Avatar Kyoshi",
        "The Legend of Roku // Avatar Roku",
        "The Legend of Yangchen // Avatar Yangchen",
    ] {
        let mut t = TestGame::new(2);
        let bears = bears_copying(&mut t, name);
        // Two lore counters already (no chapter abilities), then the third.
        t.g.objects[bears.0 as usize]
            .counters
            .insert(counters::LORE.into(), 2);
        add(&mut t, bears, counters::LORE, 1);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "III — Exile ~"), 1, "{name}");
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Exile, "{name}");
    }
}

#[test]
fn becoming_a_copy_of_werewolf_ransacker_isnt_transforming_into_it() {
    cr!("701.27e", "707.2");
    ruling!(
        "Afflicted Deserter // Werewolf Ransacker",
        "If something becomes a copy of Werewolf Ransacker, that doesn't count as \"transforming into Werewolf Ransacker.\""
    );
    supported(DESERTER);
    let mut t = TestGame::new(2);
    let deserter = t.battlefield(P1, DESERTER);
    t.battlefield(P0, "Ornithopter");
    transform(&mut t, deserter);
    assert_eq!(t.stack_len(), 1, "the real one transforming triggers");
    t.answer_yes(P1, false);
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(&mut t, bears, deserter);
    assert_eq!(name_of(&t, bears), "Werewolf Ransacker");
    assert_eq!(t.stack_len(), 0);
}
