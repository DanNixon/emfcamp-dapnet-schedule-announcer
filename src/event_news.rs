use dapnet_api::{OutgoingNews, OutgoingNewsBuilder};
use emfcamp_schedule_api::schedule::event::Event;

pub(crate) trait EventExt {
    fn to_rubric_news(&self) -> OutgoingNews;
}

impl EventExt for Event {
    fn to_rubric_news(&self) -> OutgoingNews {
        let venue = shorten_venue_name(&self.venue);
        let msg = format!("{venue}: {}", self.title);

        OutgoingNewsBuilder::default()
            .rubric("emfcamp".to_string())
            .number(news_number_for_venue(&self.venue))
            .text(msg)
            .build()
            .expect("outgoing news should be built")
    }
}

mod venues {
    // TODO: check that this is actually how they appear in the schedule
    pub(crate) const STAGE_A: &str = "Stage A";
    pub(crate) const STAGE_B: &str = "Stage B";
    pub(crate) const STAGE_C: &str = "Stage C";
    pub(crate) const WORKSHOP_1: &str = "Workshop 1 (NottingHack)";
    pub(crate) const WORKSHOP_2: &str = "Workshop 2";
    pub(crate) const WORKSHOP_3: &str = "Workshop 3 (Furry High Commission)";
    pub(crate) const WORKSHOP_4: &str = "Workshop 4";
    pub(crate) const WORKSHOP_5: &str = "Workshop 5";
    pub(crate) const NULL_SECTOR: &str = "Null Sector";
}

fn news_number_for_venue(venue: &str) -> i8 {
    // This can be between 1 and 10
    match venue {
        venues::STAGE_A => 1,
        venues::STAGE_B => 2,
        venues::STAGE_C => 3,
        venues::WORKSHOP_1 => 4,
        venues::WORKSHOP_2 => 5,
        venues::WORKSHOP_3 => 6,
        venues::WORKSHOP_4 => 7,
        venues::WORKSHOP_5 => 8,
        venues::NULL_SECTOR => 9,
        _ => 10,
    }
}

fn shorten_venue_name(venue: &str) -> &str {
    match venue {
        venues::STAGE_A => "Stg A",
        venues::STAGE_B => "Stg B",
        venues::STAGE_C => "Stg C",
        venues::WORKSHOP_1 => "Wksp 1",
        venues::WORKSHOP_2 => "Wksp 2",
        venues::WORKSHOP_3 => "Wksp 3",
        venues::WORKSHOP_4 => "Wksp 4",
        venues::WORKSHOP_5 => "Wksp 5",
        venues::NULL_SECTOR => "Null Sec",
        _ => venue,
    }
}
