//! Logika questow.

use super::*;

impl Game {
    pub(crate) fn quest_tick(&mut self, kind: QuestKind, amount: u32) {
        let quest = &QUESTS[self.quest_idx];
        if quest.kind != kind {
            return;
        }
        self.quest_progress += amount;
        if self.quest_progress >= quest.target {
            let reward = quest.reward;
            self.cash += reward;
            self.total_earned += reward;
            self.quest_done += 1;
            self.log(format!("QUEST DONE +${:.0}", reward));
            self.quest_idx = (self.quest_idx + 1) % QUESTS.len();
            self.quest_progress = 0;
        }
    }

    // ---- losowość (xorshift64*) ----
}
