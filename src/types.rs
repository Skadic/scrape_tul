use std::{fmt::Display, num::ParseIntError, str::FromStr};

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LsfError {
    #[error("course with id '{0}' does not exist")]
    CourseDoesNotExist(usize),
    #[error("{0}")]
    InvalidCourseType(#[from] CourseTypeParseError),
    #[error("{0}")]
    InvalidRotation(#[from] RotationParseError),
    #[error("{0}")]
    InvalidSemesterType(#[from] SemesterTypeParseError),
    #[error("malformed course id")]
    MalformedCourseId(ParseIntError),
    #[error("malformed weekly hours")]
    MalformedWeeklyHours(ParseIntError),
    #[error("malformed year")]
    MalformedYear(ParseIntError),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CourseType {
    Lecture,
    Excercise,
    Practice,
    /// A seminar, 
    Seminar(SeminarType),
    Blockseminar,
    Proseminar,
    TechProject,
    ProjectGroup,
    Other,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SeminarType {
    Normal,
    Block
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CourseTypeParseError {
    #[error("invalid course type '{0}'")]
    Invalid(String),
}

impl FromStr for CourseType {
    type Err = CourseTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_lowercase().as_str() {
            "vorlesung" | "wahlpflichtvorlesung" | "vertiefungsvorlesung" => Self::Lecture,
            "übung" | "uebung" | "blockkurs" => Self::Excercise,
            "seminar" => Self::Seminar(SeminarType::Normal),
            "blockseminar" => Self::Seminar(SeminarType::Block),
            "proseminar" => Self::Proseminar,
            "praktikum" => Self::Practice,
            "fachprojekt" => Self::TechProject,
            "projektgruppe" => Self::ProjectGroup,
            _ => return Err(CourseTypeParseError::Invalid(s.to_string())),
        })
    }
}

impl Default for CourseType {
    fn default() -> Self {
        Self::Other
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Rotation {
    EverySemester,
    Yearly(Option<SemesterType>),
}

impl Default for Rotation {
    fn default() -> Self {
        Self::Yearly(None)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RotationParseError {
    #[error("invalid rotation: '{0}'")]
    Invalid(String),
}

impl TryFrom<(&str, SemesterType)> for Rotation {
    type Error = RotationParseError;
    fn try_from((rotation, semester): (&str, SemesterType)) -> Result<Self, Self::Error> {
        (rotation, Some(semester)).try_into()
    }
}

impl TryFrom<(&str, Option<SemesterType>)> for Rotation {
    type Error = RotationParseError;

    fn try_from((rotation, semester): (&str, Option<SemesterType>)) -> Result<Self, Self::Error> {
        Ok(match rotation.to_lowercase().as_str() {
            "jährlich" | "jedes 2. semester" => Self::Yearly(semester),
            "jedes semester" => Self::EverySemester, 
            _ => return Err(RotationParseError::Invalid(rotation.to_string())),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SemesterType {
    Winter,
    Summer,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SemesterTypeParseError {
    #[error("invalid semester type: '{0}'")]
    Invalid(String),
}

impl FromStr for SemesterType {
    type Err = SemesterTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_lowercase().as_str() {
            "wise" | "ws" | "winter" => Self::Winter,
            "sose" | "ss" | "sommer" => Self::Summer,
            _ => return Err(SemesterTypeParseError::Invalid(s.to_string())),
        })
    }
}

impl Display for SemesterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SemesterType::Winter => "WiSe",
                SemesterType::Summer => "SoSe",
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Semester {
    year: usize,
    semester_type: SemesterType,
}

impl Display for Semester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.semester_type, self.year)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Course {
    pub course_type: CourseType,
    pub course_id: usize,
    pub lsf_id: usize,
    pub rotation: Rotation,
    pub credits: u8,
    pub name: String,
    pub short_name: Option<String>,
    pub weekly_hours: u8,
}
