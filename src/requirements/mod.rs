use std::{
    borrow::Borrow,
    fmt::Debug,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
    sync::Arc,
};

pub mod bachelor_def;

pub enum PlanRequirements {
    Empty,
    /// A single course lsf id
    Course(&'static str),
    /// Represents that all of these requirements need to be fulfilled
    All(Arc<Vec<PlanRequirements>>),
    /// Represents that only a specified number of these requirements need to be fulfilled
    Any(usize, Arc<Vec<PlanRequirements>>),
}

impl PlanRequirements {
    pub const fn empty() -> Self {
        Self::Empty
    }

    pub const fn course(id: &'static str) -> Self {
        Self::Course(id)
    }

    pub fn all<'a>(reqs: impl IntoIterator<Item = &'a PlanRequirements>) -> Self {
        let reqs = reqs.into_iter().cloned().collect::<Vec<_>>();
        if reqs.is_empty() {
            return Self::Empty;
        }
        Self::All(Arc::new(reqs))
    }

    pub fn all_courses(courses: impl IntoIterator<Item = &'static str>) -> Self {
        let courses = courses.into_iter().map(Self::Course).collect::<Vec<_>>();
        if courses.is_empty() {
            return Self::Empty;
        }

        Self::All(Arc::new(courses))
    }

    pub fn any<'a>(
        num_required: usize,
        reqs: impl IntoIterator<Item = &'a PlanRequirements>,
    ) -> Self {
        let reqs = reqs.into_iter().cloned().collect::<Vec<_>>();
        if reqs.is_empty() {
            return Self::Empty;
        }
        Self::Any(num_required, Arc::new(reqs))
    }

    pub fn any_courses(
        num_required: usize,
        courses: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        let courses = courses.into_iter().map(Self::Course).collect::<Vec<_>>();
        if courses.is_empty() {
            return Self::Empty;
        }
        Self::Any(num_required, Arc::new(courses))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn inner(&self) -> Arc<Vec<PlanRequirements>> {
        match self {
            Self::Empty => Arc::new(vec![]),
            Self::Course(id) => Arc::new(vec![Self::Course(id)]),
            Self::All(reqs) | Self::Any(_, reqs) => Arc::clone(reqs),
        }
    }
}

impl Clone for PlanRequirements {
    fn clone(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Course(course) => Self::Course(course),
            Self::All(requirements) => Self::All(Arc::clone(requirements)),
            Self::Any(num_required, requirements) => {
                Self::Any(*num_required, Arc::clone(requirements))
            }
        }
    }
}

impl Debug for PlanRequirements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Course(id) => write!(f, "{id:?}"),
            Self::All(reqs) => f.debug_tuple("All").field(&reqs.as_slice()).finish(),
            Self::Any(num_required, reqs) => f
                .debug_tuple("Any")
                .field(num_required)
                .field(&reqs.as_slice())
                .finish(),
        }
    }
}

impl BitAnd for PlanRequirements {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        use PlanRequirements::*;
        if self.is_empty() {
            return rhs;
        }
        if rhs.is_empty() {
            return self;
        }
        match (self, rhs) {
            (All(lvec), All(rvec)) => match (Arc::try_unwrap(lvec), Arc::try_unwrap(rvec)) {
                (Ok(mut lvec), Ok(mut rvec)) => {
                    if lvec.len() > rvec.len() {
                        lvec.extend(rvec.into_iter());
                        All(Arc::new(lvec))
                    } else {
                        rvec.extend(lvec.into_iter());
                        All(Arc::new(rvec))
                    }
                }
                (Ok(lvec), Err(rvec_rc)) => All(Arc::new(vec![All(Arc::new(lvec)), All(rvec_rc)])),
                (Err(lvec_rc), Ok(rvec)) => All(Arc::new(vec![All(lvec_rc), All(Arc::new(rvec))])),
                (Err(lvec_rc), Err(rvec_rc)) => All(Arc::new(vec![All(lvec_rc), All(rvec_rc)])),
            },
            (l, r) => All(Arc::new(vec![l, r])),
        }
    }
}

impl BitAndAssign for PlanRequirements {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = std::mem::replace(self, unsafe { std::mem::zeroed() }) & rhs;
    }
}

impl BitOr for PlanRequirements {
    type Output = Self;

    #[rustfmt::skip]
    fn bitor(self, rhs: Self) -> Self::Output {
        use PlanRequirements::*;
        if self.is_empty() {
            return rhs;
        }
        if rhs.is_empty() {
            return self;
        }
        match (self, rhs) {
            (Self::Any(1, l), Self::Any(1, r)) => match (Arc::try_unwrap(l), Arc::try_unwrap(r)) {
                (Ok(mut lvec), Ok(mut rvec)) => {
                    if lvec.len() > rvec.len() {
                        lvec.extend(rvec.into_iter());
                        Any(1, Arc::new(lvec))
                    } else {
                        rvec.extend(lvec.into_iter());
                        Any(1, Arc::new(rvec))
                    }
                }
                (Ok(lvec), Err(rvec_rc)) => Any(1, Arc::new(vec![Any(1, Arc::new(lvec)), Any(1, rvec_rc)])),
                (Err(lvec_rc), Ok(rvec)) => Any(1, Arc::new(vec![Any(1, lvec_rc), Any(1, Arc::new(rvec))])),
                (Err(lvec_rc), Err(rvec_rc)) => Any(1, Arc::new(vec![Any(1, lvec_rc), Any(1, rvec_rc)])),
            },
            (l, r) => Self::Any(1, Arc::new(vec![l, r])),
        }
    }
}

impl BitOrAssign for PlanRequirements {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = std::mem::replace(self, unsafe { std::mem::zeroed() }) | rhs;
    }
}
