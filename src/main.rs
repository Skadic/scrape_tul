#![allow(unused)]

use futures::{stream::FuturesUnordered, Stream, StreamExt};
use log::{debug, error, trace, warn};
use scraper::{ElementRef, Selector};
use simplelog::{ConfigBuilder, TermLogger};
use types::{Course, LsfError};

use crate::{types::SemesterType, requirements::bachelor_def::BACHELOR_REQUIREMENTS};

const LSF_BASE: &str =
    "https://www.lsf.tu-dortmund.de/qisserver/rds?state=wtree&search=1&trex=step&root120222";

mod types;
mod requirements;

#[tokio::main]
async fn main() {
    setup_logger();

    // DAP1, PO, BS, BS Ü, PA Prosem, KoKoVa PG, GDV Blocksem, LaL Seminar, invalid
    let ids = [
        283038, 285849, 283046, 286735, 286085, 287390, 283059, 285848, 12345,
    ];

    debug!("{:#?}", *BACHELOR_REQUIREMENTS);

    let mut res = ids
        .iter()
        .copied()
        .map(parse_course)
        .collect::<FuturesUnordered<_>>();
    while let Some(course) = res.next().await {
        let course = match course {
            Ok(course) => course,
            Err(e) => {
                warn!("{e}");
                continue;
            }
        };
        debug!("{course:#?}")
    }
}

fn setup_logger() {
    let cfg = ConfigBuilder::new()
        .add_filter_allow_str("scrapperino")
        .build();

    TermLogger::init(
        log::LevelFilter::Debug,
        cfg,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Always,
    )
    .unwrap();
}

async fn parse_course(id: usize) -> Result<Course, LsfError> {
    let rp = reqwest::get(
        format!("https://www.lsf.tu-dortmund.de/qisserver/rds?state=verpublish&status=init&vmfile=no&publishid={id}&moduleCall=webInfo&publishConfFile=webInfo&publishSubDir=veranstaltung"),
    ).await.unwrap().text().await.unwrap();

    let document = scraper::html::Html::parse_document(&rp);

    let select_table_rows = Selector::parse(".form > table:first-of-type tr").unwrap();
    let select_table_elems = Selector::parse("th, td").unwrap();
    let select_name = Selector::parse(".form > h1:nth-child(1)").unwrap();

    let table_elements = document
        .select(&select_table_rows)
        .flat_map(|elem| elem.select(&select_table_elems))
        .collect::<Vec<_>>();

    let heading = document
        .select(&select_name)
        .next()
        .and_then(|elem| {
            elem.inner_html()
                .split('-')
                .next()
                .map(str::trim)
                .map(str::to_owned)
        })
        .ok_or(LsfError::CourseDoesNotExist(id))?;

    debug!("Processing course with name \"{heading}\"");

    let mut course = Course {
        lsf_id: id,
        ..Course::default()
    };
    let mut semester = None;
    let mut year = None;

    course.name = heading;

    for i in 0..table_elements.len() - 1 {
        if table_elements[i].value().name() != "th" {
            continue;
        }

        let header = &table_elements[i];
        let data = &table_elements[i + 1];
        trace!(
            "Header: {}, Data: {}",
            header.inner_html().trim(),
            data.inner_html().trim()
        );

        let data_html = data.inner_html();
        let data_content = data_html.trim();
        match header.inner_html().trim() {
            "Veranstaltungsart" => course.course_type = data_content.parse()?,
            "Veranstaltungsnummer" => {
                course.course_id = data_content.parse().map_err(LsfError::MalformedCourseId)?
            }
            "Kurztext" if !data_content.is_empty() => {
                course.short_name = Some(data_content.to_owned())
            }
            "SWS" => {
                course.weekly_hours = data_content
                    .parse()
                    .map_err(LsfError::MalformedWeeklyHours)?
            }
            "Rhythmus" => course.rotation = (data_content, None).try_into().unwrap_or_default(),
            "Semester" => {
                let mut split = data_content.split('-');
                let Some(sem) = split.next() else {
                    continue;
                };
                if let Ok(sem) = sem.parse::<SemesterType>() {
                    semester = Some(sem);
                }

                let Some(parsed_year) = split.next().and_then(|year| year.split('/').next().or(Some(year))) else {
                    continue;
                };
                let parsed_year = parsed_year
                    .parse::<usize>()
                    .map_err(LsfError::MalformedYear)?;
                year = Some(parsed_year);
            }
            "Credits" => {
                if let Ok(credits) = data_content.parse() {
                    course.credits = credits;
                }
            }
            _ => {}
        }
    }

    if let types::Rotation::Yearly(sem) = &mut course.rotation {
        *sem = semester;
    }

    Ok(course)
}
