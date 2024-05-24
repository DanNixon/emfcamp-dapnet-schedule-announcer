use dapnet_api::{OutgoingNews, OutgoingNewsBuilder};
use emfcamp_schedule_api::schedule::event::Event;
use tracing::error;

pub(crate) trait EventExt {
    fn to_rubric_news(&self) -> Option<OutgoingNews>;
}

impl EventExt for Event {
    fn to_rubric_news(&self) -> Option<OutgoingNews> {
        let venue = Venue::from_schedule_name(&self.venue);

        let news_number = news_number_for_venue(&venue);
        let msg = format!("{}: {}", venue_short_name(venue), self.title);

        match OutgoingNewsBuilder::default()
            .rubric("emfcamp".to_string())
            .number(news_number)
            .text(msg)
            .build()
        {
            Ok(news) => Some(news),
            Err(e) => {
                error!("Failed to build news: {e}");
                None
            }
        }
    }
}

enum Venue {
    StageA,
    StageB,
    StageC,
    Workshop0,
    Workshop1,
    Workshop2,
    Workshop3,
    Workshop4,
    Workshop5,
    Workshop6,
    YouthWorkshop,
    NullSector,
    Other(String),
}

impl Venue {
    fn from_schedule_name(v: &str) -> Self {
        match v {
            "Stage A" => Self::StageA,
            "Stage B" => Self::StageB,
            "Stage C" => Self::StageC,
            "Workshop 0 (Drop-in)" => Self::Workshop0,
            "Workshop 1 (NottingHack)" => Self::Workshop1,
            "Workshop 2 (Milliways)" => Self::Workshop2,
            "Workshop 3 (Furry High Commission)" => Self::Workshop3,
            "Workshop 4 (FieldFX)" => Self::Workshop4,
            "Workshop 5 (Maths)" => Self::Workshop5,
            "Workshop 6 (Hardware Hacking)" => Self::Workshop6,
            "Youth Workshop" => Self::YouthWorkshop,
            "Null Sector" => Self::NullSector,
            other => Self::Other(other.to_string()),
        }
    }
}

fn news_number_for_venue(venue: &Venue) -> i8 {
    // This can be between 1 and 10
    match venue {
        Venue::StageA => 1,
        Venue::StageB => 2,
        Venue::StageC => 3,
        Venue::Workshop0 => 4,
        Venue::Workshop1 => 4,
        Venue::Workshop2 => 4,
        Venue::Workshop3 => 4,
        Venue::Workshop4 => 4,
        Venue::Workshop5 => 4,
        Venue::Workshop6 => 4,
        Venue::YouthWorkshop => 5,
        Venue::NullSector => 6,
        _ => 10,
    }
}

fn venue_short_name(venue: Venue) -> String {
    match venue {
        Venue::StageA => "Stg A".to_string(),
        Venue::StageB => "Stg B".to_string(),
        Venue::StageC => "Stg C".to_string(),
        Venue::Workshop0 => "Wksp 0".to_string(),
        Venue::Workshop1 => "Wksp 1".to_string(),
        Venue::Workshop2 => "Wksp 2".to_string(),
        Venue::Workshop3 => "Wksp 3".to_string(),
        Venue::Workshop4 => "Wksp 4".to_string(),
        Venue::Workshop5 => "Wksp 5".to_string(),
        Venue::Workshop6 => "Wksp 6".to_string(),
        Venue::YouthWorkshop => "Yth Wksp".to_string(),
        Venue::NullSector => "Nul Sec".to_string(),
        Venue::Other(name) => name,
    }
}
