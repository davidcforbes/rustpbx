//! CRUD handlers for `iiz.a2p_campaigns`.

use crate::iiz::models::trust_center::{A2pCampaign, NewA2pCampaign, UpdateA2pCampaign};

crate::crud_handlers!(
    table: crate::iiz::schema::iiz::a2p_campaigns,
    entity: A2pCampaign,
    new_entity: NewA2pCampaign,
    update_entity: UpdateA2pCampaign,
);
