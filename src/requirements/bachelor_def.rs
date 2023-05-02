
use lazy_static::lazy_static;

use crate::requirements::PlanRequirements;

lazy_static! {
    pub static ref BACHELOR_REQUIREMENTS: PlanRequirements = {
        let mut req = PlanRequirements::Empty;
        
        // Required Courses
        req &= PlanRequirements::all_courses([
            "040105", // DAP 1
            "040115", // DAP 2
            "040101", // RS
            "040111", // BS
            "040501", // MafI 1
            "040503", // MafI 2
            "040113", // RvS,
            "040125", // Logik
            "040121", // HaPra 
            "050358", // WrumS
            "040131", // IS
            "040135", // SWT
            "040141", // GTI
            "080624", // ETKT
        ]);      

        // SoPra
        req &= PlanRequirements::any_courses(1, [
            "040137", // im semester
            "040138", // in semesterferien
        ]);

        // Software Wahlpflicht
        let software = PlanRequirements::any_courses(2, [
            "040215", //ÜBau
            "040217", //FuPro
            "040211", // SWK
        ]);
        
        let software_courses = software.inner();

        req &= software;


        let wahl = PlanRequirements::any(1,[

        ]);

        req
    };
}

