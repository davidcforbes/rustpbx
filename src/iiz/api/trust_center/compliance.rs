//! CRUD handlers for `iiz.compliance_requirements` and `iiz.compliance_applications`.

mod requirements {
    use crate::iiz::models::trust_center::{ComplianceRequirement, NewComplianceRequirement, UpdateComplianceRequirement};

    crate::crud_handlers!(
        table: crate::iiz::schema::iiz::compliance_requirements,
        entity: ComplianceRequirement,
        new_entity: NewComplianceRequirement,
        update_entity: UpdateComplianceRequirement,
    );
}

mod applications {
    use crate::iiz::models::trust_center::{ComplianceApplication, NewComplianceApplication, UpdateComplianceApplication};

    crate::crud_handlers!(
        table: crate::iiz::schema::iiz::compliance_applications,
        entity: ComplianceApplication,
        new_entity: NewComplianceApplication,
        update_entity: UpdateComplianceApplication,
    );
}

// Re-export with prefixed names for the router
pub use requirements::list as list_requirements;
pub use requirements::get as get_requirement;
pub use requirements::create as create_requirement;
pub use requirements::update as update_requirement;
pub use requirements::delete as delete_requirement;

pub use applications::list as list_applications;
pub use applications::get as get_application;
pub use applications::create as create_application;
pub use applications::update as update_application;
pub use applications::delete as delete_application;
