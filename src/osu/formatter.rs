use rosu_v2::prelude as rosu;

pub fn map_title(map: &rosu::BeatmapExtended) -> String {
    let mapset = map.mapset.as_deref().expect("missing mapset");
    format!("{} - {} [{}]", mapset.artist, mapset.title, map.version)
}

pub fn osu_hits(score_statistics: &rosu::ScoreStatistics) -> String {
    format!("{}/{}/{}/{}", score_statistics.great, score_statistics.ok, score_statistics.meh, score_statistics.miss)
}

pub fn score_url(score_id: &u64) -> String {
    format!("https://osu.ppy.sh/scores/{}", score_id.to_string())
}

pub fn mods_string(mods: &rosu::GameMods) -> String {
    mods.iter().map(|map: &rosu::GameMod| map.acronym().to_string()).collect::<Vec<_>>().join("")
} 