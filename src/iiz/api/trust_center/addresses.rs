//! CRUD handlers for `iiz.compliance_addresses`.

use crate::iiz::models::trust_center::{ComplianceAddress, NewComplianceAddress, UpdateComplianceAddress};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::compliance_addresses,
    entity: ComplianceAddress,
    new_entity: NewComplianceAddress,
    update_entity: UpdateComplianceAddress,
);
